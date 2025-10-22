use core::cell::Cell;
use palette::white_point::D65;
use log::{info, warn};

use rs_matter_embassy::matter::dm::{Dataver, ReadContext, WriteContext, InvokeContext, Cluster};
use rs_matter_embassy::matter::with;
use rs_matter_embassy::matter::error::{Error, ErrorCode};
use rs_matter_embassy::matter::tlv::Nullable;
use rs_matter_embassy::matter::dm::clusters::level_control::OptionsBitmap;

use crate::data_model::clusters::color_control::*;
pub use crate::data_model::clusters::color_control::ClusterHandler;

pub struct ColorControlCluster<T: ColorControlHooks> {
    dataver: Dataver,
    handler: T,
    current_x: Cell<u16>,
    current_y: Cell<u16>,
    color_mode: ColorMode,
    options: OptionsBitmap,
    number_of_primes: u8,
    primary_1_x: u16,
    primary_1_y: u16,
    primary_1_intensity: u8,
    primary_2_x: u16,
    primary_2_y: u16,
    primary_2_intensity: u8,
    primary_3_x: u16,
    primary_3_y: u16,
    primary_3_intensity: u8,
    // enhanced_color_mode: , // todo EnhancedColorModeEnum is not defined.
    // color_capabilities: ColorCapabilitiesBitmap,
    remaining_time: u16,
    color_temperature_mireds: u16,
    color_temp_physical_max_mireds: u16,
    color_temp_physical_min_mireds: u16,
    couple_color_temp_to_level_min_mireds: u16,
    start_up_color_temperature_mireds: u16,
}

impl<T: ColorControlHooks> ColorControlCluster<T> {
    pub fn new(dataver: Dataver, handler: T) -> Self {
        Self {
            dataver,
            handler,
            current_x: Cell::new(39518), // white
            current_y: Cell::new(21233),
            color_mode: ColorMode::CurrentXAndCurrentY,
            options: OptionsBitmap::empty(),
            number_of_primes: 3,
            primary_1_x: 0,
            primary_1_y: 0,
            primary_1_intensity: 0,
            primary_2_x: 0,
            primary_2_y: 0,
            primary_2_intensity: 0,
            primary_3_x: 0,
            primary_3_y: 0,
            primary_3_intensity: 0,
            remaining_time: 0,
            color_temperature_mireds: 0,
            color_temp_physical_max_mireds: 0,
            color_temp_physical_min_mireds: 0,
            couple_color_temp_to_level_min_mireds: 0,
            start_up_color_temperature_mireds: 0,
        }
    }

    /// Adapt the handler instance to the generic `rs-matter` `Handler` trait
    pub const fn adapt(self) -> HandlerAdaptor<Self> {
        HandlerAdaptor(self)
    }
}

impl<T: ColorControlHooks> ClusterHandler for ColorControlCluster<T> {
    #[doc = "The cluster-metadata corresponding to this handler trait."]
    const CLUSTER:Cluster<'static> = FULL_CLUSTER
        .with_revision(7)
        .with_features(Feature::XY.bits() | Feature::COLOR_TEMPERATURE.bits())
        .with_attrs(with!(
            required;
            AttributeId::CurrentX
            | AttributeId::CurrentY
            | AttributeId::ColorMode
            | AttributeId::Options
            | AttributeId::NumberOfPrimaries
            | AttributeId::Primary1X
            | AttributeId::Primary1Y
            | AttributeId::Primary1Intensity
            | AttributeId::Primary2X
            | AttributeId::Primary2Y
            | AttributeId::Primary2Intensity
            | AttributeId::Primary3X
            | AttributeId::Primary3Y
            | AttributeId::Primary3Intensity
            | AttributeId::EnhancedColorMode
            | AttributeId::ColorCapabilities
            | AttributeId::RemainingTime
            | AttributeId::ColorTemperatureMireds
            | AttributeId::ColorTempPhysicalMaxMireds
            | AttributeId::ColorTempPhysicalMinMireds
            | AttributeId::CoupleColorTempToLevelMinMireds
            | AttributeId::StartUpColorTemperatureMireds
        ))
        .with_cmds(with!(
            CommandId::MoveToColor
            | CommandId::MoveColor
            | CommandId::StepColor
            | CommandId::StopMoveStep
            | CommandId::MoveColorTemperature
            | CommandId::StepColorTemperature
        ));

