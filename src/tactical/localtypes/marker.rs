use crate::tactical::{
    localtypes::IMCategory,
    xmltypes::{
        xml_category::MarkerCategory,
        xml_marker::{Behavior, POI},
    },
};
use serde::{Deserialize, Serialize};
/// the struct we use for inheritance from category/other markers. 
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MarkerTemplate {
    pub map_display_size: Option<u32>,
    pub icon_file: Option<String>,
    pub icon_size: Option<f32>,
    pub alpha: Option<f32>,
    pub behavior: Option<Behavior>,
    pub height_offset: Option<f32>,
    pub fade_near: Option<u32>,
    pub fade_far: Option<u32>,
    pub min_size: Option<u32>,
    pub max_size: Option<u32>,
    pub reset_length: Option<u32>,
    pub color: Option<[u8; 4]>,
    pub auto_trigger: Option<bool>,
    pub has_countdown: Option<bool>,
    pub trigger_range: Option<f32>,
    pub achievement_id: Option<u32>,
    pub achievement_bit: Option<u32>,
    pub info: Option<String>,
    pub info_range: Option<f32>,
    pub map_visibility: Option<bool>,
    pub mini_map_visibility: Option<bool>,
}

impl MarkerTemplate {
    pub fn inherit_from_marker_category(&mut self, other: &MarkerCategory) {
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
            self.reset_length = other.max_size;
        }
        if self.color.is_none() {
            self.color = other.color.clone();
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
            self.reset_length = other.max_size;
        }
        if self.color.is_none() {
            self.color = other.color.clone();
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
    }
}

impl POI {
    pub fn register_category(&self, global_cats: &mut Vec<IMCategory>) {
        let cat = global_cats
            .iter_mut()
            .find(|c| &c.full_name == &self.category);
        if let Some(c) = cat {
            c.poi_registry.push(self.guid);
        } else {
            log::error!(
                "marker with guid: {:?} cannot find category: {:?} to register",
                self.guid,
                self.category
            );
        }
    }
}
