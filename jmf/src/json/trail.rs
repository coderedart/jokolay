use serde::{Deserialize, Serialize};

use crate::json::marker::Achievement;

#[serde_with::skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Trail {
    pub achievement: Option<Achievement>,
    pub alpha: Option<u8>,
    pub anim_speed: Option<f32>,
    pub cat: u16,
    pub color: Option<[u8; 4]>,
    pub fade_range: Option<[f32; 2]>,
    pub flags: super::marker::MarkerFlags,
    pub map_display_size: Option<u16>,
    pub map_fade_out_scale_level: Option<f32>,
    pub pos: [f32; 3],
    pub scale: Option<f32>,
    pub texture: Option<u16>,
    pub tbin: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TBinDescription {
    pub name: String,
    pub version: u8,
}