    fn dataver(&self) -> u32 {
        self.dataver.get()
    }

    fn dataver_changed(&self) {
        self.dataver.changed();
    }

    fn current_x(&self, _ctx: impl ReadContext) -> Result<u16, Error> {
        info!("ColorControl: Called current_x()");
        Ok(self.current_x.get())
    }

    fn current_y(&self, _ctx: impl ReadContext) -> Result<u16, Error> {
        info!("ColorControl: Called current_y()");
        Ok(self.current_y.get())
    }

    fn primary_1_x(&self, _ctx: impl ReadContext) -> Result<u16, Error> {
        info!("ColorControl: Called primary_1_x()");
        Ok(self.primary_1_x)
    }

    fn primary_1_y(&self, _ctx: impl ReadContext) -> Result<u16, Error> {
        info!("ColorControl: Called primary_1_y()");
        Ok(self.primary_1_y)
    }

    fn primary_1_intensity(&self, _ctx: impl ReadContext) -> Result<Nullable<u8> , Error> {
        info!("ColorControl: Called primary_1_intensity()");
        Ok(Nullable::some(self.primary_1_intensity))
    }

    fn primary_2_x(&self, _ctx: impl ReadContext) -> Result<u16, Error> {
        info!("ColorControl: Called primary_2_x()");
        Ok(self.primary_2_x)
    }

    fn primary_2_y(&self, _ctx: impl ReadContext) -> Result<u16, Error> {
        info!("ColorControl: Called primary_2_y()");
        Ok(self.primary_2_y)
    }

    fn primary_2_intensity(&self, _ctx: impl ReadContext) -> Result<Nullable<u8> , Error> {
        info!("ColorControl: Called primary_2_intensity()");
        Ok(Nullable::some(self.primary_2_intensity))
    }

    fn primary_3_x(&self, _ctx: impl ReadContext) -> Result<u16, Error> {
        info!("ColorControl: Called primary_3_x()");
        Ok(self.primary_3_x)
    }

    fn primary_3_y(&self, _ctx: impl ReadContext) -> Result<u16, Error> {
        info!("ColorControl: Called primary_3_y()");
        Ok(self.primary_3_y)
    }

    fn primary_3_intensity(&self, _ctx: impl ReadContext) -> Result<Nullable<u8> , Error> {
        info!("ColorControl: Called primary_3_intensity()");
        Ok(Nullable::some(self.primary_3_intensity))
    }

    fn remaining_time(&self, _ctx: impl ReadContext) -> Result<u16, Error> {
        info!("ColorControl: Called remaining_time()");
        Ok(self.remaining_time)
    }

    fn color_temperature_mireds(&self, _ctx: impl ReadContext) -> Result<u16, Error> {
        info!("ColorControl: Called color_temperature_mireds()");
        Ok(self.color_temperature_mireds)
    }

    fn color_temp_physical_max_mireds(&self, _ctx: impl ReadContext) -> Result<u16, Error> {
        info!("ColorControl: Called color_temp_physical_max_mireds()");
        Ok(self.color_temp_physical_max_mireds)
    }

    fn color_temp_physical_min_mireds(&self, _ctx: impl ReadContext) -> Result<u16, Error> {
        info!("ColorControl: Called color_temp_physical_min_mireds()");
        Ok(self.color_temp_physical_min_mireds)
    }

    fn couple_color_temp_to_level_min_mireds(&self, _ctx: impl ReadContext) -> Result<u16, Error> {
        info!("ColorControl: Called couple_color_temp_to_level_min_mireds()");
        Ok(self.couple_color_temp_to_level_min_mireds)
    }

    fn start_up_color_temperature_mireds(&self, _ctx: impl ReadContext) -> Result<Nullable<u16> , Error> {
        info!("ColorControl: Called start_up_color_temperature_mireds()");
        Ok(Nullable::some(self.start_up_color_temperature_mireds))
    }

    fn color_mode(&self, _ctx: impl ReadContext) -> Result<u8, Error>  {
        info!("ColorControl: Called color_mode()");
        Ok(self.color_mode as u8)
    }

