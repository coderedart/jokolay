use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, PartialOrd)]
pub struct XMLRoute {
    #[serde(rename = "MapID")]
    pub map_id: u32,
    #[serde(rename = "Name")]
    pub name: String,
    #[serde(rename = "resetposx")]
    pub reset_pos_x: Option<f32>,
    #[serde(rename = "resetposy")]
    pub reset_pos_y: Option<f32>,
    #[serde(rename = "resetposz")]
    pub reset_pos_z: Option<f32>,

    #[serde(rename = "resetrange")]
    pub reset_range: Option<f32>,
}
