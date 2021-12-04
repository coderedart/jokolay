use crate::jsonpack::json_pack::{JsonPack, PackData};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SinglePack {
    pub pack: JsonPack,
    pub pack_data: PackData,
}

impl SinglePack {}

#[derive(Debug, thiserror::Error)]
pub enum MultiJsonPackError {
    #[error("io errors")]
    IOError(#[from] tokio::io::Error),
    #[error("deserialization error")]
    DeSerError(#[from] serde_json::Error),
    #[error("image decode errors")]
    ImageDecodeError(#[from] image::ImageError),
}
