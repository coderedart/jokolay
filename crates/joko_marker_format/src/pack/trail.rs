use joko_core::prelude::Vec3A;
use uuid::Uuid;

use super::CommonAttributes;

#[derive(Debug)]
pub struct Trail {
    pub guid: Uuid,
    pub category: String,
    pub props: CommonAttributes,
}

#[derive(Debug)]
pub struct TBin {
    pub map_id: u32,
    pub version: u32,
    pub nodes: Vec<Vec3A>,
}

impl TBin {}
