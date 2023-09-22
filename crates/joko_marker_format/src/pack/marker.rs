use super::CommonAttributes;
use glam::Vec3;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub(crate) struct Marker {
    pub guid: Uuid,
    pub position: Vec3,
    pub map_id: u32,
    pub category: String,
    pub attrs: CommonAttributes,
}
