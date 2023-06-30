pub mod xml;

use indexmap::IndexMap;
use miette::Result;

pub const MARKER_PNG: &[u8] = include_bytes!("marker.png");
pub const TRAIL_PNG: &[u8] = include_bytes!("trail.png");
use glam::{Vec3, Vec3A};
use relative_path::RelativePathBuf;
use serde::{Deserialize, Serialize};
use serde_with::*;
use std::collections::BTreeMap;
use uuid::Uuid;

#[derive(Default)]
pub struct Pack {
    pub textures: BTreeMap<RelativePathBuf, Vec<u8>>,
    pub tbins: BTreeMap<RelativePathBuf, TBin>,
    pub categories: IndexMap<String, Category>,
    pub maps: BTreeMap<u16, MapData>,
}
#[derive(Default)]
pub struct MapData {
    pub markers: Vec<Marker>,
    pub trails: Vec<Trail>,
}

impl Pack {
    pub fn from_taco(_zip_file: &[u8]) -> Result<Self> {
        unimplemented!()
    }
}

#[derive(Debug)]
pub struct Category {
    pub display_name: String,
    pub separator: bool,
    pub default_enabled: bool,
    pub props: CommonAttributes,
    pub children: IndexMap<String, Category>,
}

#[derive(Debug)]
pub struct Marker {
    pub guid: Uuid,
    pub position: Vec3,
    pub map_id: u16,
    pub category: String,
    pub props: CommonAttributes,
}

#[derive(Debug)]
pub struct Trail {
    pub guid: Uuid,
    pub map_id: u16,
    pub category: String,
    pub props: CommonAttributes,
}
pub struct ActivationData {
    pub cats_status: bitvec::vec::BitVec,
    /// the key is marker id. and the value is the timestamp at which we can remove this entry.
    pub markers_status: BTreeMap<Uuid, u64>,
}

