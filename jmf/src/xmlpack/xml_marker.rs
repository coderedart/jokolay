use crate::xmlpack::MarkerTemplate;



use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};
use serde_with::{serde_as, skip_serializing_none};
use uuid::Uuid;
/// Markers format in the xml files are described under the <POIs> tag under the root <OverlayData> tag. The <POI> tag describes a marker.
/// everything is optional except xpos, ypos, zpos, mapId.
#[serde_as]
#[skip_serializing_none]
#[derive(Debug, Default, Clone, Serialize, Deserialize, PartialEq, PartialOrd)]
pub struct POI {
    /// position of the marker in world space.
    pub xpos: f32,
    pub ypos: f32,
    pub zpos: f32,
    /// Describes which map the marker is located on.
    #[serde(rename = "MapID")]
    pub map_id: u16,
    /// An ID for an achievement from the GW2 API. Markers with the corresponding achievement ID will be hidden if the ID is marked as "done" for the API key that's entered in TacO.
    pub achievement_id: Option<u16>,
    /// This is similar to achievementId, but works for partially completed achievements as well, if the achievement has "bits", they can be individually referenced with this.
    pub achievement_bit: Option<u8>,
    /// How opaque the displayed icon should be. The default is 1.0
    pub alpha: Option<f32>,
    /// Determines if going near the marker triggers it
    pub auto_trigger: Option<u8>,
    /// it describes the way the marker will behave when a player presses 'F' over it.
    pub behavior: Option<Behavior>,
    /// This text is used to display the type of the marker. It can contain spaces.
    #[serde(rename = "type")]
    pub category: String,
    /// hex value. The color tint of the marker
    #[serde_as(as = "Option<serde_with::hex::Hex>")]
    #[serde(default)]
    pub color: Option<[u8; 4]>,
    /// Determines how far the marker will completely disappear. If below 0, the marker won't disappear at any distance. Default is -1. FadeFar needs to be higher than fadeNear for sane results. This value is in game units (inches).
    #[serde(rename = "fadeFar")]
    pub fade_far: Option<i32>,
    /// Determines how far the marker will start to fade out. If below 0, the marker won't disappear at any distance. Default is -1. This value is in game units (inches).
    #[serde(rename = "fadeNear")]
    pub fade_near: Option<i32>,
    /// base64 encoded string of a UUID version 4 variant b, optional. This is a unique identifier for the marker used in tracking activation of markers through the activationdata.xml file. If this doesn't exist for a marker, one will be generated automatically and added on the next export.
    #[serde(with = "base64_uuid")]
    #[serde(rename = "GUID")]
    pub guid: Option<Uuid>,
    /// Determines if a marker has a countdown timer display when triggered
    pub has_countdown: Option<u8>,
    /// Specifies how high above the ground the marker is displayed. Default value is 1.5
    #[serde(rename = "heightOffset")]
    pub height_offset: Option<f32>,
    /// if true, the marker/trails belonging to this category will show up in-game, like the markers you're used to. Default value: true.
    #[serde(rename = "inGameVisibility")]
    pub in_game_visibility: Option<u8>,
    /// The icon to be displayed for the marker. If not given, this defaults to the image shown at the start of this article. This should point to a .png file. The overlay looks for the image files both starting from the root directory and the POIs directory for convenience. Make sure you don't use too high resolution (above 128x128) images because the texture atlas used for these is limited in size and it's a needless waste of resources to fill it quickly.Default value: 20
    #[serde(rename = "iconFile")]
    pub icon_file: Option<String>,
    /// The size of the icon in the game world. Default is 1.0 if this is not defined. Note that the "screen edges herd icons" option will limit the size of the displayed images for technical reasons.
    #[serde(rename = "iconSize")]
    pub icon_size: Option<f32>,
    /// only affects markers, not trails. If true, markers belonging to this category will not disappear as they move out of the minimap's rectangle, but will be kept on the edge like the personal waypoint. Default value: false.
    #[serde(rename = "keepOnMapEdge")]
    pub keep_on_map_edge: Option<u8>,
    /// his can be a multiline string, it will show up on screen as a text when the player is inside of infoRange of the marker
    pub info: Option<String>,
    /// This determines how far away from the marker the info string will be visible
    pub info_range: Option<f32>,
    /// The size of the marker at normal UI scale, at zoom level 1 on the miniMap, in Pixels. For trails this value can be used to tweak the width
    #[serde(rename = "mapDisplaySize")]
    pub map_display_size: Option<u16>,
    /// Zooming out farther than this value will result in the marker/trail fading out over the course of 2 zoom levels. Default value is 100, which effectively means no fading.
    #[serde(rename = "mapFadeoutScaleLevel")]
    pub map_fade_out_scale_level: Option<f32>,
    /// if true, the marker/trails belonging to this category will show up on the main map. Default value: true.
    #[serde(rename = "mapVisibility")]
    pub map_visibility: Option<u8>,
    /// Determines the maximum size of a marker on the screen, in pixels.
    #[serde(rename = "maxSize")]
    pub max_size: Option<u16>,
    /// Determines the minimum size of a marker on the screen, in pixels.
    #[serde(rename = "minSize")]
    pub min_size: Option<u16>,
    /// if true, the marker/trails belonging to this category will show up on the minimap. Default value: true.
    #[serde(rename = "miniMapVisibility")]
    pub mini_map_visibility: Option<u8>,
    /// For behavior 4 this tells how long the marker should be invisible after pressing 'F'. For behavior 5 this will tell how long a map cycle is.
    #[serde(rename = "resetLength")]
    pub reset_length: Option<u32>,
    /// if true, the markers/width of the trails belonging to this category will scale with the zoom level as you zoom in and out. Default value: true.
    #[serde(rename = "scaleOnMapWithZoom")]
    pub scale_on_map_with_zoom: Option<u8>,
    /// will toggle the specified category on or off when triggered with the action key. or with auto_trigger/trigger_range
    #[serde(rename = "toggleCategory")]
    pub toggle_cateogry: Option<String>,
    /// Determines the range from where the marker is triggered
    #[serde(rename = "triggerRange")]
    pub trigger_range: Option<f32>,
}

