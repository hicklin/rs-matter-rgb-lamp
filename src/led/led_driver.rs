use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::channel::{Receiver, Sender};

#[cfg(feature = "defmt")]
use defmt::{debug, warn};
#[cfg(feature = "log")]
use log::{debug, warn};

use esp_hal::{
    gpio::AnyPin,
    peripherals,
    rmt::{PulseCode, Rmt},
    time::Rate,
};
use esp_hal_smartled::{LedAdapterError, SmartLedsAdapterAsync, buffer_size_async};
use smart_leds::{RGB8, SmartLedsWriteAsync, brightness, gamma};

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Mode {
    Solid,
    Pulse { pulse_duration: u8 },
    ColourPulsing { pulse_duration: u8 },
    ColourChanging { speed: u8 },
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
    led: SmartLedsAdapterAsync<'a, 25>,
    receiver: LedReceiver<'a>,
    level: u8,
    colour: RGB8,
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
            led,
            receiver,
            colour: RGB8 {
                r: 239,
                g: 235,
                b: 216,
            },
            level: 150,
            mode: Mode::Solid,
        }
    }

    // Sets the LED to the current values.
    async fn update_led(&mut self) -> Result<(), LedAdapterError> {
        debug!(
            "Updating LED: colour: {}, {}, {} | level: {}",
            self.colour.r, self.colour.g, self.colour.b, self.level
        );
        self.led
            .write(brightness(gamma([self.colour].into_iter()), self.level))
            .await
    }

    pub async fn run(mut self) -> ! {
        self.update_led().await.unwrap();

        loop {
            // todo: When we have effects, turn this into a select with a timeout of 100 ms.
            let command = self.receiver.receive().await;

            match command {
                ControlMessage::SetOn(_on) => {
                    // todo physically switch the LED off, i.e. cut power.
                    // unsure if this is possible of the esp32c6.
                }
                ControlMessage::SetBrightness(level) => {
                    self.level = level;
                    self.update_led().await.unwrap();
                }
                ControlMessage::SetColour { r, g, b } => {
                    self.colour = RGB8 { r, g, b };
                    self.update_led().await.unwrap();
                }
                ControlMessage::SetMode(mode) => {
                    warn!("Only Solid mode supported at this time");
                    self.mode = mode;
                }
                ControlMessage::Reset => {
                    self.colour = RGB8 {
                        r: 220,
                        g: 100,
                        b: 20,
                    };
                    self.level = 255;
                    self.mode = Mode::Solid;
                    self.update_led().await.unwrap();
                }
            }

            // todo: do something similar for the ColourChanger mode.
            // for hue in 0..=255 {
            //     let hue_colour = Hsv {
            //         hue: hue,
            //         sat: 255,
            //         val: 255,
            //     };
            //     // Convert from the HSV colour space (where we can easily transition from one
            //     // colour to the other) to the RGB colour space that we can then send to the LED
            //     self.colour = hsv2rgb(hue_colour);
            //     // When sending to the LED, we do a gamma correction first (see smart_leds
            //     // documentation for details) and then limit the brightness to 10 out of 255 so
            //     // that the output is not too bright.
            //     self.update_led().await.unwrap();
            //     Timer::after(Duration::from_millis(10)).await;
            // }
        }
    }
}
