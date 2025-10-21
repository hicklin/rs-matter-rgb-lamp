use core::cell::Cell;
use log::info;

use rs_matter_embassy::matter::tlv::Nullable;
use rs_matter_embassy::matter::error::{Error, ErrorCode};
use rs_matter_embassy::matter::dm::{InvokeContext, Cluster};
use rs_matter_embassy::matter::with;
use rs_matter_embassy::matter::dm::clusters::on_off::{self, OnOffHooks, StartUpOnOffEnum};

use crate::led::led::{LedSender, ControlMessage};
use crate::data_model::clusters::level_control::{OptionsBitmap};
use crate::data_model::level_control::LevelControlHooks;


pub struct LedHandler<'a> {
    sender: LedSender<'a>,
    // OnOff Attributes
    on_off: Cell<bool>,
    start_up_on_off: Cell<Option<StartUpOnOffEnum>>,
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
            start_up_on_off: Cell::new(None),
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
    const CLUSTER: Cluster<'static> = on_off::FULL_CLUSTER
        .with_revision(6)
        .with_features(on_off::Feature::LIGHTING.bits())
        .with_attrs(with!(
            required;
            on_off::AttributeId::OnOff
            | on_off::AttributeId::GlobalSceneControl
            | on_off::AttributeId::OnTime
            | on_off::AttributeId::OffWaitTime
            | on_off::AttributeId::StartUpOnOff
        ))
        .with_cmds(with!(
            on_off::CommandId::Off
                | on_off::CommandId::On
                | on_off::CommandId::Toggle
                | on_off::CommandId::OffWithEffect
                | on_off::CommandId::OnWithRecallGlobalScene
                | on_off::CommandId::OnWithTimedOff
        ));

    fn on_off(&self) -> bool {
        self.on_off.get()
    }

    fn set_on_off(&self, on: bool) {
        match on {
            // todo this method should probably return an error `.map_err(|_| Error::new(ErrorCode::Busy))`
            true =>  { let _ = self.sender.try_send(ControlMessage::SetOn(Some(150))); },
            false => { let _ = self.sender.try_send(ControlMessage::SetOn(None)); },
        }
        self.on_off.set(on);
        info!("OnOff state set to: {}", on);
    }

    fn start_up_on_off(&self) -> Nullable<on_off::StartUpOnOffEnum> {
        match self.start_up_on_off.get() {
            Some(value) => Nullable::some(value),
            None => Nullable::none(),
        }
    }

    fn set_start_up_on_off(&self, value: Nullable<on_off::StartUpOnOffEnum>) -> Result<(), Error> {
        self.start_up_on_off.set(value.into_option());
        Ok(())
    }

    async fn handle_off_with_effect(&self, _effect: on_off::EffectVariantEnum) {
        // no effect
    }
}

impl<'a> LevelControlHooks for LedHandler<'a> {
    const MIN_LEVEL: u8 = 1;

    const MAX_LEVEL: u8 = 254;

    fn set_level(&self, _ctx: impl InvokeContext, level: u8) -> Result<(), Error> {
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