/**
behavior - integer. it describes the way the marker will behave when a player presses 'F' over it. The following values are valid for this parameter:
    0: the default value. Marker is always visible.
    1: 'Reappear on map change' - this is not implemented yet, it will be useful for markers that need to reappear if the player changes the map instance.
    2: 'Reappear on daily reset' - these markers disappear if the player presses 'F' over them, and reappear at the daily reset. These were used for the orphan markers during wintersday.
    3: 'Only visible before activation' - these markers disappear forever once the player pressed 'F' over them. Useful for collection style markers like golden lost badges, etc.
    4: 'Reappear after timer' - This behavior makes the marker reappear after a fix amount of time given in 'resetLength'.
    5: 'Reappear on map reset' - not implemented yet. This will make the marker reappear when the map cycles. In this case 'resetLength' will define the map cycle length in seconds, and 'resetOffset' will define when the first map cycle of the day begins after the daily reset, in seconds.
    6: 'Once per instance' - these markers disappear when triggered but reappear if you go into another instance of the map
    7: 'Once daily per character' - these markers disappear when triggered, but reappear with the daily reset, and can be triggered separately for every character

**/
#[derive(Serialize_repr, Deserialize_repr, PartialEq, Debug, Clone, Copy, PartialOrd, Eq, Ord)]
#[repr(u8)]
pub enum Behavior {
    AlwaysVisible = 0,
    ReappearOnMapChange = 1,
    ReappearOnDailyReset = 2,
    OnlyVisibleBeforeActivation = 3,
    ReappearAfterTimer = 4,
    ReappearOnMapReset = 5,
    OncePerInstance = 6,
    DailyPerChar = 7,
    OncePerInstancePerChar = 8,
    WvWObjective = 9,
}


impl POI {
    // pub fn from_json_marker(
    //     jm: crate::json::Marker,
    //     map_id: u16,
    //     cat: String,
    //     images_dir_name: &str,
    //     id_names_map: &UOMap<CategoryID, String>,
    // ) -> Self {
    //     let mut xm = Self {
    //         xpos: jm.position[0],
    //         ypos: jm.position[1],
    //         zpos: jm.position[2],
    //         map_id,
    //         alpha: jm.alpha.map(f) ,
    //         category: cat,
    //         color: jm.color,
    //         guid: Some(jm.id.into()),
    //         in_game_visibility: jm.in_game_visibility,
    //         icon_file: jm
    //             .image
    //             .map(|hash| format!("{}{}.png", images_dir_name, hash)),
    //         icon_size: jm.scale,
    //         keep_on_map_edge: jm.keep_on_map_edge,
    //         map_display_size: jm.map_display_size,
    //         map_fade_out_scale_level: jm.map_fade_out_scale_level,
    //         map_visibility: jm.map_visibility,
    //         max_size: jm.max_size,
    //         min_size: jm.min_size,
    //         mini_map_visibility: jm.mini_map_visibility,
    //         scale_on_map_with_zoom: jm.scale_on_map_with_zoom,

    //         ..Default::default()
    //     };

