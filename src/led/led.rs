use embassy_time::{Duration, Timer};

use esp_hal_smartled::{buffer_size_async, SmartLedsAdapterAsync};
use esp_hal::{peripherals, rmt::{Channel, Rmt}, time::Rate, Async, gpio::AnyPin};
use smart_leds::{
    brightness, gamma,
    hsv::{hsv2rgb, Hsv},
    SmartLedsWriteAsync, RGB8,
};


pub enum Mode {
    Solid,
    Fade{fade_duration: u8},
    ColourFade{fade_duration: u8},
    ColourChanger{speed: u8},
}

pub enum Cmd {
    OnOff(bool),
    Level(u8),
    Colour{r: u8, g: u8, b: u8},
    Mode(Mode),
}

pub struct Driver {
    led: SmartLedsAdapterAsync<Channel<Async, 0>, 25>,
    level: u8,
    colour: Hsv,
}

impl Driver {
    pub fn new(rmt: peripherals::RMT, pin: AnyPin) -> Self {

        // Setup the LED
        // Configure RMT (Remote Control Transceiver) peripheral globally
        // <https://docs.espressif.com/projects/esp-idf/en/stable/esp32/api-reference/peripherals/rmt.html>
        let rmt: Rmt<'_, esp_hal::Async> = {
            let frequency: Rate = {Rate::from_mhz(80)};
            Rmt::new(rmt, frequency)
        }
        .expect("Failed to initialize RMT")
        .into_async();

        // We use one of the RMT channels to instantiate a `SmartLedsAdapterAsync` which can
        // be used directly with all `smart_led` implementations
        let rmt_channel = rmt.channel0;
        let rmt_buffer = [0_u32; buffer_size_async(1)];

        // Each devkit uses a unique GPIO for the RGB LED, so in order to support
        // all chips we must unfortunately use `#[cfg]`s:
        let led: SmartLedsAdapterAsync<_, 25> = {
            SmartLedsAdapterAsync::new(rmt_channel, pin, rmt_buffer)
        };

        Self{
            led,
            colour: Hsv {
                hue: 0,
                sat: 255,
                val: 255,
            },
            level: 255,
        }
    }

    pub async fn run(mut self) -> ! {
        loop {
            for hue in 0..=255 {
                self.colour.hue = hue;
                // Convert from the HSV colour space (where we can easily transition from one
                // colour to the other) to the RGB colour space that we can then send to the LED
                let data: RGB8 = hsv2rgb(self.colour);
                // When sending to the LED, we do a gamma correction first (see smart_leds
                // documentation for details) and then limit the brightness to 10 out of 255 so
                // that the output is not too bright.
                self.led.write(brightness(gamma([data].into_iter()), self.level))
                    .await
                    .unwrap();
                Timer::after(Duration::from_millis(10)).await;
            }
        }
    }
}
