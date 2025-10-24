use core::cell::Cell;
#[cfg(feature = "defmt")]
use defmt::debug;
#[cfg(feature = "log")]
use log::debug;

use rs_matter_embassy::matter::dm::Cluster;
use rs_matter_embassy::matter::dm::clusters::level_control::{self, LevelControlHooks};
use rs_matter_embassy::matter::dm::clusters::on_off::{self, OnOffHooks, StartUpOnOffEnum};
use rs_matter_embassy::matter::error::Error;
use rs_matter_embassy::matter::tlv::Nullable;
use rs_matter_embassy::matter::with;

use crate::led::led_driver::{ControlMessage, LedSender};

#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct LedHandler<'a> {
    sender: LedSender<'a>,
    // OnOff Attributes
    on_off: Cell<bool>,
    start_up_on_off: Cell<Option<StartUpOnOffEnum>>,
    // LevelControl Attributes
    current_level: Cell<Option<u8>>,
    startup_current_level: Cell<Option<u8>>,
}

impl<'a> LedHandler<'a> {
    pub const fn new(sender: LedSender<'a>) -> Self {
        Self {
            sender,
            on_off: Cell::new(true),
            start_up_on_off: Cell::new(None),
            current_level: Cell::new(Some(42)),
            startup_current_level: Cell::new(None),
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

    // todo this method should probably return an error `.map_err(|_| Error::new(ErrorCode::Busy))`
    fn set_on_off(&self, on: bool) {
        let _ = self.sender.try_send(ControlMessage::SetOn(on));
        self.on_off.set(on);
        debug!("OnOff state set to: {}", on);
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

    const FASTEST_RATE: u8 = 50;

    const CLUSTER: Cluster<'static> = level_control::FULL_CLUSTER
        .with_features(
            level_control::Feature::LIGHTING.bits() | level_control::Feature::ON_OFF.bits(),
        )
        .with_attrs(with!(
            required;
            level_control::AttributeId::CurrentLevel
            | level_control::AttributeId::RemainingTime
            | level_control::AttributeId::MinLevel
            | level_control::AttributeId::MaxLevel
            | level_control::AttributeId::OnOffTransitionTime
            | level_control::AttributeId::OnLevel
            | level_control::AttributeId::OnTransitionTime
            | level_control::AttributeId::OffTransitionTime
            | level_control::AttributeId::DefaultMoveRate
            | level_control::AttributeId::Options
            | level_control::AttributeId::StartUpCurrentLevel
        ))
        .with_cmds(with!(
            level_control::CommandId::MoveToLevel
                | level_control::CommandId::Move
                | level_control::CommandId::Step
                | level_control::CommandId::Stop
                | level_control::CommandId::MoveToLevelWithOnOff
                | level_control::CommandId::MoveWithOnOff
                | level_control::CommandId::StepWithOnOff
                | level_control::CommandId::StopWithOnOff
        ));

    fn set_device_level(&self, level: u8) -> Result<Option<u8>, ()> {
        debug!("LedHandler::set_device_level: level {}", level);
        self.sender
            .try_send(ControlMessage::SetBrightness(level))
            .map_err(|_| ())?;
        Ok(Some(level))
    }

    fn current_level(&self) -> Option<u8> {
        self.current_level.get()
    }

    fn set_current_level(&self, level: Option<u8>) {
        debug!("LedHandler::set_current_level: level {:?}", level);
        self.current_level.set(level)
    }

    fn start_up_current_level(&self) -> Result<Option<u8>, Error> {
        Ok(self.startup_current_level.get())
    }

    fn set_start_up_current_level(&self, value: Option<u8>) -> Result<(), Error> {
        self.startup_current_level.set(value);
        Ok(())
    }
}