    //     if let Some(ac) = jm.achievement {
    //         xm.achievement_id = Some(ac.id);
    //         xm.achievement_bit = ac.bit;
    //     }
    //     if let Some(d) = jm.dynamic_props {
    //         if let Some(t) = d.trigger {
    //             xm.auto_trigger = t.auto_trigger;
    //             xm.trigger_range = Some(t.range);
    //             xm.toggle_cateogry = t.toggle_cat.map(|id| {
    //                 id_names_map
    //                     .get(&id)
    //                     .expect("failed to get full name from id of cat")
    //                     .clone()
    //             });
    //             if let Some(b) = t.behavior {
    //                 match b {
    //                     crate::json::json_marker::Behavior::AlwaysVisible => {
    //                         xm.behavior = Some(Behavior::AlwaysVisible)
    //                     }
    //                     crate::json::json_marker::Behavior::ReappearOnMapChange => {
    //                         xm.behavior = Some(Behavior::ReappearOnMapChange)
    //                     }
    //                     crate::json::json_marker::Behavior::ReappearOnDailyReset => {
    //                         xm.behavior = Some(Behavior::ReappearOnDailyReset)
    //                     }
    //                     crate::json::json_marker::Behavior::OnlyVisibleBeforeActivation => {
    //                         xm.behavior = Some(Behavior::OnlyVisibleBeforeActivation)
    //                     }
    //                     crate::json::json_marker::Behavior::ReappearAfterTimer {
    //                         reset_length,
    //                     } => {
    //                         xm.behavior = Some(Behavior::ReappearAfterTimer);
    //                         xm.reset_length = Some(reset_length);
    //                     }
    //                     crate::json::json_marker::Behavior::ReappearOnMapReset {
    //                         map_cycle_length: _,
    //                         map_cycle_offset_after_reset: _,
    //                     } => {
    //                         xm.behavior = Some(Behavior::ReappearOnMapReset);
    //                         // unimplemented!("attributes map_cycle_length and map_reset_offset not yet implemented");
    //                     }
    //                     crate::json::json_marker::Behavior::OncePerInstance => {
    //                         xm.behavior = Some(Behavior::OncePerInstance)
    //                     }
    //                     crate::json::json_marker::Behavior::DailyPerChar => {
    //                         xm.behavior = Some(Behavior::DailyPerChar)
    //                     }
    //                     crate::json::json_marker::Behavior::OncePerInstancePerChar => {
    //                         xm.behavior = Some(Behavior::OncePerInstancePerChar)
    //                     }
    //                     crate::json::json_marker::Behavior::WvWObjective => {
    //                         xm.behavior = Some(Behavior::WvWObjective)
    //                     }
    //                 }
    //             }
    //             xm.has_countdown = t.count_down;
    //         }
    //         if let Some(info) = d.info {
    //             xm.info = Some(info.text);
    //             xm.info_range = Some(info.range);
    //         }
    //     }
    //     if let Some(fade_range) = jm.fade_range {
    //         xm.fade_near = Some(fade_range[0] as i32);
    //         xm.fade_far = Some(fade_range[1] as i32);
    //     }

