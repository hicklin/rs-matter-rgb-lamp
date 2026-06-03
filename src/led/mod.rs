pub mod dimmable_led_driver;
pub mod rgb_led_driver;
pub mod led_handler;

pub trait LedSend {
    fn try_set_on(&self, on: bool);
    fn try_set_brightness(&self, level: u8) -> Result<(), ()>;
}

pub trait ColorLedSend: LedSend {
    fn try_set_colour(&self, r: u8, g: u8, b: u8) -> Result<(), ()>;
}
