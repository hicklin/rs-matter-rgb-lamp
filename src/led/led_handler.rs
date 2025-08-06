use core::cell::Cell;
use crate::led::led::{LedSender, ControlMessage};
use crate::data_model::clusters::level_control::{OptionsBitmap};
use crate::data_model::level_control::LevelControlHooks;
use crate::data_model::on_off::OnOffHooks;

use rs_matter::tlv::Nullable;
use rs_matter::error::{Error, ErrorCode};
use rs_matter::data_model::objects::{InvokeContext};


pub struct LedHandler<'a> {
    sender: LedSender<'a>,
    // OnOff Attributes
    on_off: Cell<bool>,
    // LevelControl Attributes
    options: Cell<OptionsBitmap>,
    on_level: Cell<Nullable<u8>>,
    current_level: Cell<u8>,
    startup_current_level: Cell<Nullable<u8>>,
    remaining_time: Cell<u16>,
}

impl<'a> LedHandler<'a> {
    pub const fn new(sender: LedSender<'a>) -> Self {
        Self {
            sender,
            on_off: Cell::new(false),
            options: Cell::new(OptionsBitmap::from_bits(OptionsBitmap::EXECUTE_IF_OFF.bits() as u8)
                .unwrap()),
            on_level: Cell::new(Nullable::some(42)),
            current_level: Cell::new(1),
            startup_current_level: Cell::new(Nullable::some(73)),
            remaining_time: Cell::new(0),
        }
    }
}

impl<'a> OnOffHooks for LedHandler<'a> {
    fn raw_get_on_off(&self) -> bool {
        self.on_off.get()
    }
    
    fn raw_set_on_off(&self, on: bool) -> Result<(), Error> {
        self.on_off.set(on);
        Ok(())
    }    

    fn set_on(&self, _ctx: &InvokeContext<'_>, on: bool) -> Result<(), Error> {
        match on {
            true =>  self.sender.try_send(ControlMessage::SetOn(Some(150))).map_err(|_| Error::new(ErrorCode::Busy)),
            false => self.sender.try_send(ControlMessage::SetOn(None)).map_err(|_| Error::new(ErrorCode::Busy)),
        }
    }
}

impl<'a> LevelControlHooks for LedHandler<'a> {
    const MIN_LEVEL: u8 = 1;

    const MAX_LEVEL: u8 = 254;

    fn set_level(&self, _ctx: &InvokeContext<'_>, level: u8) -> Result<(), Error> {
        self.sender.try_send(ControlMessage::SetBrightness(level)).map_err(|_| ErrorCode::Busy.into())
    }
    
    fn raw_get_options(&self) -> OptionsBitmap {
        self.options.get()
    }
    
    fn raw_set_options(&self, value: OptionsBitmap) -> Result<(), Error> {
        self.options.set(value);
        Ok(())
    }
    
    fn raw_get_on_level(&self) -> Nullable<u8> {
        // todo can we impl Copy for Nullable?
        let val = self.on_level.take();
        self.on_level.set(val.clone());
        val
    }
    
    fn raw_set_on_level(&self, value: Nullable<u8>) -> Result<(), Error> {
        self.on_level.set(value);
        Ok(())
    }
    
    fn raw_get_current_level(&self) -> u8 {
        self.current_level.get()
    }
    
    fn raw_set_current_level(&self, value: u8) -> Result<(), Error> {
        self.current_level.set(value);
        Ok(())
    }
    
    fn raw_get_startup_current_level(&self) -> Nullable<u8> {
        let val = self.startup_current_level.take();
        self.startup_current_level.set(val.clone());
        val
    }
    
    fn raw_set_startup_current_level(&self, value: Nullable<u8>) -> Result<(), Error> {
        self.startup_current_level.set(value);
        Ok(())
    }
    
    fn raw_get_remaining_time(&self) -> u16 {
        self.remaining_time.get()
    }
    
    fn raw_set_remaining_time(&self, value: u16) -> Result<(), Error> {
        self.remaining_time.set(value);
        Ok(())
    }
}