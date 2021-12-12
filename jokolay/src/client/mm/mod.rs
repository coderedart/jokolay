use jmf::jsonpack::{json_pack::ActivationData, SinglePack};
use jokotypes::{MapID, MarkerID, OSet, PackID, UOMap, UOSet};
use serde::{Deserialize, Serialize};

pub struct MarkerManager {
    pub packs: UOMap<PackID, SinglePack>,
    pub activation_data: UOMap<PackID, ActivationData>,
    pub current_map: Option<MapID>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DrawableMarkers {
    pub static_markers: UOSet<MarkerID>,
    pub dynamic_markers: UOSet<MarkerID>,
    pub sleeping_markers: UOSet<MarkerID>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct MarkerConfig {}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct MarkerData {
    pub ignore_packs: OSet<PackID>,
}
