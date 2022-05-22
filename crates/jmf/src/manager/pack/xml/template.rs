use elementtree::Element;
use serde::{Deserialize, Serialize};
use serde_with::*;

use std::num::ParseFloatError;
use tracing::warn;

macro_rules! inheritable {
  (
    $( #[$attr:meta] )*
    $vis:vis struct $name:ident {
      $( $( #[$field_attr:meta] )* $field_vis:vis $field:ident : $ty:ty ),* $(,)?
    }
  ) => {
    $( #[$attr] )*
    $vis struct $name {
      $( $( #[$field_attr] )* $vis $field : $ty ),*
    }
    impl $name {
      $vis fn inherit_if_prop_none(&mut self, other: &$name) {
        $( self.$field = self.$field.take().or(other.$field.clone()); )*
      }
    }
  }
}

inheritable!(
    /// the struct we use for inheritance from category/other markers.
    #[serde_with::serde_as]
    #[derive(Debug, Serialize, Deserialize, Clone, Default)]
    pub struct MarkerTemplate {
        /// An ID for an achievement from the GW2 API. Markers with the corresponding achievement ID will be hidden if the ID is marked as "done" for the API key that's entered in TacO.
        // achievement_id: Option<u16>,
        /// This is similar to achievementId, but works for partially completed achievements as well, if the achievement has "bits", they can be individually referenced with this.
        // achievement_bit: Option<u8>,

        /// How opaque the displayed icon should be. The default is 1.0
        alpha: Option<f32>,
        // anim_speed: Option<f32>,
        /// Determines if going near the marker triggers it
        // auto_trigger: Option<bool>,
        /// it describes the way the marker will behave when a player presses 'F' over it.
        // behavior: Option<u8>,
        // bounce_delay: Option<f32>,
        // bounce_duration: Option<f32>,
        // bounce_height: Option<f32>,
        /// hex value. The color tint of the marker. sRGBA8
        color: Option<[u8; 4]>,
        // copy: Option<String>,
        // copy_message: Option<String>,
        // cull: Option<String>,
        /// Determines how far the marker will completely disappear. If below 0, the marker won't disappear at any distance. Default is -1. FadeFar needs to be higher than fadeNear for sane results. This value is in game units (inches).
        // #[serde(rename = "fadeFar")]
        // fade_far: Option<i32>,
        /// Determines how far the marker will start to fade out. If below 0, the marker won't disappear at any distance. Default is -1. This value is in game units (inches).
        // #[serde(rename = "fadeNear")]
        // fade_near: Option<i32>,
        // festival: Option<Festivals>,
        /// Determines if a marker has a countdown timer display when triggered
        // has_countdown: Option<bool>,
        /// Specifies how high above the ground the marker is displayed. Default value is 1.5. in meters
        #[serde(rename = "heightOffset")]
        height_offset: Option<f32>,
        // hide: Option<String>,
        /// The icon to be displayed for the marker. If not given, this defaults to the image shown at the start of this article. This should point to a .png file. The overlay looks for the image files both starting from the root directory and the POIs directory for convenience. Make sure you don't use too high resolution (above 128x128) images because the texture atlas used for these is limited in size and it's a needless waste of resources to fill it quickly.Default value: 20
        #[serde(rename = "iconFile")]
        icon_file: Option<String>,
        /// The size of the icon in the game world. Default is 1.0 if this is not defined. Note that the "screen edges herd icons" option will limit the size of the displayed images for technical reasons.
        #[serde(rename = "iconSize")]
        icon_size: Option<f32>,
        /// if true, the marker/trails belonging to this category will show up in-game, like the markers you're used to. Default value: true.
        // #[serde(rename = "inGameVisibility")]
        // in_game_visibility: Option<bool>,

        /// his can be a multiline string, it will show up on screen as a text when the player is inside of infoRange of the marker
        // info: Option<String>,
        /// This determines how far away from the marker the info string will be visible. in meters.
        // info_range: Option<f32>,
        // invert_behavior: Option<bool>,
        // is_wall: Option<bool>,
        /// only affects markers, not trails. If true, markers belonging to this category will not disappear as they move out of the minimap's rectangle, but will be kept on the edge like the personal waypoint. Default value: false.
        // #[serde(rename = "keepOnMapEdge")]
        // keep_on_map_edge: Option<bool>,

        /// The size of the marker at normal UI scale, at zoom level 1 on the miniMap, in Pixels. For trails this value can be used to tweak the width
        // #[serde(rename = "mapDisplaySize")]
        // map_display_size: Option<u16>,
        // map_fade_out_scale_level: Option<f32>,
        /// if true, the marker/trails belonging to this category will show up on the main map. Default value: true.
        // #[serde(rename = "mapVisibility")]
        // map_visibility: Option<bool>,
        // map_type: Option<MapTypes>,
        /// Determines the maximum size of a marker on the screen, in pixels.
        // #[serde(rename = "maxSize")]
        // max_size: Option<u16>,
        /// Determines the minimum size of a marker on the screen, in pixels.
        // #[serde(rename = "minSize")]
        // min_size: Option<u16>,
        /// if true, the marker/trails belonging to this category will show up on the minimap. Default value: true.
        // #[serde(rename = "miniMapVisibility")]
        // mini_map_visibility: Option<bool>,
        // mount: Option<Mounts>,
        // profession: Option<Professions>,
        // race: Option<Races>,
        /// For behavior 4 this tells how long the marker should be invisible after pressing 'F'. For behavior 5 this will tell how long a map cycle is. in seconds.
        // #[serde(rename = "resetLength")]
        // reset_length: Option<u32>,
        /// this will supply data for behavior 5. The data will be given in seconds.
        // #[serde(rename = "resetOffset")]
        // reset_offset: Option<u32>,
        #[serde(rename = "rotate")]
        rotate: Option<[f32; 3]>,
        #[serde(rename = "rotate-x")]
        rotate_x: Option<f32>,
        #[serde(rename = "rotate-y")]
        rotate_y: Option<f32>,
        #[serde(rename = "rotate-z")]
        rotate_z: Option<f32>,
        /// if true, the markers/width of the trails belonging to this category will scale with the zoom level as you zoom in and out. Default value: true.
        // #[serde(rename = "scaleOnMapWithZoom")]
        // scale_on_map_with_zoom: Option<bool>,
        // show: Option<String>,
        // specialization: Option<Specializations>,
        texture: Option<String>,
        // tip_name: Option<String>,
        // tip_description: Option<String>,
        /// will toggle the specified category on or off when triggered with the action key. or with auto_trigger/trigger_range
        // #[serde(rename = "toggleCategory")]
        // toggle_cateogry: Option<String>,
        trail_data_file: Option<String>,
        trail_scale: Option<f32>,
        // Determines the range from where the marker is triggered. in meters.
        // trigger_range: Option<f32>,
    }
);

impl MarkerTemplate {
    pub fn inherit_from_template(&mut self, other: &MarkerTemplate) {
        self.inherit_if_prop_none(other);
    }
    pub fn update_from_element(&mut self, ele: &Element) {
        for (attr_name, attr_value) in ele.attrs() {
            let parsed_properly = match attr_name.name().trim().to_lowercase().as_str() {
                "alpha" => {
                    if let Ok(alpha) = attr_value.parse() {
                        self.alpha = Some(alpha);
                        true
                    } else {
                        false
                    }
                }
                "color" => {
                    let mut color = [0u8; 4];
                    if hex::decode_to_slice(attr_value, &mut color).is_ok() {
                        self.color = Some(color);
                        true
                    } else {
                        false
                    }
                }
                "heightoffset" => {
                    if let Ok(offset) = attr_value.parse() {
                        self.height_offset = Some(offset);
                        true
                    } else {
                        false
                    }
                }
                "iconfile" => {
                    self.icon_file = Some(attr_value.to_string());
                    true
                }
                "iconsize" => {
                    if let Ok(scale) = attr_value.parse() {
                        self.icon_size = Some(scale);
                        true
                    } else {
                        false
                    }
                }
                "rotate" => {
                    let floats: Result<Vec<f32>, ParseFloatError> =
                        attr_value.split(',').map(|value| value.parse()).collect();
                    if let Ok(floats) = floats {
                        let mut rotate = [0.0; 3];
                        rotate.copy_from_slice(&floats);
                        self.rotate = Some(rotate);
                        true
                    } else {
                        false
                    }
                }
                "rotate-x" => {
                    if let Ok(rotate) = attr_value.parse() {
                        self.rotate_x = Some(rotate);
                        true
                    } else {
                        false
                    }
                }
                "rotate-y" => {
                    if let Ok(rotate) = attr_value.parse() {
                        self.rotate_y = Some(rotate);
                        true
                    } else {
                        false
                    }
                }
                "rotate-z" => {
                    if let Ok(rotate) = attr_value.parse() {
                        self.rotate_z = Some(rotate);
                        true
                    } else {
                        false
                    }
                }
                "texture" => {
                    self.texture = Some(attr_value.to_string());
                    true
                }
                "trailfile" => {
                    self.trail_data_file = Some(attr_value.to_string());
                    true
                }
                "trailscale" => {
                    if let Ok(scale) = attr_value.parse() {
                        self.trail_scale = Some(scale);
                        true
                    } else {
                        false
                    }
                }
                _ => true,
            };
            if !parsed_properly {
                warn!(
                    "failed to properly parse attribute {} with value {}",
                    attr_name, attr_value
                );
            }
        }
    }
}