    fn options(&self, _ctx: impl ReadContext) -> Result<u8, Error>  {
        info!("ColorControl: Called options()");
        Ok(self.options.bits() as u8)
    }

    fn number_of_primaries(&self, _ctx: impl ReadContext) -> Result<Nullable<u8> , Error>  {
        info!("ColorControl: Called number_of_primaries()");
        Ok(Nullable::some(self.number_of_primes))
    }

    fn enhanced_color_mode(&self, _ctx: impl ReadContext) -> Result<u8, Error>  {
        info!("ColorControl: Called enhanced_color_mode()");
        Ok(1) // todo needs fixing when enhanced color mode bitmap is included
    }

    fn color_capabilities(&self, _ctx: impl ReadContext) -> Result<u16, Error>  {
        info!("ColorControl: Called color_capabilities()");
        Ok(ColorCapabilities::XY_ATTRIBUTES_SUPPORTED.bits() | ColorCapabilities::COLOR_TEMPERATURE_SUPPORTED.bits())
    }

    fn set_options(&self, _ctx: impl WriteContext, _value:u8) -> Result<(), Error>  {
        info!("ColorControl: Called set_options()");
        // todo is `&self` correct? We should be able to modify self if we want to set a value. 
        warn!("Not yet implemented. Doing nothing.");
        Ok(())
    }

    fn handle_move_to_hue(&self, _ctx: impl InvokeContext , _request:MoveToHueRequest<'_> ,) -> Result<(), Error>  {
        info!("ColorControl: Called handle_move_to_hue()");
        Err(ErrorCode::InvalidCommand.into())
    }

    fn handle_move_hue(&self, _ctx: impl InvokeContext , _request:MoveHueRequest<'_> ,) -> Result<(), Error>  {
        info!("ColorControl: Called handle_move_hue()");
        Err(ErrorCode::InvalidCommand.into())
    }

    fn handle_step_hue(&self, _ctx: impl InvokeContext , _request:StepHueRequest<'_> ,) -> Result<(), Error>  {
        info!("ColorControl: Called handle_step_hue()");
        Err(ErrorCode::InvalidCommand.into())
    }

    fn handle_move_to_saturation(&self, _ctx: impl InvokeContext , _request:MoveToSaturationRequest<'_> ,) -> Result<(), Error>  {
        info!("ColorControl: Called handle_move_to_saturation()");
        Err(ErrorCode::InvalidCommand.into())
    }

    fn handle_move_saturation(&self, _ctx: impl InvokeContext , _request:MoveSaturationRequest<'_> ,) -> Result<(), Error>  {
        info!("ColorControl: Called handle_move_saturation()");
        Err(ErrorCode::InvalidCommand.into())
    }

    fn handle_step_saturation(&self, _ctx: impl InvokeContext , _request:StepSaturationRequest<'_> ,) -> Result<(), Error>  {
        info!("ColorControl: Called handle_step_saturation()");
        Err(ErrorCode::InvalidCommand.into())
    }

    fn handle_move_to_hue_and_saturation(&self, _ctx: impl InvokeContext , _request:MoveToHueAndSaturationRequest<'_> ,) -> Result<(), Error>  {
        info!("ColorControl: Called handle_move_to_hue_and_saturation()");
        Err(ErrorCode::InvalidCommand.into())
    }

    fn handle_move_to_color(&self, _ctx: impl InvokeContext , request:MoveToColorRequest<'_> ,) -> Result<(), Error>  {
        info!("ColorControl: Called handle_move_to_color()");
        // todo process options
        self.handler.set_color(request.color_x()?, request.color_y()?)?;

        self.current_x.set(request.color_x()?);
        self.current_y.set(request.color_y()?);
        Ok(())
    }

    fn handle_move_color(&self, _ctx: impl InvokeContext , _request:MoveColorRequest<'_> ,) -> Result<(), Error>  {
        info!("ColorControl: Called handle_move_color()");
        warn!("Not yet implemented. Doing nothing.");
        Ok(())
    }

    fn handle_step_color(&self, _ctx: impl InvokeContext , _request:StepColorRequest<'_> ,) -> Result<(), Error>  {
        info!("ColorControl: Called handle_step_color()");
        warn!("Not yet implemented. Doing nothing.");
        Ok(())
    }

