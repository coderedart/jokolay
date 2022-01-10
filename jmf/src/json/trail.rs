use serde::{Deserialize, Serialize};
use serde_with::serde_as;

use jokotypes::{TBinID, TrailID, ImageID};
use validator::Validate;

#[serde_with::skip_serializing_none]
#[serde_with::serde_as]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Trail {
    pub id: TrailID,
    pub pos: [f32; 3],
    pub tbin: TBinID,
    pub alpha: Option<u8>,
    pub anim_speed: Option<f32>,
    pub color: Option<rgb::RGBA8>,
    pub fade_range: Option<[f32; 2]>,
    pub image: Option<ImageID>,
    pub scale: Option<f32>,
}



#[serde_as]
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct TBinDescription {
    #[validate(length(min = 1))]
    pub name: String,
}

