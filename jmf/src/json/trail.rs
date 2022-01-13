use serde::{Deserialize, Serialize};



#[serde_with::skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Trail {
    pub pos: [f32; 3],
    pub tbin: u16,
    pub alpha: Option<u8>,
    pub anim_speed: Option<f32>,
    pub color: Option<[u8; 4]>,
    pub fade_range: Option<[f32; 2]>,
    pub texture: Option<u16>,
    pub scale: Option<f32>,
}



#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TBinDescription {
    pub name: String,
    pub version: u8,
}

