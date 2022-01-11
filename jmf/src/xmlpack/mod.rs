use serde::{Deserialize, Serialize};

use crate::xmlpack::{xml_category::XMLMarkerCategory, xml_marker::Behavior};

pub mod xml_category;
pub mod xml_file;
pub mod xml_marker;
pub mod xml_pack;
pub mod xml_pack_entry;
pub mod xml_route;
pub mod xml_trail;
pub mod rapid;
pub const MARKER_SCHEMA_XSD: &str = include_str!("xmlfile_schema.xsd");

/// the struct we use for inheritance from category/other markers.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MarkerTemplate {
    pub achievement_bit: Option<u32>,
    pub achievement_id: Option<u32>,
    pub alpha: Option<f32>,
    pub auto_trigger: Option<u8>,
    pub behavior: Option<Behavior>,
    pub color: Option<[u8; 4]>,
    pub fade_far: Option<i32>,
    pub fade_near: Option<i32>,
    pub has_countdown: Option<u8>,
    pub height_offset: Option<f32>,
    pub in_game_visibility: Option<u8>,
    pub icon_file: Option<String>,
    pub icon_size: Option<f32>,
    pub info: Option<String>,
    pub info_range: Option<f32>,
    pub keep_on_map_edge: Option<u8>,
    pub map_display_size: Option<u16>,
    pub map_fade_out_scale_level: Option<f32>,
    pub map_visibility: Option<u8>,
    pub max_size: Option<u16>,
    pub min_size: Option<u16>,
    pub mini_map_visibility: Option<u8>,
    pub reset_length: Option<u32>,
    pub scale_on_map_with_zoom: Option<u8>,
    pub toggle_cateogry: Option<String>,
    pub trigger_range: Option<f32>,
}

impl MarkerTemplate {
    pub fn new(mc: &XMLMarkerCategory) -> Self {
        Self {
            achievement_bit: mc.achievement_bit,
            achievement_id: mc.achievement_id,
            alpha: mc.alpha,
            auto_trigger: mc.auto_trigger,
            behavior: mc.behavior,
            color: mc.color,
            fade_far: mc.fade_far,
            fade_near: mc.fade_near,
            has_countdown: mc.has_countdown,
            height_offset: mc.height_offset,
            in_game_visibility: mc.in_game_visibility,
            icon_file: mc.icon_file.clone(),
            icon_size: mc.icon_size,
            info: mc.info.clone(),
            info_range: mc.info_range,
            keep_on_map_edge: mc.keep_on_map_edge,
            map_display_size: mc.map_display_size,
            map_fade_out_scale_level: mc.map_fade_out_scale_level,
            map_visibility: mc.map_visibility,
            max_size: mc.max_size,
            min_size: mc.min_size,
            mini_map_visibility: mc.mini_map_visibility,
            reset_length: mc.reset_length,
            scale_on_map_with_zoom: mc.scale_on_map_with_zoom,
            toggle_cateogry: mc.toggle_cateogry.clone(),
            trigger_range: mc.trigger_range,
        }
    }

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
