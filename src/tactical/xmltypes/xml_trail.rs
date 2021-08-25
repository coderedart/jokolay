use serde::{Deserialize, Serialize};
use uuid::Uuid;
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
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, PartialOrd)]
pub struct XMLTrail {
    #[serde(rename = "type")]
    pub category: String,
    #[serde(default)]
    #[serde(deserialize_with = "super::xml_marker::check_base64_uuid")]
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
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(with = "super::xml_marker::color")]
    pub color: Option<[u8; 4]>,
    pub alpha: Option<f32>,
    #[serde(rename = "fadeFar")]
    pub fade_near: Option<u32>,
    #[serde(rename = "fadeFar")]
    pub fade_far: Option<u32>,
}
