use jmf::jsonpack::{json_pack::ActivationData, SinglePack};
use serde::{Deserialize, Serialize};

pub struct MarkerManager {}

pub struct MarkerPack {
    pub pack: SinglePack,
    pub activation_data: ActivationData,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct MarkerConfig {}
