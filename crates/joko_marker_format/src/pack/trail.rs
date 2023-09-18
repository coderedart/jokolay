use uuid::Uuid;

use super::CommonAttributes;

#[derive(Debug, Clone)]
pub(crate) struct Trail {
    pub guid: Uuid,
    pub map_id: u32,
    pub category: String,
    pub props: CommonAttributes,
}

#[derive(Debug, Clone)]
pub(crate) struct TBin {
    pub map_id: u32,
    pub version: u32,
    pub nodes: Vec<glam::Vec3>,
}

impl TBin {}
