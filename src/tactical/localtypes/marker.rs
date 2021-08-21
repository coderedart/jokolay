use std::collections::HashMap;

use crate::{
    fm::{FileManager, VID},
    tactical::{
        localtypes::{category::CategoryIndex,  IMCategory},
        xmltypes::{
            xml_category::XMLMarkerCategory,
            xml_marker::{Behavior, XMLPOI},
            xml_trail::XMLTrail,
        },
    },
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// A Marker struct for simpler representation and abstract away icon_file/type fields into indexes of global_cats/images etc..
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, PartialOrd)]
pub struct POI {
    /// position of the marker in world space.
    pub pos: [f32; 3],
    /// Describes which map the marker is located on.
    pub map_id: u32,
    /// Marker id
    pub guid: Uuid,
    /// size on minimap/map
    pub map_display_size: Option<u32>,
    /// index of icon_file to use as texture for this marker
    pub icon_file: Option<VID>,
    /// The size of the icon in the game world. Default is 1.0 if this is not defined. Note that the "screen edges herd icons" option will limit the size of the displayed images for technical reasons.
    pub icon_size: Option<f32>,
    /// How opaque the displayed icon should be. The default is 1.0
    pub alpha: Option<f32>,
    /// it describes the way the marker will behave when a player presses 'F' over it.
    pub behavior: Option<Behavior>,
    /// Determines how far the marker will start to fade out. If below 0, the marker won't disappear at any distance. Default is -1. This value is in game units (inches).
    pub fade_near: Option<u32>,
    /// Determines how far the marker will completely disappear. If below 0, the marker won't disappear at any distance. Default is -1. FadeFar needs to be higher than fadeNear for sane results. This value is in game units (inches).
    pub fade_far: Option<u32>,
    /// Determines the minimum size of a marker on the screen, in pixels.
    pub min_size: Option<u32>,
    /// Determines the maximum size of a marker on the screen, in pixels.
    pub max_size: Option<u32>,
    /// For behavior 4 this tells how long the marker should be invisible after pressing 'F'. For behavior 5 this will tell how long a map cycle is.
    pub reset_length: Option<u32>,
    pub color: Option<[u8; 4]>,
    /// Determines if going near the marker triggers it
    pub auto_trigger: Option<bool>,
    /// Determines if a marker has a countdown timer display when triggered
    pub has_countdown: Option<bool>,
    /// Determines the range from where the marker is triggered
    pub trigger_range: Option<f32>,
    /// An ID for an achievement from the GW2 API. Markers with the corresponding achievement ID will be hidden if the ID is marked as "done" for the API key that's entered in TacO.
    pub achievement_id: Option<u32>,
    /// This is similar to achievementId, but works for partially completed achievements as well, if the achievement has "bits", they can be individually referenced with this.
    pub achievement_bit: Option<u32>,
    /// his can be a multiline string, it will show up on screen as a text when the player is inside of infoRange of the marker
    pub info: Option<String>,
    /// This determines how far away from the marker the info string will be visible
    pub info_range: Option<f32>,
    pub map_visibility: Option<bool>,
    pub mini_map_visibility: Option<bool>,
    /// This text is used to display the type of the marker. It can contain spaces.
    pub category: CategoryIndex,
}
impl POI {
    pub fn from_xmlpoi(
        pack_path: VID,
        poi: &XMLPOI,
        global_cats: &Vec<IMCategory>,
        fm: &FileManager,
    ) -> Option<POI> {
        let pos = [poi.xpos, poi.ypos, poi.zpos];
        let category = global_cats
            .iter()
            .position(|c| c.full_name == poi.category)?;
        let category = CategoryIndex(category);
        let icon_path = poi.icon_file.clone();
        let icon_vid = if let Some(ipath) = icon_path {
            let pack_path = fm.get_path(pack_path).unwrap();
            let ipath = pack_path.join(&ipath).unwrap();
            if let Some(v) = fm.get_vid(&ipath) {
                Some(v)
            } else {
                log::error!(
                    "{:?}, {:?}, {:?}, {:?}, {:?}",
                    ipath,
                    pack_path,
                    poi.guid,
                    &poi.icon_file,
                    &poi.category
                );
                None
            }
        } else {
            None
        };

        Some(POI {
            pos,
            map_id: poi.map_id,
            guid: poi.guid,
            map_display_size: poi.map_display_size,
            icon_file: icon_vid,
            icon_size: poi.icon_size,
            alpha: poi.alpha,
            behavior: poi.behavior,
            fade_near: poi.fade_near,
            fade_far: poi.fade_far,
            min_size: poi.min_size,
            max_size: poi.max_size,
            reset_length: poi.reset_length,
            color: poi.color,
            auto_trigger: poi.auto_trigger,
            has_countdown: poi.has_countdown,
            trigger_range: poi.trigger_range,
            achievement_id: poi.achievement_id,
            achievement_bit: poi.achievement_bit,
            info: poi.info.clone(),
            info_range: poi.info_range,
            map_visibility: poi.map_visibility,
            mini_map_visibility: poi.mini_map_visibility,
            category,
        })
    }
}
/// the struct we use for inheritance from category/other markers.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MarkerTemplate {
    pub map_display_size: Option<u32>,
    pub icon_file: Option<VID>,
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
    pub fn inherit_from_marker_category(&mut self, other: &XMLMarkerCategory, icon_file: Option<VID>) {
        if self.map_display_size.is_none() {
            self.map_display_size = other.map_display_size;
        }
        if self.icon_file.is_none() {
            self.icon_file = icon_file;
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
        if let Some(c) = global_cats.get_mut(self.category.0) {
            c.poi_registry.push(self.guid);
        } else {
            log::error!(
                "marker with guid: {:?} cannot find category: {:?} to register",
                self.guid,
                self.category
            );
        }
    }
/// inserts poi from `pvec` into `all_pois` and returns the Uuids of those inserted pois as a Vec to keep the order
    /// This is to have all unique POI at one place and use an array of Uuid instead to refer to the contents
    pub fn get_vec_uuid_pois(pvec: Vec<XMLPOI>, all_pois: &mut HashMap<Uuid, POI>, pack_path: VID, global_cats: &mut Vec<IMCategory>, fm: &FileManager ) -> Vec<Uuid>{
        let mut uuid_vec = Vec::new();
        for xp in pvec {
            let id = xp.guid;
            if let Some(p) = POI::from_xmlpoi(pack_path, &xp, &global_cats, fm) {
                all_pois.entry(id).or_insert(p);
                uuid_vec.push(id);

            } 
        }
        uuid_vec
                            
    }
}

