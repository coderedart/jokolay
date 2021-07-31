

use serde::{Deserialize, Serialize};

use super::xml_marker::{Behavior, POIs};

/// Marker Category tag in xml files
/// acts as a template for markers to inherit from when there's a common property to all the markers under that category/subcatagories.
#[derive(Default, Debug, Clone, Serialize, Deserialize, PartialEq, PartialOrd)]
pub struct MarkerCategory {
    /// name will be how we merge/check for consistencies when they are declared in multiple Marker Files and we try to merge them all into a Category Selection Tree.
    pub name: String,
    /// this is what will be shown in the user facing menu when selecting to enable/disable this Category of markers to draw.
    #[serde(rename = "DisplayName")]
    pub display_name: String,
    /// used to mark a category as just for displaying a heading and doesn't need to be "interactable" as it doesn't have any markers
    #[serde(rename = "IsSeparator")]
    pub is_separator: Option<u32>,
    /// These are all the direct sub categories
    #[serde(rename = "MarkerCategory")]
    pub children: Option<Vec<MarkerCategory>>,
    /// from here on, the rest of the attributes are for marker inheritance and are documented in the POI Struct
    #[serde(rename = "mapDisplaySize")]
    pub map_display_size: Option<u32>,
    #[serde(rename = "iconFile")]
    pub icon_file: Option<String>,
    #[serde(rename = "iconSize")]
    pub icon_size: Option<f32>,
    pub alpha: Option<f32>,
    pub behavior: Option<Behavior>,
    #[serde(rename = "heightOffset")]
    pub height_offset: Option<f32>,
    #[serde(rename = "fadeNear")]
    pub fade_near: Option<u32>,
    #[serde(rename = "fadeFar")]
    pub fade_far: Option<u32>,
    #[serde(rename = "minSize")]
    pub min_size: Option<u32>,
    #[serde(rename = "maxSize")]
    pub max_size: Option<u32>,
    pub reset_length: Option<u32>,
    #[serde(default)]
    #[serde(with = "super::xml_marker::color")]
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

/// The root overlay tag in any valid xml file
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct OverlayData {
    #[serde(rename = "MarkerCategory")]
    pub categories: Option<MarkerCategory>,
    #[serde(rename = "POIs")]
    pub pois: Option<POIs>,
}

impl MarkerCategory {
    pub fn inherit_if_none(&mut self, other: &MarkerCategory) {
        self.name = other.name + "." + &self.name;
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

    // pub fn build_categories(
    //     mut prefix: String,
    //     mut cat: MarkerCategory,
    //     cat_map: &mut BTreeMap<String, MarkerCategory>,
    // ) {
    //     let previous_cat = cat_map.get(&prefix);
    //     let template;
    //     if previous_cat.is_some() {
    //         template = previous_cat.unwrap().clone();
    //         cat.inherit_if_none(&template);
    //     }
    //     if !prefix.is_empty() {
    //         prefix.push('.');
    //     }
    //     prefix += &cat.name;

    //     if cat.children.is_none() {
    //         cat_map.insert(prefix, cat);
    //     } else {
    //         let children = cat.children;
    //         cat.children = None;
    //         cat_map.insert(prefix.clone(), cat);
    //         if children.is_some() {
    //             for mc in children.unwrap() {
    //                 Self::build_categories(prefix.clone(), mc, cat_map);
    //             }
    //         }
    //     }
    // }
}

// #[cfg(test)]
// mod tests {
//     #[test]
//     fn mc_serde() {

//     }
// }
