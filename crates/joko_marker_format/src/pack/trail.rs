use base64::Engine;
use glam::Vec3A;
use uuid::Uuid;
use xot::Element;

use super::{CommonAttributes, PackError, XotAttributeNameIDs};

#[derive(Debug)]
pub struct Trail {
    pub guid: Uuid,
    pub category: String,
    pub props: CommonAttributes,
}

impl Trail {
    pub fn serialize_to_element(&self, ele: &mut Element, names: &XotAttributeNameIDs) {
        ele.set_attribute(names.guid, super::PackCore::BASE64_ENGINE.encode(self.guid));
        ele.set_attribute(names.category, &self.category);
        self.props.serialize_to_element(ele, &names);
    }
}
#[derive(Debug)]
pub struct TBin {
    pub map_id: u32,
    pub version: u32,
    pub nodes: Vec<Vec3A>,
}

impl TBin {
    pub fn parse_from_slice(entry_contents: &[u8]) -> Result<Self, PackError> {
        let content_length = entry_contents.len();
        // content_length must be atleast 8 to contain version + map_id
        // and the remaining length must be a multiple of 12 bytes (size of vec3) to be valid series of position nodes
        if content_length < 8 || ((content_length - 8) % 12) != 0 {
            return Err(PackError::TBinInvalid);
        }

        let mut version_bytes = [0_u8; 4];
        version_bytes.copy_from_slice(&entry_contents[4..8]);
        let version = u32::from_ne_bytes(version_bytes);
        let mut map_id_bytes = [0_u8; 4];
        map_id_bytes.copy_from_slice(&entry_contents[4..8]);
        let map_id = u32::from_ne_bytes(map_id_bytes);

        // because we already checked before that the len of the slice is divisible by 12
        // this will either be empty vec or series of vec3s.
        let nodes: Vec<Vec3A> = entry_contents[8..]
            .chunks_exact(12)
            .map(|float_bytes| {
                // make [f32 ;3] out of those 12 bytes
                let arr = [
                    f32::from_le_bytes([
                        // first float
                        float_bytes[0],
                        float_bytes[1],
                        float_bytes[2],
                        float_bytes[3],
                    ]),
                    f32::from_le_bytes([
                        // second float
                        float_bytes[4],
                        float_bytes[5],
                        float_bytes[6],
                        float_bytes[7],
                    ]),
                    f32::from_le_bytes([
                        // third float
                        float_bytes[8],
                        float_bytes[9],
                        float_bytes[10],
                        float_bytes[11],
                    ]),
                ];

                Vec3A::from_array(arr)
            })
            .collect();
        Ok(TBin {
            map_id,
            version,
            nodes,
        })
    }
}