    fn handle_move_to_color_temperature(&self, _ctx: impl InvokeContext , _request:MoveToColorTemperatureRequest<'_> ,) -> Result<(), Error>  {
        info!("ColorControl: Called handle_move_to_color_temperature()");
        warn!("Not yet implemented. Doing nothing.");
        Ok(())
    }

    fn handle_enhanced_move_to_hue(&self, _ctx: impl InvokeContext , _request:EnhancedMoveToHueRequest<'_> ,) -> Result<(), Error>  {
        info!("ColorControl: Called handle_enhanced_move_to_hue()");
        Err(ErrorCode::InvalidCommand.into())
    }

    fn handle_enhanced_move_hue(&self, _ctx: impl InvokeContext , _request:EnhancedMoveHueRequest<'_> ,) -> Result<(), Error>  {
        info!("ColorControl: Called handle_enhanced_move_hue()");
        Err(ErrorCode::InvalidCommand.into())
    }

    fn handle_enhanced_step_hue(&self, _ctx: impl InvokeContext , _request:EnhancedStepHueRequest<'_> ,) -> Result<(), Error>  {
        info!("ColorControl: Called handle_enhanced_step_hue()");
        Err(ErrorCode::InvalidCommand.into())
    }

    fn handle_enhanced_move_to_hue_and_saturation(&self, _ctx: impl InvokeContext , _request:EnhancedMoveToHueAndSaturationRequest<'_> ,) -> Result<(), Error>  {
        info!("ColorControl: Called handle_enhanced_move_to_hue_and_saturation()");
        Err(ErrorCode::InvalidCommand.into())
    }

    fn handle_color_loop_set(&self, _ctx: impl InvokeContext , _request:ColorLoopSetRequest<'_> ,) -> Result<(), Error>  {
        info!("ColorControl: Called handle_color_loop_set()");
        Err(ErrorCode::InvalidCommand.into())
    }

    fn handle_stop_move_step(&self, _ctx: impl InvokeContext , _request:StopMoveStepRequest<'_> ,) -> Result<(), Error>  {
        info!("ColorControl: Called handle_stop_move_step()");
        warn!("Not yet implemented. Doing nothing.");
        Ok(())
    }

    fn handle_move_color_temperature(&self, _ctx: impl InvokeContext , _request:MoveColorTemperatureRequest<'_> ,) -> Result<(), Error>  {
        info!("ColorControl: Called handle_move_color_temperature()");
        warn!("Not yet implemented. Doing nothing.");
        Ok(())
    }

    fn handle_step_color_temperature(&self, _ctx: impl InvokeContext , _request:StepColorTemperatureRequest<'_> ,) -> Result<(), Error>  {
        info!("ColorControl: Called handle_step_color_temperature()");
        warn!("Not yet implemented. Doing nothing.");
        Ok(())
    }
}

pub trait ColorControlHooks {
    // todo add the transition time
    fn set_color(&self, x: u16, y: u16) -> Result<(), Error>;
}

// todo move to a separate file

use palette::{FromColor, Srgb, Yxy};
use crate::led::led::{LedSender, ControlMessage};

pub struct ColorControlHandler<'a> {
    sender: LedSender<'a>,
}

impl<'a> ColorControlHandler<'a> {
    pub fn new(sender: LedSender<'a>) -> Self {
        Self {
            sender,
        }
    }
}

impl<'a> ColorControlHooks for ColorControlHandler<'a> {
    fn set_color(&self, x: u16, y: u16) -> Result<(), Error> {
        let x_f32 = x as f32 / 65536.0;
        let y_f32 = y as f32 / 65536.0;
        
        let yxy: Yxy<D65, f32> = Yxy::new(x_f32, y_f32, 1.0);

        let srgb: Srgb<f32> = Srgb::from_color(yxy);

        let r = (srgb.red * 255.0) as u8;
        let g = (srgb.green * 255.0) as u8;
        let b = (srgb.blue * 255.0) as u8;

        self.sender.try_send(ControlMessage::SetColour { r: r, g: g, b: b }).map_err(|_| ErrorCode::Busy.into())
    }
}