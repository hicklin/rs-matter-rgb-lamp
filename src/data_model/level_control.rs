use rs_matter::{with, data_model::basic_info::{ClusterHandler, FULL_CLUSTER, CapabilityMinimaStructBuilder}, import};

import!(LevelControl);


pub struct LevelControlHandler {

}

impl LevelControlHandler {
    fn new() -> LevelControlHandler {
        LevelControlHandler{}
    }
}

impl ClusterHandler for LevelControlHandler {
    #[doc = "The cluster-metadata corresponding to this handler trait."]
    const CLUSTER: rs_matter::data_model::objects::Cluster<'static> = FULL_CLUSTER
        .with_revision(1)
        .with_attrs(with!(required))
        .with_cmds(with!(
            level_control::CommandId::MoveToLevel | 
            level_control::CommandId::Move | 
            level_control::CommandId::Step | 
            level_control::CommandId::Stop | 
            level_control::CommandId::MoveToLevelWithOnOff | 
            level_control::CommandId::MoveWithOnOff | 
            level_control::CommandId::StepWithOnOff | 
            level_control::CommandId::StopWithOnOff | 
            level_control::CommandId::MoveToClosestFrequency));

    fn dataver(&self) -> u32 {
        todo!()
    }

    fn dataver_changed(&self) {
        todo!()
    }

    fn data_model_revision(&self, ctx: &rs_matter::data_model::objects::ReadContext<'_>) -> Result<u16,rs_matter::error::Error>  {
        todo!()
    }

    fn vendor_name<P:rs_matter::tlv::TLVBuilderParent>(&self, ctx: &rs_matter::data_model::objects::ReadContext<'_> , builder: rs_matter::tlv::Utf8StrBuilder<P>) -> Result<P,rs_matter::error::Error>  {
        todo!()
    }

    fn vendor_id(&self, ctx: &rs_matter::data_model::objects::ReadContext<'_>) -> Result<u16,rs_matter::error::Error>  {
        todo!()
    }

    fn product_name<P:rs_matter::tlv::TLVBuilderParent>(&self, ctx: &rs_matter::data_model::objects::ReadContext<'_> , builder: rs_matter::tlv::Utf8StrBuilder<P>) -> Result<P,rs_matter::error::Error>  {
        todo!()
    }

    fn product_id(&self, ctx: &rs_matter::data_model::objects::ReadContext<'_>) -> Result<u16,rs_matter::error::Error>  {
        todo!()
    }

    fn node_label<P:rs_matter::tlv::TLVBuilderParent>(&self, ctx: &rs_matter::data_model::objects::ReadContext<'_> , builder: rs_matter::tlv::Utf8StrBuilder<P>) -> Result<P,rs_matter::error::Error>  {
        todo!()
    }

    fn location<P:rs_matter::tlv::TLVBuilderParent>(&self, ctx: &rs_matter::data_model::objects::ReadContext<'_> , builder: rs_matter::tlv::Utf8StrBuilder<P>) -> Result<P,rs_matter::error::Error>  {
        todo!()
    }

    fn hardware_version(&self, ctx: &rs_matter::data_model::objects::ReadContext<'_>) -> Result<u16,rs_matter::error::Error>  {
        todo!()
    }

    fn hardware_version_string<P:rs_matter::tlv::TLVBuilderParent>(&self, ctx: &rs_matter::data_model::objects::ReadContext<'_> , builder: rs_matter::tlv::Utf8StrBuilder<P>) -> Result<P,rs_matter::error::Error>  {
        todo!()
    }

    fn software_version(&self, ctx: &rs_matter::data_model::objects::ReadContext<'_>) -> Result<u32,rs_matter::error::Error>  {
        todo!()
    }

    fn software_version_string<P:rs_matter::tlv::TLVBuilderParent>(&self, ctx: &rs_matter::data_model::objects::ReadContext<'_> , builder: rs_matter::tlv::Utf8StrBuilder<P>) -> Result<P,rs_matter::error::Error>  {
        todo!()
    }

    fn capability_minima<P:rs_matter::tlv::TLVBuilderParent>(&self, ctx: &rs_matter::data_model::objects::ReadContext<'_> , builder: CapabilityMinimaStructBuilder<P>) -> Result<P,rs_matter::error::Error>  {
        todo!()
    }

    fn specification_version(&self, ctx: &rs_matter::data_model::objects::ReadContext<'_>) -> Result<u32,rs_matter::error::Error>  {
        todo!()
    }

    fn max_paths_per_invoke(&self, ctx: &rs_matter::data_model::objects::ReadContext<'_>) -> Result<u16,rs_matter::error::Error>  {
        todo!()
    }

    fn set_node_label(&self, ctx: &rs_matter::data_model::objects::WriteContext<'_> ,value:rs_matter::tlv::Utf8Str<'_>) -> Result<(),rs_matter::error::Error>  {
        todo!()
    }

    fn set_location(&self, ctx: &rs_matter::data_model::objects::WriteContext<'_> ,value:rs_matter::tlv::Utf8Str<'_>) -> Result<(),rs_matter::error::Error>  {
        todo!()
    }

    fn handle_mfg_specific_ping(&self, ctx: &rs_matter::data_model::objects::InvokeContext<'_> ,) -> Result<(),rs_matter::error::Error>  {
        todo!()
    }
}