#[serde_as]
#[derive(Debug, Serialize, Deserialize)]
pub struct TBin {
    pub map_id: u16,
    pub version: u32,
    pub nodes: Vec<Vec3A>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Behavior {
    AlwaysVisible,
    /// live. marker_id
    ReappearOnMapChange,
    /// store. marker_id + next reset timestamp
    ReappearOnDailyReset,
    /// store. marker_id
    OnlyVisibleBeforeActivation,
    /// store. marker_id + timestamp of when to wakeup
    ReappearAfterTimer {
        reset_length: u32, // in seconds
    },
    /// store. marker_id + timestamp of next reset of map
    ReappearOnMapReset {
        map_cycle_length: u32,             // length of a map cycle in seconds
        map_cycle_offset_after_reset: u32, // how many seconds after daily reset does the new map cycle start in seconds
    },
    /// live. marker_id + instance ip / shard id
    OncePerInstance,
    /// store. marker_id + next reset. character data
    DailyPerChar,
    /// live. marker_id + instance_id + character_name
    OncePerInstancePerChar,
    /// I have no idea.
    WvWObjective,
}
bitflags::bitflags! {
    pub struct MarkerFlags: u8 {
        /// should the trigger activate when within trigger range
        const AUTO_TRIGGER  = 0b00000001;
        /// should we show the countdown timers for markers that are sleeping
        const COUNT_DOWN  = 0b00000010;
        /// whether the marker is drawn ingame
        const IN_GAME_VISIBILITY  = 0b00000100;
        /// scaling of marker on 2d map (or minimap)
        const MAP_SCALE  = 0b00001000;
        /// whether draw on map
        const MAP_VISIBILITY = 0b00010000;
        /// whether stays at the boundary of minimap when overbounds, just like personal weaypoint
        const MINI_MAP_EDGE_HERD = 0b00100000;
        /// draw on minimap
        const MINI_MAP_VISIBILITY = 0b01000000;
    }
}
bitflags::bitflags! {
    /// Filter which races the marker should be active for. if its null, its available for all races
    pub struct Races: u8 {
        const ASURA  = 0b00000001;
        const CHARR  = 0b00000010;
        const HUMAN  = 0b00000100;
        const NORN  = 0b00001000;
        const SYLVARI = 0b00010000;
    }
}
bitflags::bitflags! {
    /// Filter which professions the marker should be active for. if its null, its available for all professions
    pub struct Professions: u16 {
        const ELEMENTALIST  = 0b00000001;
        const ENGINEER  = 0b00000010;
        const GUARDIAN  = 0b00000100;
        const MESMER  = 0b00001000;
        const NECROMANCER = 0b00010000;
        const RANGER = 0b00100000;
        const REVENANT = 0b01000000;
        const THIEF = 0b10000000;
        const WARRIOR = 0b100000000;
    }
}
bitflags::bitflags! {
    /// Filter which mounts should the player be on for the markers to be visible
    pub struct Mounts: u16 {
        const GRIFFON  = 0b00000001;
        const JACKAL  = 0b00000010;
        const RAPTOR  = 0b00000100;
        const ROLLER_BEETLE  = 0b00001000;
        const SKIMMER = 0b00010000;
        const SKYSCALE = 0b00100000;
        const SPRINGER = 0b01000000;
        const WARCLAW = 0b10000000;
    }
}
bitflags::bitflags! {
    /// Filter for which festivals will the marker be active for
    pub struct Festivals: u8 {
        const DRAGON_BASH  = 0b00000001;
        const FESTIVAL_OF_THE_FOUR_WINDS  = 0b00000010;
        const HALLOWEEN  = 0b00000100;
        const LUNAR_NEW_YEAR  = 0b00001000;
        const SUPER_ADVENTURE_BOX = 0b00010000;
        const WINTERSDAY = 0b00100000;
    }
}

bitflags::bitflags! {
    /// Filter for which festivals will the marker be active for
    pub struct Specializations: u128 {
        const DUELING  = 1 << 0 ;
        const DEATH_MAGIC  = 1 << 1;
        const INVOCATION  = 1 << 2;
        const STRENGTH  = 1 << 3;
        const DRUID = 1 << 4;
        const EXPLOSIVES = 1 << 5;
        const DAREDEVIL = 1 << 6;
        const MARKSMANSHIP = 1 << 7;
        const RETRIBUTION = 1 << 8;
        const DOMINATION = 1 << 9;
        const TACTICS = 1 << 10;
        const SALVATION = 1 << 11;
        const VALOR = 1 << 12;
        const CORRUPTION = 1 << 13;
        const DEVASTATION = 1 << 14;
        const RADIANCE = 1 << 15;
        const WATER = 1 << 16;
        const BERSERKER = 1 << 17;
        const BLOOD_MAGIC = 1 << 18;
        const SHADOW_ARTS = 1 << 19;
        const TOOLS = 1 << 20;
        const DEFENSE  = 1 << 21;
        const INSPIRATION  = 1 << 22;
        const ILLUSIONS  = 1 << 23;
        const NATURE_MAGIC = 1 << 24;
        const EARTH = 1 << 25;
        const DRAGONHUNTER = 1 << 26;
        const DEADLY_ARTS = 1 << 27;
        const ALCHEMY = 1 << 28;
        const SKIRMISHING = 1 << 29;
        const FIRE = 1 << 30;
        const BEAST_MASTERY  = 1 << 31;
        const WILDERNESS_SURVIVAL  = 1 << 32;
        const REAPER  = 1 << 33;
        const CRITICAL_STRIKES = 1 << 34;
        const ARMS = 1 << 35;
        const ARCANE = 1 << 36;
        const FIREARMS = 1 << 37;
        const CURSES = 1 << 38;
        const CHRONOMANCER = 1 << 39;
        const AIR  = 1 << 40 ;
        const ZEAL  = 1 << 41;
        const SCRAPPER  = 1 << 42;
        const TRICKERY  = 1 << 43;
        const CHAOS = 1 << 44;
        const VIRTUES = 1 << 45;
        const INVENTIONS = 1 << 46;
        const TEMPEST = 1 << 47;
        const HONOR = 1 << 48;
        const SOUL_REAPING = 1 << 49;
        const DISCIPLINE  = 1 << 50 ;
        const HERALD  = 1 << 51;
        const SPITE  = 1 << 52;
        const ACROBATICS  = 1 << 53;
        const SOULBEAST = 1 << 54;
        const WEAVER = 1 << 55;
        const HOLOSMITH = 1 << 56;
        const DEADEYE = 1 << 57;
        const MIRAGE = 1 << 58;
        const SCOURGE = 1 << 59;
        const SPELLBREAKER  = 1 << 60 ;
        const FIREBRAND  = 1 << 61;
        const RENEGADE  = 1 << 62;
        const HARBINGER  = 1 << 63;
        const WILLBENDER = 1 << 64;
        const VIRTUOSO = 1 << 65;
        const CATALYST = 1 << 66;
        const BLADESWORN = 1 << 67;
        const VINDICATOR = 1 << 68;
        const MECHANIST = 1 << 69;
        const SPECTER  = 1 << 70 ;
        const UNTAMED  = 1 << 71;
    }
}

bitflags::bitflags! {
    pub struct MapTypes: u32 {
        /// <summary>
        /// Redirect map type, e.g. when logging in while in a PvP match.
        /// </summary>
        const REDIRECT = 1 << 0;

        /// <summary>
        /// Character create map type.
        /// </summary>
        const CHARACTER_CREATE = 1 << 1;

        /// <summary>
        /// PvP map type.
        /// </summary>
        const PVP = 1 << 2;

        /// <summary>
        /// GvG map type. Unused.
        /// Quote from lye: "lol unused ;_;".
        /// </summary>
        const GVG = 1 << 3;

        /// <summary>
        /// Instance map type, e.g. dungeons and story content.
        /// </summary>
        const INSTANCE = 1 << 4;

        /// <summary>
        /// Public map type, e.g. open world.
        /// </summary>
        const PUBLIC = 1 << 5;

        /// <summary>
        /// Tournament map type. Probably unused.
        /// </summary>
        const TOURNAMENT = 1 << 6;

        /// <summary>
        /// Tutorial map type.
        /// </summary>
        const TUTORIAL = 1 << 7;

        /// <summary>
        /// User tournament map type. Probably unused.
        /// </summary>
        const USER_TOURNAMENT = 1 << 8;

        /// <summary>
        /// Eternal Battlegrounds (WvW) map type.
        /// </summary>
        const ETERNAL_BATTLEGROUNDS = 1 << 9;

        /// <summary>
        /// Blue Borderlands (WvW) map type.
        /// </summary>
        const BLUE_BORDERLANDS = 1 << 10;

        /// <summary>
        /// Green Borderlands (WvW) map type.
        /// </summary>
        const GREEN_BORDERLANDS = 1 << 11;

        /// <summary>
        /// Red Borderlands (WvW) map type.
        /// </summary>
        const RED_BORDERLANDS = 1 << 12;

        /// <summary>
        /// Fortune's Vale. Unused.
        /// </summary>
        const FORTUNES_VALE = 1 << 13;

        /// <summary>
        /// Obsidian Sanctum (WvW) map type.
        /// </summary>
        const OBSIDIAN_SANCTUM = 1 << 14;

        /// <summary>
        /// Edge of the Mists (WvW) map type.
        /// </summary>
        const EDGE_OF_THE_MISTS = 1 << 15;

        /// <summary>
        /// Mini public map type, e.g. Dry Top, the Silverwastes and Mistlock Sanctuary.
        /// </summary>
        const PUBLIC_MINI = 1 << 16;

        /// <summary>
        /// WvW lounge map type, e.g. Armistice Bastion.
        /// </summary>
        const WVW_LOUNGE = 1 << 18;
    }
}

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
    #[derive(Debug, Clone, Default)]
    pub struct CommonAttributes {
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
        height_offset: Option<f32>,
        // hide: Option<String>,
        /// The icon to be displayed for the marker. If not given, this defaults to the image shown at the start of this article. This should point to a .png file. The overlay looks for the image files both starting from the root directory and the POIs directory for convenience. Make sure you don't use too high resolution (above 128x128) images because the texture atlas used for these is limited in size and it's a needless waste of resources to fill it quickly.Default value: 20
        icon_file: Option<RelativePathBuf>,
        /// The size of the icon in the game world. Default is 1.0 if this is not defined. Note that the "screen edges herd icons" option will limit the size of the displayed images for technical reasons.
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
        rotate: Option<[f32; 3]>,
        rotate_x: Option<f32>,
        rotate_y: Option<f32>,
        rotate_z: Option<f32>,
        /// if true, the markers/width of the trails belonging to this category will scale with the zoom level as you zoom in and out. Default value: true.
        // #[serde(rename = "scaleOnMapWithZoom")]
        // scale_on_map_with_zoom: Option<bool>,
        // show: Option<String>,
        // specialization: Option<Specializations>,
        texture: Option<RelativePathBuf>,
        // tip_name: Option<String>,
        // tip_description: Option<String>,
        /// will toggle the specified category on or off when triggered with the action key. or with auto_trigger/trigger_range
        // #[serde(rename = "toggleCategory")]
        // toggle_cateogry: Option<String>,
        trail_data_file: Option<RelativePathBuf>,
        trail_scale: Option<f32>,
        // Determines the range from where the marker is triggered. in meters.
        // trigger_range: Option<f32>,
    }
);

impl CommonAttributes {
    pub fn inherit_from_template(&mut self, other: &CommonAttributes) {
        self.inherit_if_prop_none(other);
    }
}
