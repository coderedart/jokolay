use std::collections::HashMap;

// use crate::json::{pack::ActivationData, SinglePack};

use crate::json::FullPack;

pub struct MarkerManager {
    pub packs: HashMap<u16, FullPack>,
}

// #[derive(Debug, Clone, Serialize, Deserialize, Default)]
// #[serde(default)]
// pub struct MarkerConfig {}
