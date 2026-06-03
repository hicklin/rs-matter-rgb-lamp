//! This LED driver drives LEDs that only support On/Off and Brightness functionality.
//! The driver provides setup and control functions as well as a LED behaviour modes.

use core::cell::{Cell, RefCell};

use embassy_futures::select::{Either, select};
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::channel::{Receiver, Sender};
use embassy_time::{Duration, Timer};

use esp_hal::ledc::channel::ChannelIFace;
use esp_hal::ledc::{
    LowSpeed,
    channel::Channel,
};

#[cfg(feature = "defmt")]
use defmt::{error, warn};
#[cfg(feature = "log")]
use log::{error, warn};

/// Defines the behaviour of the light.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Mode {
    /// The light remains solid and not changing.
    Solid,
    /// Duration represents the time to travers from min to max brightness.
    Pulse { duration: Duration },
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ControlMessage {
    SetOn(bool),
    SetBrightness(u8),
    SetMode(Mode),
    Reset,
}

pub type LedSender<'a> = Sender<'a, CriticalSectionRawMutex, ControlMessage, 4>;
pub type LedReceiver<'a> = Receiver<'a, CriticalSectionRawMutex, ControlMessage, 4>;

pub struct DimmableLedDriver<'a> {
    led: RefCell<Channel<'a, LowSpeed>>,
    receiver: LedReceiver<'a>,
    level: Cell<u8>,
    mode: Mode,
    invert: bool
}

impl<'a> DimmableLedDriver<'a> {
    pub fn new(
        channel: Channel<'a, LowSpeed>,
        receiver: LedReceiver<'a>,
        invert: bool,
    ) -> Self {
        Self {
            led: RefCell::new(channel),
            receiver,
            level: Cell::new(150),
            mode: Mode::Solid,
            invert,
        }
    }

    // Sets the LED to the current values.
    fn update_led(&self) {
        match self.led.try_borrow_mut() {
            Ok(led) => {
                let level = match self.invert {
                    true => 0xff - self.level.get(),
                    false => self.level.get(),
                };
                // todo is this needed?
                let level = level*100/255;
                if let Err(_) = led.set_duty(level) {
                    error!("unable to update LED. Skipping");
                }
            }
            Err(_) => {
                error!("unable to update LED. Skipping");
            }
        }
    }

    pub async fn run(mut self) -> ! {
        self.update_led();
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
                            self.update_led();
                        }
                        ControlMessage::SetMode(mode) => {
                            warn!("Only Solid mode supported at this time");
                            self.mode = mode;
                        }
                        ControlMessage::Reset => {
                            self.level = Cell::new(255);
                            self.mode = Mode::Solid;
                            self.update_led();
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
                    self.update_led();
                    Timer::after(duration.checked_div(max_level as u32).unwrap()).await
                }
            }
        }
    }
}
