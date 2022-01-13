use serde::{Deserialize, Serialize};
use serde_with::{serde_as, skip_serializing_none};
use uuid::Uuid;

// use super::xml_marker::PoiOrTrail;
use crate::xmlpack::MarkerTemplate;
/**
In order to get an exported trail to show up in TacO, it needs to be added to a marker pack just like a marker.
Trails are described by the <Trail> tag and uses the same category system as the markers.
Trail tag is usually under <POIs> tag under <OverlayData> tag.
If you put a marker and a trail in the same category, the user can hide them both at the same time by hiding the category.
Here's an example trail:

<OverlayData>
 <POIs>
  <Trail trailData="Trails/poobadoo.trl" texture="data/Bounty.png" color="ffffffff" animSpeed="1" alpha="1" type="tactical.guildmission.bounty.poobadoo" fadeNear="3000" fadeFar="4000"/>
 </POIs>
</OverlayData>

The color, type, alpha, fadeNear and fadeFar attributes function the same as they do for markers.
The trailData tag needs to point to a binary trail. These are the files that you get by exporting them during a recording session. The binary trails also contain the map they were recorded on, so the MapID tag is ignored for trails.
The texture tag points to the texture that should scroll on the trail.
The animSpeed tag is a float value that modifies the speed of the animation on a trail.
There's also a trailScale tag that is a float value that modifies how stretched the texture will look on the trail.
**/
#[serde_as]
#[skip_serializing_none]
#[derive(Debug, Default, Clone, Serialize, Deserialize, PartialEq, PartialOrd)]
pub struct Trail {
    #[serde(rename = "type")]
    pub category: String,
    #[serde(default)]
    #[serde(with = "super::xml_marker::base64_uuid")]
    #[serde(rename = "GUID")]
    pub guid: Option<Uuid>,
    #[serde(rename = "trailData")]
    pub trail_data_file: String,
    pub texture: Option<String>,
    #[serde(rename = "animSpeed")]
    pub anim_speed: Option<f32>,
    #[serde(rename = "trailScale")]
    pub trail_scale: Option<f32>,
    #[serde(default)]
    #[serde_as(as = "Option<serde_with::hex::Hex>")]
    pub color: Option<[u8; 4]>,
    pub alpha: Option<f32>,
    #[serde(rename = "fadeNear")]
    pub fade_near: Option<i32>,
    #[serde(rename = "fadeFar")]
    pub fade_far: Option<i32>,
}

// impl From<PoiOrTrail> for Trail {
//     fn from(trail_enum: PoiOrTrail) -> Self {
//         let mut t = Self::default();
//         match trail_enum {
//             PoiOrTrail::Trail {
//                 category,
//                 guid,
//                 trail_data_file,
//                 texture,
//                 anim_speed,
//                 trail_scale,
//                 color,
//                 alpha,
//                 fade_near,
//                 fade_far,
//             } => {
//                 t.category = category;
//                 t.guid = guid;
//                 t.trail_data_file = trail_data_file;
//                 t.texture = texture;
//                 t.anim_speed = anim_speed;
//                 t.trail_scale = trail_scale;
//                 t.color = color;
//                 t.alpha = alpha;
//                 t.fade_near = fade_near;
//                 t.fade_far = fade_far;
//             }
//             _ => unimplemented!(),
//         }
//         t
//     }
// }
// impl From<&PoiOrTrail> for Trail {
//     fn from(trail_enum: &PoiOrTrail) -> Self {
//         trail_enum.clone().into()
//     }
// }
// impl From<Trail> for PoiOrTrail {
//     fn from(t: Trail) -> Self {
//         PoiOrTrail::Trail {
//             category: t.category,
//             guid: t.guid,
//             trail_data_file: t.trail_data_file,
//             texture: t.texture,
//             anim_speed: t.anim_speed,
//             trail_scale: t.trail_scale,
//             color: t.color,
//             alpha: t.alpha,
//             fade_near: t.fade_near,
//             fade_far: t.fade_far,
//         }
//     }
// }
/// The trail data struct holding the nodes
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct TrailData {
    pub version: u32,
    pub map_id: u32,
    pub nodes: Vec<[f32; 3]>,
}
impl TrailData {
    pub fn parse_from_bytes(
        trail_binary_data: &[u8],
    ) -> Result<TrailData, TrailDataDeserializeError> {
        if trail_binary_data.len() <= 8 {
            return Err(TrailDataDeserializeError::FileTooSmall {
                size: trail_binary_data.len() as u32,
            });
        }
        let mut version_bytes = [0_u8; 4];
        version_bytes.copy_from_slice(&trail_binary_data[..4]);
        let mut version = u32::from_ne_bytes(version_bytes);
        if version < 2 {
            version = 2;
        }
        if version != 2 {
            return Err(TrailDataDeserializeError::InvalidVersion { version });
        }
        let mut map_id_bytes = [0_u8; 4];
        map_id_bytes.copy_from_slice(&trail_binary_data[4..8]);
        let map_id = u32::from_ne_bytes(map_id_bytes);

        let nodes: &[f32] = bytemuck::cast_slice(&trail_binary_data[8..]);
        let nodes: Vec<[f32; 3]> = nodes
            .chunks_exact(3)
            .map(|pos| [pos[0], pos[1], pos[2]])
            .collect();
        Ok(TrailData {
            map_id,
            version,
            nodes,
        })
    }
}

#[derive(Debug, thiserror::Error)]
pub enum TrailDataDeserializeError {
    #[error("File Size too small. size: {size}")]
    FileTooSmall { size: u32 },
    #[error("invalid version: {version}")]
    InvalidVersion { version: u32 },
}
impl Trail {
    // pub fn from_json_trail(
    //     jt: crate::json::trail::Trail,
    //     cat: String,
    //     images_dir_name: &str,
    //     trails_dir_name: &str,
    // ) -> Self {
    //     let mut xt = Self {
    //         alpha: jt.alpha,
    //         anim_speed: jt.anim_speed,
    //         color: jt.color,
    //         category: cat,
    //         guid: Some(jt.id.into()),
    //         trail_scale: jt.scale,
    //         texture: jt
    //             .image
    //             .map(|hash| format!("{}{}.png", images_dir_name, &hash)),
    //         trail_data_file: format!("{}{}.trl", trails_dir_name, jt.tbin),
    //         ..Default::default()
    //     };
    //     if let Some(fade_range) = jt.fade_range {
    //         xt.fade_far = Some(fade_range[1] as i32);
    //         xt.fade_near = Some(fade_range[0] as i32);
    //     }
    //     xt
    // }
    pub fn inherit_if_none(&mut self, other: &MarkerTemplate) {
        if self.texture.is_none() {
            self.texture = other.icon_file.clone();
        }
        if self.trail_scale.is_none() {
            self.trail_scale = other.icon_size;
        }
        if self.alpha.is_none() {
            self.alpha = other.alpha;
        }

        if self.fade_near.is_none() {
            self.fade_near = other.fade_near;
        }
        if self.fade_far.is_none() {
            self.fade_far = other.fade_far;
        }

        if self.color.is_none() {
            self.color = other.color;
        }
    }
}
