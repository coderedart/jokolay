use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};

use super::{xml_category::MarkerCategory, xml_trail::Trail};

/// Markers format in the xml files are described under the <POIs> tag under the root <OverlayData> tag. The <POI> tag describes a marker.
/// everything is optional except xpos, ypos, zpos.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, PartialOrd)]
#[serde(rename = "POI")]
#[serde(rename_all = "camelCase")]
pub struct Marker {
    /// position of the marker in world space.
    pub xpos: f32,
    pub ypos: f32,
    pub zpos: f32,
    /// Describes which map the marker is located on.
    #[serde(rename = "MapID")]
    pub map_id: u32,
    /// base64 encoded string, optional. This is a unique identifier for the marker used in tracking activation of markers through the activationdata.xml file. If this doesn't exist for a marker, one will be generated automatically and added on the next export.
    #[serde(rename = "GUID")]
    pub guid: String,
    #[serde(rename = "mapDisplaySize")]
    pub map_display_size: Option<u32>,
    /// The icon to be displayed for the marker. If not given, this defaults to the image shown at the start of this article. This should point to a .png file. The overlay looks for the image files both starting from the root directory and the POIs directory for convenience. Make sure you don't use too high resolution (above 128x128) images because the texture atlas used for these is limited in size and it's a needless waste of resources to fill it quickly.
    #[serde(rename = "iconFile")]
    pub icon_file: Option<String>,
    /// The size of the icon in the game world. Default is 1.0 if this is not defined. Note that the "screen edges herd icons" option will limit the size of the displayed images for technical reasons.
    #[serde(rename = "iconSize")]
    pub icon_size: Option<f32>,
    /// How opaque the displayed icon should be. The default is 1.0
    pub alpha: Option<f32>,
    /// it describes the way the marker will behave when a player presses 'F' over it.
    pub behavior: Option<Behavior>,
    /// Specifies how high above the ground the marker is displayed. Default value is 1.5
    #[serde(rename = "heightOffset")]
    pub height_offset: Option<f32>,
    /// Determines how far the marker will start to fade out. If below 0, the marker won't disappear at any distance. Default is -1. This value is in game units (inches).
    #[serde(rename = "fadeNear")]
    pub fade_near: Option<u32>,
    /// Determines how far the marker will completely disappear. If below 0, the marker won't disappear at any distance. Default is -1. FadeFar needs to be higher than fadeNear for sane results. This value is in game units (inches).
    #[serde(rename = "fadeFar")]
    pub fade_far: Option<u32>,
    /// Determines the minimum size of a marker on the screen, in pixels.
    #[serde(rename = "minSize")]
    pub min_size: Option<u32>,
    /// Determines the maximum size of a marker on the screen, in pixels.
    #[serde(rename = "maxSize")]
    pub max_size: Option<u32>,
    /// For behavior 4 this tells how long the marker should be invisible after pressing 'F'. For behavior 5 this will tell how long a map cycle is.
    pub reset_length: Option<u32>,
    /// hex value. The color tint of the marker
    pub color: Option<u32>,
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
    #[serde(rename = "type")]
    pub category: Option<String>,
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

/// POIS tag under OverlayData which contains the array of tags POI/Trail
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename = "POIs")]
pub struct POIS {
    #[serde(rename = "POI")]
    pub poi: Option<Vec<Marker>>,
    #[serde(rename = "Trail")]
    pub trail: Option<Vec<Trail>>,
}

impl Marker {
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
}

