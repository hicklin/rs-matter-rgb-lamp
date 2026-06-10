use core::cell::Cell;

#[cfg(feature = "defmt")]
use defmt::{debug, error};
#[cfg(feature = "log")]
use log::{debug};

use rs_matter_embassy::matter::dm::Cluster;
use rs_matter_embassy::matter::dm::clusters::app::{
    level_control::{self, LevelControlHooks, OptionsBitmap},
    on_off::{self, OnOffHooks, StartUpOnOffEnum},
};
use rs_matter_embassy::matter::error::Error;
use rs_matter_embassy::matter::tlv::Nullable;
use rs_matter_embassy::matter::with;

use crate::led::{ColorLedSend, LedSend};
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;

use crate::dm::color_control::ColorControlHooks;
use rs_matter_embassy::matter::error::ErrorCode;
use palette::{
    white_point::D65,
    FromColor,
    Srgb,
    Yxy,
};

use embassy_sync::signal::Signal;
use embassy_futures::select::select;

const BRIGHTNESS_INCREMENT: u8 = 5;

#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct LedHandler<S: LedSend> {
    sender: S,
    on_off_signal: &'static Signal<CriticalSectionRawMutex, bool>,
    level_signal: &'static Signal<CriticalSectionRawMutex, bool>,
    color_signal: &'static Signal<CriticalSectionRawMutex, bool>,
    mode_signal: &'static Signal<CriticalSectionRawMutex, bool>,
    // OnOff Attributes
    on_off: Cell<bool>,
    start_up_on_off: Cell<Option<StartUpOnOffEnum>>,
    // LevelControl Attributes
    current_level: Cell<Option<u8>>,
    startup_current_level: Cell<Option<u8>>,
}

impl< S: LedSend> LedHandler<S> {
    pub fn new(
        sender: S,
        on_off_signal: &'static Signal<CriticalSectionRawMutex, bool>,
        level_signal: &'static Signal<CriticalSectionRawMutex, bool>,
        color_signal: &'static Signal<CriticalSectionRawMutex, bool>,
        mode_signal: &'static Signal<CriticalSectionRawMutex, bool>,
    ) -> Self {
        Self {
            sender,
            on_off_signal,
            level_signal,
            color_signal,
            mode_signal,
            on_off: Cell::new(true),
            start_up_on_off: Cell::new(None),
            current_level: Cell::new(Some(42)),
            startup_current_level: Cell::new(None),
        }
    }

    pub async fn run(&self) {
        loop {
            select(self.run_color_task(), self.run_mode_task()).await;
        }
    }

    // TODO: This task will be moved to the colour cluster's run impl.
    async fn run_color_task(&self) {
        loop {
            match self.color_signal.wait().await {
                true => todo!(),
                false => todo!(),
            }
        }
    }

    async fn run_mode_task(&self) {
        loop {
            match self.mode_signal.wait().await {
                true => todo!(),
                false => todo!(),
            }
        }
    }
}

impl< S: LedSend> OnOffHooks for LedHandler<S> {
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
        self.sender.try_set_on(on);
        self.on_off.set(on);
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

    async fn run<F: Fn(on_off::OutOfBandMessage)>(&self, notify: F) {
        loop {
            self.on_off_signal.wait().await;
            notify(on_off::OutOfBandMessage::Toggle);
        }
    }
}

impl< S: LedSend> LevelControlHooks for LedHandler<S> {
    const MIN_LEVEL: u8 = 1;

    const MAX_LEVEL: u8 = S::MAX_LED_LEVEL;

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
            .try_set_brightness(level)
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

    async fn run<F: Fn(level_control::OutOfBandMessage)>(&self, notify: F) {
        loop {
            let level = match self.level_signal.wait().await {
                true => {
                    Self::MAX_LEVEL.min(
                        self.current_level().unwrap_or(Self::MIN_LEVEL).saturating_add(BRIGHTNESS_INCREMENT))
                },
                false => {
                    Self::MIN_LEVEL.max(
                        self.current_level().unwrap_or(Self::MIN_LEVEL).saturating_sub(BRIGHTNESS_INCREMENT))
                },
            };

            notify(level_control::OutOfBandMessage::MoveToLevel {
                with_on_off: true,
                level: level,
                transition_time: Some(0),
                options_mask: OptionsBitmap::default(),
                options_override: OptionsBitmap::default(),
            });
        }
    }
}

impl< S: ColorLedSend> ColorControlHooks for LedHandler<S> {
    fn set_color(&self, x: u16, y: u16) -> Result<(), Error> {
        let x_f32 = x as f32 / 65536.0;
        let y_f32 = y as f32 / 65536.0;

        let yxy: Yxy<D65, f32> = Yxy::new(x_f32, y_f32, 1.0);

        let srgb: Srgb<f32> = Srgb::from_color(yxy);

        let r = (srgb.red * 255.0) as u8;
        let g = (srgb.green * 255.0) as u8;
        let b = (srgb.blue * 255.0) as u8;

        self.sender
            .try_set_colour(r, g, b)
            .map_err(|_| ErrorCode::Busy.into())
    }
}
