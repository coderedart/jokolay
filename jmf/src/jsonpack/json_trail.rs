use serde::{Deserialize, Serialize};
use serde_with::serde_as;

use jokotypes::*;

#[serde_with::skip_serializing_none]
#[serde_with::serde_as]
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Trail {
    pub id: TrailID,
    pub tbin: TrailHash,
    pub alpha: Option<f32>,
    pub anim_speed: Option<f32>,
    pub color: Option<[u8; 4]>,
    pub fade_range: Option<[f32; 2]>,
    pub image: Option<ImageHash>,
    pub scale: Option<f32>,
}

#[serde_as]
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TBinDescription {
    pub name: String,
    pub map_id: MapID,
}
