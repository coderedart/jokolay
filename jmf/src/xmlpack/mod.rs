use serde::{Deserialize, Serialize};

use crate::xmlpack::{xml_marker::Behavior};

pub mod xml_category;
pub mod xml_file;
pub mod xml_marker;
pub mod xml_pack;
pub mod xml_pack_entry;
pub mod xml_route;
pub mod xml_trail;
pub mod load;
pub mod rapid;
pub const MARKER_SCHEMA_XSD: &str = include_str!("xmlfile_schema.xsd");

// /// the struct we use for inheritance from category/other markers.
#[serde_with::serde_as]
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MarkerTemplate {
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
    /// this will supply data for behavior 5. The data will be given in seconds.
    #[serde(rename = "resetOffset")]
    pub reset_offset: Option<u32>,
    /// if true, the markers/width of the trails belonging to this category will scale with the zoom level as you zoom in and out. Default value: true.
    #[serde(rename = "scaleOnMapWithZoom")]
    pub scale_on_map_with_zoom: Option<u8>,
    /// will toggle the specified category on or off when triggered with the action key. or with auto_trigger/trigger_range
    #[serde(rename = "toggleCategory")]
    pub toggle_cateogry: Option<String>,
    /// Determines the range from where the marker is triggered
    pub trigger_range: Option<f32>,
}

impl MarkerTemplate {
//     pub fn new(mc: &XMLMarkerCategory) -> Self {
//         Self {
//             achievement_bit: mc.achievement_bit,
//             achievement_id: mc.achievement_id,
//             alpha: mc.alpha,
//             auto_trigger: mc.auto_trigger,
//             behavior: mc.behavior,
//             color: mc.color,
//             fade_far: mc.fade_far,
//             fade_near: mc.fade_near,
//             has_countdown: mc.has_countdown,
//             height_offset: mc.height_offset,
//             in_game_visibility: mc.in_game_visibility,
//             icon_file: mc.icon_file.clone(),
//             icon_size: mc.icon_size,
//             info: mc.info.clone(),
//             info_range: mc.info_range,
//             keep_on_map_edge: mc.keep_on_map_edge,
//             map_display_size: mc.map_display_size,
//             map_fade_out_scale_level: mc.map_fade_out_scale_level,
//             map_visibility: mc.map_visibility,
//             max_size: mc.max_size,
//             min_size: mc.min_size,
//             mini_map_visibility: mc.mini_map_visibility,
//             reset_length: mc.reset_length,
//             scale_on_map_with_zoom: mc.scale_on_map_with_zoom,
//             toggle_cateogry: mc.toggle_cateogry.clone(),
//             trigger_range: mc.trigger_range,
//         }
//     }

    pub fn inherit_from_template(&mut self, other: &MarkerTemplate) {
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
        if self.reset_offset.is_none() {
            self.reset_offset = other.reset_offset;
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
    }
}
