use super::CommonAttributes;
use joko_core::prelude::*;
use uuid::Uuid;

#[derive(Debug)]
pub struct Marker {
    pub guid: Uuid,
    pub position: Vec3,
    pub map_id: u32,
    pub category: String,
    pub props: CommonAttributes,
}
impl Marker {
    pub fn new(guid: Uuid, map_id: u32) -> Self {
        Self {
            guid,
            position: Default::default(),
            map_id,
            category: Default::default(),
            props: Default::default(),
        }
    }
}
