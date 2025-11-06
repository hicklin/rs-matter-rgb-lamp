use core::cell::{Cell, RefCell};

use embassy_futures::select::{Either, select};
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::channel::{Receiver, Sender};
use embassy_time::{Duration, Timer};

#[cfg(feature = "defmt")]
use defmt::{debug, error, warn};
#[cfg(feature = "log")]
use log::{debug, error, warn};

use esp_hal::{
    gpio::AnyPin,
    peripherals,
    rmt::{PulseCode, Rmt},
    time::Rate,
};
use esp_hal_smartled::{LedAdapterError, SmartLedsAdapterAsync, buffer_size_async};
use smart_leds::{
    RGB8, SmartLedsWriteAsync, brightness, gamma,
    hsv::{Hsv, hsv2rgb},
};

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Mode {
    Solid,
    // Duration represents the time to travers from min to max brightness.
    Pulse { duration: Duration },
    ColourPulsing { pulse_duration: u8 },
    // Duration represents the time to complete one cycle.
    ColourChanging { duration: Duration },
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ControlMessage {
    SetOn(bool),
    SetBrightness(u8),
    SetColour { r: u8, g: u8, b: u8 },
    SetMode(Mode),
    Reset,
}

pub type LedSender<'a> = Sender<'a, CriticalSectionRawMutex, ControlMessage, 4>;
pub type LedReceiver<'a> = Receiver<'a, CriticalSectionRawMutex, ControlMessage, 4>;

pub struct Driver<'a> {
    led: RefCell<SmartLedsAdapterAsync<'a, 25>>,
    receiver: LedReceiver<'a>,
    level: Cell<u8>,
    colour: Cell<RGB8>,
    mode: Mode,
}

impl<'a> Driver<'a> {
    pub fn new(rmt: peripherals::RMT<'a>, pin: AnyPin<'a>, receiver: LedReceiver<'a>) -> Self {
        // Setup the LED
        // Configure RMT (Remote Control Transceiver) peripheral globally
        // <https://docs.espressif.com/projects/esp-idf/en/stable/esp32/api-reference/peripherals/rmt.html>
        let rmt: Rmt<'_, esp_hal::Async> = {
            let frequency: Rate = { Rate::from_mhz(80) };
            Rmt::new(rmt, frequency)
        }
        .expect("Failed to initialize RMT")
        .into_async();

        // We use one of the RMT channels to instantiate a `SmartLedsAdapterAsync` which can
        // be used directly with all `smart_led` implementations
        let rmt_channel = rmt.channel0;
        let rmt_buffer = [PulseCode::default(); buffer_size_async(1)];

        // Each devkit uses a unique GPIO for the RGB LED, so in order to support
        // all chips we must unfortunately use `#[cfg]`s:
        let led = { SmartLedsAdapterAsync::new(rmt_channel, pin, rmt_buffer) };

        Self {
            led: RefCell::new(led),
            receiver,
            colour: Cell::new(RGB8 {
                r: 239,
                g: 235,
                b: 216,
            }),
            level: Cell::new(150),
            mode: Mode::Solid,
        }
    }

    // Sets the LED to the current values.
    async fn update_led(&self) -> Result<(), LedAdapterError> {
        let colour = self.colour.get();
        debug!(
            "Updating LED: colour: {}, {}, {} | level: {}",
            colour.r,
            colour.g,
            colour.b,
            self.level.get()
        );

        #[allow(clippy::await_holding_refcell_ref)]
        match self.led.try_borrow_mut() {
            Ok(mut led) => {
                // This operation should be quick
                led.write(brightness(
                    gamma([self.colour.get()].into_iter()),
                    self.level.get(),
                ))
                .await
            }
            Err(_) => {
                error!("unable to update LED. Skipping");
                Ok(())
            }
        }
    }

    pub async fn run(mut self) -> ! {
        self.update_led().await.unwrap();
        loop {
            match select(self.receiver.receive(), self.run_mode()).await {
                Either::First(command) => {
                    match command {
                        ControlMessage::SetOn(_on) => {
                            // todo physically switch the LED off, i.e. cut power.
                            // unsure if this is possible for the esp32c6.
                        }
                        ControlMessage::SetBrightness(level) => {
                            self.level.set(level);
                            self.update_led().await.unwrap();
                        }
                        ControlMessage::SetColour { r, g, b } => {
                            self.colour.set(RGB8 { r, g, b });
                            self.update_led().await.unwrap();
                        }
                        ControlMessage::SetMode(mode) => {
                            warn!("Only Solid mode supported at this time");
                            self.mode = mode;
                        }
                        ControlMessage::Reset => {
                            self.colour.set(RGB8 {
                                r: 220,
                                g: 100,
                                b: 20,
                            });
                            self.level = Cell::new(255);
                            self.mode = Mode::Solid;
                            self.update_led().await.unwrap();
                        }
                    }
                }
                Either::Second(_) => {
                    warn!("mode task exited unexpectedly");
                }
            }
        }
    }

    async fn run_mode(&self) {
        match self.mode {
            Mode::Solid => core::future::pending::<()>().await,
            Mode::Pulse { duration } => {
                // Limit minimum to 500 milliseconds
                let duration = duration.max(Duration::from_millis(500));

                let max_level = self.level.get();
                let mut direction_up = true;

                loop {
                    match direction_up {
                        true => {
                            self.level.set(self.level.get().saturating_add(1));
                            if self.level.get() >= max_level {
                                direction_up = false;
                            }
                        }
                        false => {
                            self.level.set(self.level.get().saturating_sub(1));
                            if self.level.get() <= 1 {
                                direction_up = true;
                            }
                        }
                    }
                    self.update_led().await.unwrap();
                    Timer::after(duration.checked_div(max_level as u32).unwrap()).await
                }
            }
            Mode::ColourPulsing { pulse_duration: _ } => {
                // todo implement
                core::future::pending::<()>().await
            }
            Mode::ColourChanging { duration } => {
                // Limit minimum to 500 milliseconds
                let duration = duration.max(Duration::from_millis(500));

                let mut hue: u8 = 0;

                loop {
                    hue = hue.wrapping_add(1);

                    let hue_colour = Hsv {
                        hue,
                        sat: 255,
                        val: 255,
                    };

                    self.colour.set(hsv2rgb(hue_colour));
                    self.update_led().await.unwrap();

                    Timer::after(duration.checked_div(255).unwrap()).await;
                }
            }
        }
    }
}
