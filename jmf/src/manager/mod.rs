
// use crate::json::{pack::ActivationData, SinglePack};
use jokotypes::{UOMap, MapID, PackID};
use serde::{Deserialize, Serialize};

use crate::json::FullPack;

pub struct MarkerManager {
    pub packs: UOMap<PackID, FullPack>,
}




impl MarkerManager {
    pub fn new() {

    }

}
// #[derive(Debug, Clone, Serialize, Deserialize, Default)]
// #[serde(default)]
// pub struct MarkerConfig {}

