use std::collections::BTreeMap;

use super::marker::Behavior;
use super::marker::POIS;
use serde::{Deserialize, Serialize};
// pub struct Category {
//     name: String,
//     display_name: String,
//     marker_template: Option<Marker>,
// }

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct MarkerCategory {
    pub name: String,
    #[serde(rename = "DisplayName")]
    pub display_name: String,
    #[serde(rename = "IsSeparator")]
    pub is_separator: Option<u32>,
    #[serde(rename = "MarkerCategory")]
    pub children: Option<Vec<MarkerCategory>>,
    #[serde(rename = "mapDisplaySize")]
    pub map_display_size: Option<u32>,
    #[serde(rename = "iconFile")]
    pub icon_file: Option<String>,
    #[serde(rename = "iconSize")]
    pub icon_size: Option<f32>,
    // How opaque the displayed icon should be. The default is 1.0
    pub alpha: Option<f32>,
    // it describes the way the marker will behave when a player presses 'F' over it.
    pub behavior: Option<Behavior>,
    #[serde(rename = "heightOffset")]
    pub height_offset: Option<f32>,
    #[serde(rename = "fadeNear")]
    pub fade_near: Option<u32>,
    #[serde(rename = "fadeFar")]
    pub fade_far: Option<u32>,
    #[serde(rename = "minSize")]
    pub min_size: Option<u32>,
    // Determines the maximum size of a marker on the screen, in pixels.
    #[serde(rename = "maxSize")]
    pub max_size: Option<u32>,
    // For behavior 4 this tells how long the marker should be invisible after pressing 'F'. For behavior 5 this will tell how long a map cycle is.
    pub reset_length: Option<u32>,
    // hex value. The color tint of the marker
    pub color: Option<u32>,
    // Determines if going near the marker triggers it
    pub auto_trigger: Option<bool>,
    // Determines if a marker has a countdown timer display when triggered
    pub has_countdown: Option<bool>,
    // Determines the range from where the marker is triggered
    pub trigger_range: Option<f32>,
    // An ID for an achievement from the GW2 API. Markers with the corresponding achievement ID will be hidden if the ID is marked as "done" for the API key that's entered in TacO.
    pub achievement_id: Option<u32>,
    // This is similar to achievementId, but works for partially completed achievements as well, if the achievement has "bits", they can be individually referenced with this.
    pub achievement_bit: Option<u32>,
    // his can be a multiline string, it will show up on screen as a text when the player is inside of infoRange of the marker
    pub info: Option<String>,
    // This determines how far away from the marker the info string will be visible
    pub info_range: Option<f32>,
    pub map_visibility: Option<bool>,
    pub mini_map_visibility: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OverlayData {
    #[serde(rename = "MarkerCategory")]
    pub categories: MarkerCategory,
    #[serde(rename = "POIs")]
    pub pois: Option<POIS>,
}
impl MarkerCategory {
    pub fn inherit_if_none(&mut self, other: &MarkerCategory) {
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
    }

    pub fn build_categories(
        mut prefix: String,
        mut cat: MarkerCategory,
        cat_map: &mut BTreeMap<String, MarkerCategory>,
    ) {
        let previous_cat = cat_map.get(&prefix);
        let template;
        if previous_cat.is_some() {
            template = previous_cat.unwrap().clone();
            cat.inherit_if_none(&template);
        }
        if !prefix.is_empty() {
            prefix.push('.');
        }
        prefix += &cat.name;

        if cat.children.is_none() {
            cat_map.insert(prefix, cat);
        } else {
            let children = cat.children;
            cat.children = None;
            cat_map.insert(prefix.clone(), cat);
            if children.is_some() {
                for mc in children.unwrap() {
                    Self::build_categories(prefix.clone(), mc, cat_map);
                }
            }
        }
    }
}

// #[cfg(test)]
// mod tests {
//     #[test]
//     fn mc_serde() {

//     }
// }
