use crate::is_default;
use camino::Utf8PathBuf;
use serde::{Deserialize, Serialize};

/// Marker has 3 kinds of Attributes. Core, Filter and Dynamic
/// at the moment, we only support Core attributes.
///
#[serde_with::skip_serializing_none]
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub(crate) struct Marker {
    /// Alpha to multiply the texture pixels with, to make the whole marker transparent
    /// if not specified, we will use fully opaque
    pub alpha: Option<u8>,
    /// category id to which marker belongs to
    /// Validation: Category with this id must exist
    pub cat: Utf8PathBuf,

    /// The color tint to be mixed with the Marker
    /// format is sRGBA
    /// if unspecified, we will use [0, 0, 0, 0] a purely transparent color to not affect the marker
    pub color: Option<[u8; 4]>,
    /// Markers without rotation behave like billboards
    /// if this is Some, then the marker is statically rotated.
    /// if there's no rotation, we will make it face user as a billboard
    pub rotation: Option<[f32; 3]>,
    /// scale in x, y, z axes.
    /// default is unit scale. [1.0, 1.0, 1.0]
    pub scale: Option<[f32; 3]>,
    /// The name of the png to be used as texture
    /// Validation: should exist in the images/ directory as `name.png` file where `name` is the
    ///     contents of the string.
    /// if empty (default), use the default marker texture.
    pub texture: Option<String>,
    /// position of the marker.
    /// we will translate the default billboard at origin by the value of position
    /// units are inches.
    /// must exist or will default to [0.0, 0.0, 0.0]
    #[serde(skip_serializing_if = "is_default")]
    #[serde(default)]
    pub position: [f32; 3],
}
impl PartialOrd for Marker {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        // to sort markers by their category
        self.cat.partial_cmp(&other.cat)
    }
}
#[serde_with::skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
#[serde(default)]
pub struct Dynamic {
    #[serde(skip_serializing_if = "is_default")]
    pub trigger: Trigger,
    #[serde(skip_serializing_if = "is_default")]
    pub info: Option<Info>,
    #[serde(skip_serializing_if = "is_default")]
    pub tip: Option<u16>,
}

#[serde_with::skip_serializing_none]
#[derive(Debug, Default, Clone, Deserialize, Serialize, Copy, PartialEq)]
pub struct Trigger {
    range: Option<f32>,
    behavior: Option<Behavior>,
    toggle_cat: Option<u16>,
    copy: Option<CopyContents>,
}

#[serde_with::skip_serializing_none]
#[derive(Debug, Default, Clone, Deserialize, Serialize, PartialEq)]
#[serde(default)]
pub struct Filters {
    achievement: Option<Achievement>,
    schedule: Option<Schedule>,
    #[serde(skip_serializing_if = "is_default")]
    festival: Festivals,
    #[serde(skip_serializing_if = "is_default")]
    mounts: Mounts,
    #[serde(skip_serializing_if = "is_default")]
    professions: Professions,
    #[serde(skip_serializing_if = "is_default")]
    races: Races,
    /// the corresponding bit will be set for specializations
    #[serde(skip_serializing_if = "is_default")]
    specializations: Specializations,
    /// reference: https://github.com/Archomeda/Gw2Sharp/blob/master/Gw2Sharp/Models/MapType.cs
    /// we will not use this for now.
    #[serde(skip_serializing_if = "is_default")]
    maptype: MapTypes,
}

#[serde_with::skip_serializing_none]
#[derive(Debug, Default, Clone, Deserialize, Serialize, Copy, PartialEq)]
pub struct CopyContents {
    pub data: u16,
    pub message: Option<u16>,
}

#[serde_with::skip_serializing_none]
#[derive(Debug, Clone, Deserialize, Serialize, Copy, PartialEq)]
pub struct Achievement {
    pub id: u16,
    pub bit: Option<u8>,
}

#[serde_with::skip_serializing_none]
#[derive(Debug, Default, Clone, Deserialize, Serialize, PartialEq)]
pub struct Info {
    pub text: u16,
    pub range: Option<f32>,
}

#[serde_with::skip_serializing_none]
#[derive(Debug, Default, Clone, Deserialize, Serialize, PartialEq)]
pub struct Schedule {
    pub expression: String,
    pub duration: f32,
}

#[derive(Debug, Clone, Deserialize, Serialize, Copy, PartialEq)]
pub(crate) enum Behavior {
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
    #[derive(Default, Serialize, Deserialize)]
    pub(crate) struct MarkerFlags: u8 {
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
    #[derive(Default, Serialize, Deserialize)]
    pub(crate) struct Races: u8 {
        const ASURA  = 0b00000001;
        const CHARR  = 0b00000010;
        const HUMAN  = 0b00000100;
        const NORN  = 0b00001000;
        const SYLVARI = 0b00010000;
    }
}
bitflags::bitflags! {
    /// Filter which professions the marker should be active for. if its null, its available for all professions
    #[derive(Default, Serialize, Deserialize)]
    pub(crate) struct Professions: u16 {
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
    #[derive(Default, Serialize, Deserialize)]
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
    #[derive(Default, Serialize, Deserialize)]
    pub(crate) struct Festivals: u8 {
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
    #[derive(Default, Serialize, Deserialize)]
    pub(crate) struct Specializations: u128 {
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
    #[derive(Default, Serialize, Deserialize)]
    pub(crate) struct MapTypes: u32 {
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
