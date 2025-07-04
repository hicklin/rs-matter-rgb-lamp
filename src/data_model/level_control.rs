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
}

impl LevelControlHandler {
    pub fn new(dataver: Dataver) -> Self {
        Self {
            dataver,
            on: Cell::new(false),
            options: OptionsBitmap::from_bits(level_control::Feature::LIGHTING.bits() as u8)
                .unwrap(),
            on_level: Cell::new(Nullable::new(Some(1))),
            current_level: Cell::new(Nullable::new(Some(0))),
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
        .with_attrs(with!(required)) // todo add missing attributes needed for a dimmable light AttributeId::MinLevel
        .with_cmds(with!(
            CommandId::MoveToLevel
                | CommandId::Move
                | CommandId::Step
                | CommandId::Stop
                | CommandId::MoveToLevelWithOnOff
                | CommandId::MoveWithOnOff
                | CommandId::StepWithOnOff
                | CommandId::StopWithOnOff
                | CommandId::MoveToClosestFrequency
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
        let val = self.current_level.take();
        self.current_level.set(val.clone());

        Ok(val)
    }

    fn options(
        &self,
        _ctx: &rs_matter::data_model::objects::ReadContext<'_>,
    ) -> Result<OptionsBitmap, Error> {
        Ok(self.options)
    }

    fn on_level(
        &self,
        _ctx: &rs_matter::data_model::objects::ReadContext<'_>,
    ) -> Result<Nullable<u8>, Error> {
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
        self.on_level.set(value);
        Ok(())
    }

    fn handle_move_to_level(
        &self,
        _ctx: &rs_matter::data_model::objects::InvokeContext<'_>,
        request: MoveToLevelRequest<'_>,
    ) -> Result<(), Error> {
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
        Ok(())
    }

    fn handle_move_with_on_off(
        &self,
        _ctx: &rs_matter::data_model::objects::InvokeContext<'_>,
        request: MoveWithOnOffRequest<'_>,
    ) -> Result<(), Error> {
        Ok(())
    }

    fn handle_step_with_on_off(
        &self,
        _ctx: &rs_matter::data_model::objects::InvokeContext<'_>,
        request: StepWithOnOffRequest<'_>,
    ) -> Result<(), Error> {
        Ok(())
    }

    fn handle_stop_with_on_off(
        &self,
        _ctx: &rs_matter::data_model::objects::InvokeContext<'_>,
        request: StopWithOnOffRequest<'_>,
    ) -> Result<(), Error> {
        Ok(())
    }

    fn handle_move_to_closest_frequency(
        &self,
        _ctx: &rs_matter::data_model::objects::InvokeContext<'_>,
        request: MoveToClosestFrequencyRequest<'_>,
    ) -> Result<(), Error> {
        Ok(())
    }
}
