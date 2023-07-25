use base64::Engine;
use glam::Vec3;
use uuid::Uuid;
use xot::Element;

use super::{CommonAttributes, XotAttributeNameIDs};

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

    pub fn serialize_to_element(&self, ele: &mut Element, names: &XotAttributeNameIDs) {
        ele.set_attribute(names.xpos, format!("{}", self.position[0]));
        ele.set_attribute(names.ypos, format!("{}", self.position[1]));
        ele.set_attribute(names.zpos, format!("{}", self.position[2]));
        ele.set_attribute(names.guid, super::PackCore::BASE64_ENGINE.encode(self.guid));
        ele.set_attribute(names.map_id, format!("{}", self.map_id));
        ele.set_attribute(names.category, &self.category);
        self.props.serialize_to_element(ele, names);
    }
}