    //     xm
    // }
    pub fn inherit_if_none(&mut self, other: &MarkerTemplate) {
        if self.map_display_size.is_none() {
            self.map_display_size = other.map_display_size;
        }
        if self.icon_file.is_none() {
            self.icon_file = other.icon_file.clone();
        }
        if self.icon_size.is_none() {
            self.icon_size = other.icon_size;
        }
        if self.alpha.is_none() {
            self.alpha = other.alpha;
        }
        if self.behavior.is_none() {
            self.behavior = other.behavior;
        }
        if self.height_offset.is_none() {
            self.height_offset = other.height_offset;
        }
        if self.fade_near.is_none() {
            self.fade_near = other.fade_near;
        }
        if self.fade_far.is_none() {
            self.fade_far = other.fade_far;
        }
        if self.min_size.is_none() {
            self.min_size = other.min_size;
        }
        if self.max_size.is_none() {
            self.max_size = other.max_size;
        }
        if self.reset_length.is_none() {
            self.reset_length = other.reset_length;
        }
        if self.color.is_none() {
            self.color = other.color;
        }
        if self.auto_trigger.is_none() {
            self.auto_trigger = other.auto_trigger;
        }
        if self.has_countdown.is_none() {
            self.has_countdown = other.has_countdown;
        }
        if self.trigger_range.is_none() {
            self.trigger_range = other.trigger_range;
        }
        if self.achievement_id.is_none() {
            self.achievement_id = other.achievement_id;
        }
        if self.achievement_bit.is_none() {
            self.achievement_bit = other.achievement_bit;
        }
        if self.info.is_none() {
            self.info = other.info.clone();
        }
        if self.info_range.is_none() {
            self.info_range = other.info_range;
        }
        if self.map_visibility.is_none() {
            self.map_visibility = other.map_visibility;
        }
        if self.mini_map_visibility.is_none() {
            self.mini_map_visibility = other.mini_map_visibility;
        }
        if self.scale_on_map_with_zoom.is_none() {
            self.scale_on_map_with_zoom = other.scale_on_map_with_zoom;
        }
        if self.keep_on_map_edge.is_none() {
            self.keep_on_map_edge = other.keep_on_map_edge;
        }
        if self.toggle_cateogry.is_none() {
            self.toggle_cateogry = other.toggle_cateogry.clone();
        }
        if self.map_fade_out_scale_level.is_none() {
            self.map_fade_out_scale_level = other.map_fade_out_scale_level;
        }
        if self.in_game_visibility.is_none() {
            self.in_game_visibility = other.in_game_visibility;
        }
        // if self.map_display_size.is_none() {
        //     self.map_display_size = other.map_display_size;
        // }
        // if self.icon_file.is_none() {
        //     self.icon_file = other.icon_file.clone();
        // }
        // if self.icon_size.is_none() {
        //     self.icon_size = other.icon_size;
        // }
        // if self.alpha.is_none() {
        //     self.alpha = other.alpha;
        // }
        // if self.behavior.is_none() {
        //     self.behavior = other.behavior;
        // }
        // if self.height_offset.is_none() {
        //     self.height_offset = other.height_offset;
        // }
        // if self.fade_near.is_none() {
        //     self.fade_near = other.fade_near;
        // }
        // if self.fade_far.is_none() {
        //     self.fade_far = other.fade_far;
        // }
        // if self.min_size.is_none() {
        //     self.min_size = other.min_size;
        // }
        // if self.max_size.is_none() {
        //     self.max_size = other.max_size;
        // }
        // if self.reset_length.is_none() {
        //     self.reset_length = other.max_size;
        // }
        // if self.color.is_none() {
        //     self.color = other.color.clone();
        // }
        // if self.auto_trigger.is_none() {
        //     self.auto_trigger = other.auto_trigger;
        // }
        // if self.has_countdown.is_none() {
        //     self.has_countdown = other.has_countdown;
        // }
        // if self.trigger_range.is_none() {
        //     self.trigger_range = other.trigger_range;
        // }
        // if self.achievement_id.is_none() {
        //     self.achievement_id = other.achievement_id;
        // }
        // if self.achievement_bit.is_none() {
        //     self.achievement_bit = other.achievement_bit;
        // }
        // if self.info.is_none() {
        //     self.info = other.info.clone();
        // }
        // if self.info_range.is_none() {
        //     self.info_range = other.info_range;
        // }
    }
}
/// Serde functions for the hex based color strings in POI/MarkerCategory to reduce to a [u8; 4]
// pub mod color {
//     use serde::{Deserializer, Serializer};

//     pub fn serialize<S>(c: &Option<[u8; 4]>, serializer: S) -> Result<S::Ok, S::Error>
//     where
//         S: Serializer,
//     {
//         if let Some(c) = c {
//             hex::serialize(c, serializer)
//         } else {
//             serializer.serialize_none()
//         }
//     }

//     pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<[u8; 4]>, D::Error>
//     where
//         D: Deserializer<'de>,
//     {
//         let result: [u8; 4] = hex::deserialize(deserializer)?;
//         Ok(Some(result))
//     }
// }
/// serde function to convert taco's GUID which is base64 encoded, back to a Uuid for easier working with Uuid Crate instead of using the slower strings comparision
pub mod base64_uuid {

    use serde::{Deserialize, Deserializer, Serializer};
    use uuid::Uuid;

    pub fn serialize<S>(c: &Option<Uuid>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        if let Some(c) = c {
            let bytes = c.as_bytes().as_ref();
            serializer.serialize_str(&base64::encode(bytes))
        } else {
            serializer.serialize_none()
        }
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<Uuid>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let b64_str = String::deserialize(deserializer)?;
        let result = base64::decode(&b64_str)
            .map_err(|e| {
                log::warn!(
                    "failed to parse uuid from decoded base64 due to error: {:?}. string: '{}'",
                    &e,
                    &b64_str
                )
            })
            .unwrap_or_default();
        Ok(Uuid::from_slice(&result).ok())
    }
    // pub fn check_base64_uuid<'de, D>(deserializer: D) -> Result<Option<Uuid>, D::Error>
    // where
    //     D: Deserializer<'de>,
    // {
    //     let id = String::deserialize(deserializer)?;
    //     if let Ok(base64_id) = base64::decode(&id) {
    //         Ok(Uuid::from_slice(&base64_id)
    //             .map_err(|e| {
    //                 log::warn!(
    //                     "failed to parse uuid from decoded base64 due to error: {:?}. string: {}",
    //                     &e,
    //                     &id
    //                 );
    //                 e
    //             })
    //             .ok())
    //     } else {
    //         Ok(None)
    //     }
    // }
}
