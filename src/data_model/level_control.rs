use core::cell::Cell;
use core::ops::BitAnd;
use log::{warn, info, debug};
use rs_matter::data_model::objects::Dataver;
use rs_matter::tlv::{AsNullable, Nullable};
use rs_matter::utils::maybe::Maybe;

use crate::data_model::clusters::level_control;
pub use crate::data_model::clusters::level_control::*;
use rs_matter::reexport::bitflags::Flags;
use rs_matter::utils::init::Init;
use rs_matter::with;
use rs_matter::error::{Error, ErrorCode};

pub struct LevelControlHandler {
    dataver: Dataver,
    on: Cell<bool>,
    options: OptionsBitmap,
    on_level: Cell<Nullable<u8>>,
    current_level: Cell<Nullable<u8>>,
    startup_current_level: Cell<Nullable<u8>>,
}

impl LevelControlHandler {
    const MIN_LEVEL: u8 = 1;
    const MAX_LEVEL: u8 = 255;

    pub fn new(dataver: Dataver) -> Self {
        Self {
            dataver,
            on: Cell::new(false),
            options: OptionsBitmap::from_bits(level_control::OptionsBitmap::EXECUTE_IF_OFF.bits() as u8)
                .unwrap(),
            on_level: Cell::new(Nullable::new(Some(1))),
            current_level: Cell::new(Nullable::new(Some(0))),
            startup_current_level: Cell::new(Nullable::new(Some(5))),
        }
    }

    // Processes the options of commands 'without On/Off'.
    // Returns true if execution of the command should continue, false otherwise.
    fn should_continue(&self, options_mask: OptionsBitmap, options_override: OptionsBitmap) -> bool {
        let temporary_options = (options_mask & options_override) | self.options;

        if temporary_options.contains(level_control::OptionsBitmap::EXECUTE_IF_OFF) {
            return true;
        }

        false
    }

    fn transition_to_level(&self, level: u8) {
        // todo implement with the transition time. For now we have a step change.

        info!("setting level to {}", level);
        self.current_level.set(Maybe::new(Some(level)));
    }
}

impl ClusterHandler for LevelControlHandler {
    #[doc = "The cluster-metadata corresponding to this handler trait."]
    const CLUSTER: rs_matter::data_model::objects::Cluster<'static> = FULL_CLUSTER
        .with_revision(1)
        .with_features(level_control::Feature::LIGHTING.bits() & level_control::Feature::ON_OFF.bits())
        .with_attrs(with!(
            AttributeId::CurrentLevel 
            | AttributeId::RemainingTime
            | AttributeId::OnLevel
            | AttributeId::MaxLevel
            | AttributeId::MinLevel
            | AttributeId::Options
            | AttributeId::StartUpCurrentLevel
        )) // todo add missing attributes needed for a dimmable light AttributeId::MinLevel
        .with_cmds(with!(
            CommandId::MoveToLevel
                | CommandId::Move
                | CommandId::Step
                | CommandId::Stop
                | CommandId::MoveToLevelWithOnOff
                | CommandId::MoveWithOnOff
                | CommandId::StepWithOnOff
                | CommandId::StopWithOnOff
        ));

    fn dataver(&self) -> u32 {
        self.dataver.get()
    }

    fn dataver_changed(&self) {
        self.dataver.changed();
    }

    fn current_level(
        &self,
        _ctx: &rs_matter::data_model::objects::ReadContext<'_>,
    ) -> Result<Nullable<u8>, Error> {
        info!("current_level called!");
        let val = self.current_level.take();
        self.current_level.set(val.clone());

        Ok(val)
    }

    fn options(
        &self,
        _ctx: &rs_matter::data_model::objects::ReadContext<'_>,
    ) -> Result<OptionsBitmap, Error> {
        info!("options called!");
        Ok(self.options)
    }

    fn on_level(
        &self,
        _ctx: &rs_matter::data_model::objects::ReadContext<'_>,
    ) -> Result<Nullable<u8>, Error> {
        info!("on_level called!");
        let val = self.on_level.take();
        self.current_level.set(val.clone());

        Ok(val)
    }

    fn set_options(
        &self,
        _ctx: &rs_matter::data_model::objects::WriteContext<'_>,
        value: OptionsBitmap,
    ) -> Result<(), Error> {
        warn!("set_options is not yet implemented.");
        Ok(())
    }

    fn set_on_level(
        &self,
        _ctx: &rs_matter::data_model::objects::WriteContext<'_>,
        value: Nullable<u8>,
    ) -> Result<(), Error> {
        info!("set_on_level called");
        self.on_level.set(value);
        Ok(())
    }

    fn remaining_time(&self, _ctx: &rs_matter::data_model::objects::ReadContext<'_>) -> Result<u16,rs_matter::error::Error> {
        info!("remaining_time called!");
        // todo this is a dummy return.
        Ok(0)
    }

