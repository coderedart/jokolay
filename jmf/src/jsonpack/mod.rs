pub mod json_cat;
pub mod json_marker;
pub mod json_pack;
pub mod json_trail;

use crate::jsonpack::json_pack::{JsonPack, PackData};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SinglePack {
    pub pack: JsonPack,
    pub pack_data: PackData,
}
