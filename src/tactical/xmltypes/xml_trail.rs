use serde::{Deserialize, Serialize};

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
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Trail {
    #[serde(rename = "type")]
    pub category: Option<String>,
    pub map_id: Option<u32>,
    pub guid: Option<String>,
    pub trail_data: Option<String>,
    pub texture: Option<String>,
    pub anim_speed: Option<f32>,
    pub trail_scale: Option<f32>,
    pub color: Option<u32>,
    pub alpha: Option<f32>,
    pub fade_near: Option<u32>,
    pub fade_far: Option<u32>,
}