    fn max_level(&self, _ctx: &rs_matter::data_model::objects::ReadContext<'_>) -> Result<u8,rs_matter::error::Error> {
        info!("max_level called!");
        Ok(Self::MAX_LEVEL)
    }

    fn min_level(&self,ctx: &rs_matter::data_model::objects::ReadContext<'_>) -> Result<u8,rs_matter::error::Error> {
        info!("min_level called!");
        Ok(Self::MIN_LEVEL)
    }

    fn start_up_current_level(&self,ctx: &rs_matter::data_model::objects::ReadContext<'_>) -> Result<rs_matter::tlv::Nullable<u8> ,rs_matter::error::Error> {
        info!("start_up_current_level called!");
        let val = self.startup_current_level.take();
        self.current_level.set(val.clone());

        Ok(val)
    }

    fn set_start_up_current_level(&self,ctx: &rs_matter::data_model::objects::WriteContext<'_> ,value:rs_matter::tlv::Nullable<u8>) -> Result<(),rs_matter::error::Error> {
        info!("set_start_up_current_level called!");
        self.startup_current_level.set(value);
        Ok(())
    }

    fn handle_move_to_level(
        &self,
        _ctx: &rs_matter::data_model::objects::InvokeContext<'_>,
        request: MoveToLevelRequest<'_>,
    ) -> Result<(), Error> {
        info!("handle_move_to_level called!");
        if !self.should_continue(request.options_mask()?, request.options_override()?) {
            // todo Should this return an error?
            debug!("Ignoring command due to options settings");
            return Ok(());
        }

        self.transition_to_level(request.level()?);

        Ok(())
    }

    fn handle_move(
        &self,
        _ctx: &rs_matter::data_model::objects::InvokeContext<'_>,
        request: MoveRequest<'_>,
    ) -> Result<(), Error> {
        info!("handle_move called!");
        if !self.should_continue(request.options_mask()?, request.options_override()?) {
            // todo Should this return an error?
            debug!("Ignoring command due to options settings");
            return Ok(());
        }

        let rate = request.rate()?.into_option();

        let rate = match rate {
            Some(0) | None => { return Err(Error::new(ErrorCode::InvalidCommand)); },
            Some(val) => val,
        };

        info!("moving with rate {}", rate);
        Ok(())
    }

    fn handle_step(
        &self,
        _ctx: &rs_matter::data_model::objects::InvokeContext<'_>,
        request: StepRequest<'_>,
    ) -> Result<(), Error> {
        info!("handle_step called!");
        if !self.should_continue(request.options_mask()?, request.options_override()?) {
            // todo Should this return an error?
            debug!("Ignoring command due to options settings");
            return Ok(());
        }

        Ok(())
    }

    fn handle_stop(
        &self,
        _ctx: &rs_matter::data_model::objects::InvokeContext<'_>,
        request: StopRequest<'_>,
    ) -> Result<(), Error> {
        info!("handle_stop called!");
        if !self.should_continue(request.options_mask()?, request.options_override()?) {
            // todo Should this return an error?
            debug!("Ignoring command due to options settings");
            return Ok(());
        }

        Ok(())
    }

    fn handle_move_to_level_with_on_off(
        &self,
        _ctx: &rs_matter::data_model::objects::InvokeContext<'_>,
        request: MoveToLevelWithOnOffRequest<'_>,
    ) -> Result<(), Error> {
        info!("handle_move_to_level_with_on_off called!");
        Ok(())
    }

    fn handle_move_with_on_off(
        &self,
        _ctx: &rs_matter::data_model::objects::InvokeContext<'_>,
        request: MoveWithOnOffRequest<'_>,
    ) -> Result<(), Error> {
        info!("handle_move_with_on_off called!");
        Ok(())
    }

    fn handle_step_with_on_off(
        &self,
        _ctx: &rs_matter::data_model::objects::InvokeContext<'_>,
        request: StepWithOnOffRequest<'_>,
    ) -> Result<(), Error> {
        info!("handle_step_with_on_off called!");
        Ok(())
    }

    fn handle_stop_with_on_off(
        &self,
        _ctx: &rs_matter::data_model::objects::InvokeContext<'_>,
        request: StopWithOnOffRequest<'_>,
    ) -> Result<(), Error> {
        info!("handle_stop_with_on_off called!");
        Ok(())
    }

    fn handle_move_to_closest_frequency(
        &self,
        _ctx: &rs_matter::data_model::objects::InvokeContext<'_>,
        request: MoveToClosestFrequencyRequest<'_>,
    ) -> Result<(), Error> {
        info!("handle_move_to_closest_frequency called!");
        Ok(())
    }
}
