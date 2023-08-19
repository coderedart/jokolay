use joko_core::prelude::bitflags;
use relative_path::RelativePathBuf;

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
bitflags! {
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
bitflags! {
    /// Filter which races the marker should be active for. if its null, its available for all races
    pub struct Races: u8 {
        const ASURA  = 0b00000001;
        const CHARR  = 0b00000010;
        const HUMAN  = 0b00000100;
        const NORN  = 0b00001000;
        const SYLVARI = 0b00010000;
    }
}
bitflags! {
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
bitflags! {
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
bitflags! {
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

bitflags! {
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

bitflags! {
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
/// made it using multi cursor (ctrl + shift + L) by copy-pasting json from api
fn _get_map_name_static(map_id: u32) -> Option<&'static str> {
    Some(match map_id {
        15 => "Queensdale",
        17 => "Harathi Hinterlands",
        18 => "Divinity's Reach",
        19 => "Plains of Ashford",
        20 => "Blazeridge Steppes",
        21 => "Fields of Ruin",
        22 => "Fireheart Rise",
        23 => "Kessex Hills",
        24 => "Gendarran Fields",
        25 => "Iron Marches",
        26 => "Dredgehaunt Cliffs",
        27 => "Lornar's Pass",
        28 => "Wayfarer Foothills",
        29 => "Timberline Falls",
        30 => "Frostgorge Sound",
        31 => "Snowden Drifts",
        32 => "Diessa Plateau",
        33 => "Ascalonian Catacombs",
        34 => "Caledon Forest",
        35 => "Metrica Province",
        36 => "Ascalonian Catacombs",
        37 => "Arson at the Orphanage",
        38 => "Eternal Battlegrounds",
        39 => "Mount Maelstrom",
        50 => "Lion's Arch",
        51 => "Straits of Devastation",
        53 => "Sparkfly Fen",
        54 => "Brisban Wildlands",
        55 => "The Hospital in Jeopardy",
        61 => "Infiltration",
        62 => "Cursed Shore",
        63 => "Sorrow's Embrace",
        64 => "Sorrow's Embrace",
        65 => "Malchor's Leap",
        66 => "Citadel of Flame",
        67 => "Twilight Arbor",
        68 => "Twilight Arbor",
        69 => "Citadel of Flame",
        70 => "Honor of the Waves",
        71 => "Honor of the Waves",
        73 => "Bloodtide Coast",
        75 => "Caudecus's Manor",
        76 => "Caudecus's Manor",
        77 => "Search the Premises",
        79 => "The Informant",
        80 => "A Society Function",
        81 => "Crucible of Eternity",
        82 => "Crucible of Eternity",
        89 => "Chasing the Culprits",
        91 => "The Grove",
        92 => "The Trial of Julius Zamon",
        95 => " Alpine Borderlands",
        96 => " Alpine Borderlands",
        97 => "Infiltration",
        110 => "The Perils of Friendship",
        111 => "Victory or Death",
        112 => "The Ruined City of Arah",
        113 => "Desperate Medicine",
        120 => "The Commander",
        138 => "Defense of Shaemoor",
        139 => "Rata Sum",
        140 => "The Apothecary",
        142 => "Going Undercover",
        143 => "Going Undercover",
        144 => "The Greater Good",
        145 => "The Rescue",
        147 => "Breaking the Blade",
        148 => "The Fall of Falcon Company",
        149 => "The Fall of Falcon Company",
        152 => "Confronting Captain Tervelan",
        153 => "Seek Logan's Aid",
        154 => "Seek Logan's Aid",
        157 => "Accusation",
        159 => "Accusation",
        161 => "Liberation",
        162 => "Voices From the Past",
        163 => "Voices From the Past",
        171 => "Rending the Mantle",
        172 => "Rending the Mantle",
        178 => "The Floating Grizwhirl",
        179 => "The Floating Grizwhirl",
        180 => "The Floating Grizwhirl",
        182 => "Clown College",
        184 => "The Artist's Workshop",
        185 => "Into the Woods",
        186 => "The Ringmaster",
        190 => "The Orders of Tyria",
        191 => "The Orders of Tyria",
        192 => "Brute Force",
        193 => "Mortus Virge",
        195 => "Triskell Quay",
        196 => "Track the Seraph",
        198 => "Speaker of the Dead",
        199 => "The Sad Tale of the \"Ravenous\"",
        201 => "Kellach's Attack",
        202 => "The Queen's Justice",
        203 => "The Trap",
        211 => "Best Laid Plans",
        212 => "Welcome Home",
        215 => "The Tribune's Call",
        216 => "The Tribune's Call",
        217 => "The Tribune's Call",
        218 => "Black Citadel",
        222 => "A Spy for a Spy",
        224 => "Scrapyard Dogs",
        225 => "A Spy for a Spy",
        226 => "On the Mend",
        232 => "Spilled Blood",
        234 => "Ghostbore Musket",
        237 => "Iron Grip of the Legion",
        238 => "The Flame Advances",
        239 => "The Flame Advances",
        242 => "Test Your Metal",
        244 => "Quick and Quiet",
        248 => "Salma District (Home)",
        249 => "An Unusual Inheritance",
        250 => "Windrock Maze",
        251 => "Mired Deep",
        252 => "Mired Deep",
        254 => "Deadly Force",
        255 => "Ghostbore Artillery",
        256 => "No Negotiations",
        257 => "Salvaging Scrap",
        258 => "Salvaging Scrap",
        259 => "In the Ruins",
        260 => "In the Ruins",
        262 => "Chain of Command",
        263 => "Chain of Command",
        264 => "Time for a Promotion",
        267 => "The End of the Line",
        269 => "Magic Users",
        271 => "Rage Suppression",
        272 => "Rage Suppression",
        274 => "Operation: Bulwark",
        275 => "AWOL",
        276 => "Human's Lament",
        282 => "Misplaced Faith",
        283 => "Thicker Than Water",
        284 => "Dishonorable Discharge",
        287 => "Searching for the Truth",
        288 => "Lighting the Beacons",
        290 => "Stoking the Flame",
        294 => "A Fork in the Road",
        295 => "Sins of the Father",
        297 => "Graveyard Ornaments",
        326 => "Hoelbrak",
        327 => "Desperate Medicine",
        330 => "Seraph Headquarters",
        334 => "Keg Brawl",
        335 => "Claw Island",
        336 => "Chantry of Secrets",
        350 => "Heart of the Mists",
        363 => "The Sting",
        364 => "Drawing Out the Cult",
        365 => "Ashes of the Past",
        371 => "Hero's Canton (Home)",
        372 => "Blood Tribune Quarters",
        373 => "The Command Core",
        374 => "Knut Whitebear's Loft",
        375 => "Hunter's Hearth (Home)",
        376 => "Stonewright's Steading",
        378 => "Queen's Throne Room",
        379 => "The Great Hunt",
        380 => "A Weapon of Legend",
        381 => "The Last of the Giant-Kings",
        382 => "Disciples of the Dragon",
        385 => "A Weapon of Legend",
        386 => "Echoes of Ages Past",
        387 => "Wild Spirits",
        388 => "Out of the Skies",
        389 => "Echoes of Ages Past",
        390 => "Twilight of the Wolf",
        391 => "Rage of the Minotaurs",
        392 => "A Pup's Illness",
        393 => "Through the Veil",
        394 => "A Trap Foiled",
        396 => "Raven's Revered",
        397 => "One Good Drink Deserves Another",
        399 => "Shape of the Spirit",
        400 => "Into the Mists",
        401 => "Through the Veil",
        405 => "Blessed of Bear",
        407 => "The Wolf Havroun",
        410 => "Minotaur Rampant",
        411 => "Minotaur Rampant",
        412 => "Unexpected Visitors",
        413 => "Rumors of Trouble",
        414 => "A New Challenger",
        415 => "Unexpected Visitors",
        416 => "Roadblock",
        417 => "Assault on Moledavia",
        418 => "Don't Leave Your Toys Out",
        419 => "A New Challenger",
        420 => "First Attack",
        421 => "The Finishing Blow",
        422 => "The Semifinals",
        423 => "The Championship Fight",
        424 => "The Championship Fight",
        425 => "The Machine in Action",
        427 => "Among the Kodan",
        428 => "Rumors of Trouble",
        429 => "Rage of the Minotaurs",
        430 => "Darkness at Drakentelt",
        432 => "Fighting the Nightmare",
        434 => "Preserving the Balance",
        435 => "Means to an End",
        436 => "Dredge Technology",
        439 => "Underground Scholar",
        440 => "Dredge Assault",
        441 => "The Dredge Hideout",
        444 => "Sabotage",
        447 => "Codebreaker",
        449 => "Armaments",
        453 => "Assault the Hill",
        454 => "Silent Warfare",
        455 => "Sever the Head",
        458 => "Fury of the Dead",
        459 => "A Fork in the Road",
        460 => "Citadel Stockade",
        464 => "Tribunes in Effigy",
        465 => "Sins of the Father",
        466 => "Misplaced Faith",
        470 => "Graveyard Ornaments",
        471 => "Undead Infestation",
        474 => "Whispers in the Dark",
        476 => "Dangerous Research",
        477 => "Digging Up Answers",
        480 => "Defending the Keep",
        481 => "Undead Detection",
        483 => "Ever Vigilant",
        485 => "Research and Destroy",
        487 => "Whispers of Vengeance",
        488 => "Killer Instinct",
        489 => "Meeting my Mentor",
        490 => "A Fragile Peace",
        492 => "Don't Shoot the Messenger",
        496 => "Meeting my Mentor",
        497 => "Dredging Up the Past",
        498 => "Dredging Up the Past",
        499 => "Scrapyard Dogs",
        502 => "Quaestor's Siege",
        503 => "Minister's Defense",
        504 => "Called to Service",
        505 => "Called to Service",
        507 => "Mockery of Death",
        509 => "Discovering Darkness",
        511 => "Hounds and the Hunted",
        512 => "Hounds and the Hunted",
        513 => "Loved and Lost",
        514 => "Saving the Stag",
        515 => "Hidden in Darkness",
        516 => "Good Work Spoiled",
        517 => "Black Night, White Stag",
        518 => "The Omphalos Chamber",
        519 => "Weakness of the Heart",
        520 => "Awakening",
        521 => "Holding Back the Darkness",
        522 => "A Sly Trick",
        523 => "Deep Tangled Roots",
        524 => "The Heart of Nightmare",
        525 => "Beneath a Cold Moon",
        527 => "The Knight's Duel",
        528 => "Hammer and Steel",
        529 => "Where Life Goes",
        532 => "After the Storm",
        533 => "After the Storm",
        534 => "Beneath the Waves",
        535 => "Mirror, Mirror",
        536 => "A Vision of Darkness",
        537 => "Shattered Light",
        538 => "An Unknown Soul",
        539 => "An Unknown Soul",
        540 => "Where Life Goes",
        542 => "Source of the Issue",
        543 => "Wild Growth",
        544 => "Wild Growth",
        545 => "Seeking the Zalisco",
        546 => "The Direct Approach",
        547 => "Trading Trickery",
        548 => "Eye of the Sun",
        549 => "Battle of Kyhlo",
        552 => "Seeking the Zalisco",
        554 => "Forest of Niflhel",
        556 => "A Different Dream",
        557 => "A Splinter in the Flesh",
        558 => "Shadow of the Tree",
        559 => "Eye of the Sun",
        560 => "Sharpened Thorns",
        561 => "Bramble Walls",
        563 => "Secrets in the Earth",
        564 => "The Blossom of Youth",
        566 => "The Bad Apple",
        567 => "Trouble at the Roots",
        569 => "Flower of Death",
        570 => "Dead of Winter",
        571 => "A Tangle of Weeds",
        573 => "Explosive Intellect",
        574 => "In Snaff's Footsteps",
        575 => "Golem Positioning System",
        576 => "Monkey Wrench",
        577 => "Defusing the Problem",
        578 => "The Things We Do For Love",
        579 => "The Snaff Prize",
        581 => "A Sparkling Rescue",
        582 => "High Maintenance",
        583 => "Snaff Would Be Proud",
        584 => "Taking Credit Back",
        586 => "Political Homicide",
        587 => "Here, There, Everywhere",
        588 => "Piece Negotiations",
        589 => "Readings On the Rise",
        590 => "Snaff Would Be Proud",
        591 => "Readings On the Rise",
        592 => "Unscheduled Delay",
        594 => "Stand By Your Krewe",
        595 => "Unwelcome Visitors",
        596 => "Where Credit Is Due",
        597 => "Where Credit Is Due",
        598 => "Short Fuse",
        599 => "Short Fuse",
        606 => "Salt in the Wound",
        607 => "Free Rein",
        608 => "Serving Up Trouble",
        609 => "Serving Up Trouble",
        610 => "Flash Flood",
        611 => "I Smell a Rat",
        613 => "Magnum Opus",
        614 => "Magnum Opus",
        617 => "Bad Business",
        618 => "Beta Test",
        619 => "Beta Test",
        620 => "Any Sufficiently Advanced Science",
        621 => "Any Sufficiently Advanced Science",
        622 => "Bad Forecast",
        623 => "Industrial Espionage",
        624 => "Split Second",
        625 => "Carry a Big Stick",
        627 => "Meeting my Mentor",
        628 => "Stealing Secrets",
        629 => "A Bold New Theory",
        630 => "Forging Permission",
        631 => "Forging Permission",
        633 => "Setting the Stage",
        634 => "Containment",
        635 => "Containment",
        636 => "Hazardous Environment",
        638 => "Down the Hatch",
        639 => "Down the Hatch",
        642 => "The Stone Sheath",
        643 => "Bad Blood",
        644 => "Test Subject",
        645 => "Field Test",
        646 => "The House of Caithe",
        647 => "Dreamer's Terrace (Home)",
        648 => "The Omphalos Chamber",
        649 => "Snaff Memorial Lab",
        650 => "Applied Development Lab (Home)",
        651 => "Council Level",
        652 => "A Meeting of the Minds",
        653 => "Mightier than the Sword",
        654 => "They Went Thataway",
        655 => "Lines of Communication",
        656 => "Untamed Wilds",
        657 => "An Apple a Day",
        658 => "Base of Operations",
        659 => "The Lost Chieftain's Return",
        660 => "Thrown Off Guard",
        662 => "Pets and Walls Make Stronger Kraals",
        663 => "Doubt",
        664 => "The False God's Lair",
        666 => "Bad Ice",
        667 => "Bad Ice",
        668 => "Pets and Walls Make Stronger Kraals",
        669 => "Attempted Deicide",
        670 => "Doubt",
        672 => "Rat-Tastrophe",
        673 => "Salvation Through Heresy",
        674 => "Enraged and Unashamed",
        675 => "Pastkeeper",
        676 => "Protest Too Much",
        677 => "Prying the Eye Open",
        678 => "The Hatchery",
        680 => "Convincing the Faithful",
        681 => "Evacuation",
        682 => "Untamed Wilds",
        683 => "Champion's Sacrifice",
        684 => "Thieving from Thieves",
        685 => "Crusader's Return",
        686 => "Unholy Grounds",
        687 => "Chosen of the Sun",
        691 => "Set to Blow",
        692 => "Gadd's Last Gizmo",
        693 => "Library Science",
        694 => "Rakt and Ruin",
        695 => "Suspicious Activity",
        696 => "Reconnaissance",
        697 => "Critical Blowback",
        698 => "The Battle of Claw Island",
        699 => "Suspicious Activity",
        700 => "Priory Library",
        701 => "On Red Alert",
        702 => "Forearmed Is Forewarned",
        703 => "The Oratory",
        704 => "Killing Fields",
        705 => "The Ghost Rite",
        706 => "The Good Fight",
        707 => "Defense Contract",
        708 => "Shards of Orr",
        709 => "The Sound of Psi-Lance",
        710 => "Early Parole",
        711 => "Magic Sucks",
        712 => "A Light in the Darkness",
        713 => "The Priory Assailed",
        714 => "Under Siege",
        715 => "Retribution",
        716 => "Retribution",
        719 => "The Sound of Psi-Lance",
        726 => "Wet Work",
        727 => "Shell Shock",
        728 => "Volcanic Extraction",
        729 => "Munition Acquisition",
        730 => "To the Core",
        731 => "The Battle of Fort Trinity",
        732 => "Tower Down",
        733 => "Forging the Pact",
        735 => "Willing Captives",
        736 => "Marshaling the Truth",
        737 => "Breaking the Bone Ship",
        738 => "Liberating Apatia",
        739 => "Liberating Apatia",
        743 => "Fixing the Blame",
        744 => "A Sad Duty",
        745 => "Striking off the Chains",
        746 => "Delivering Justice",
        747 => "Intercepting the Orb",
        750 => "Close the Eye",
        751 => "Through the Looking Glass",
        758 => "The Cathedral of Silence",
        760 => "Starving the Beast",
        761 => "Stealing Light",
        762 => "Hunters and Prey",
        763 => "Romke's Final Voyage",
        764 => "Marching Orders",
        766 => "Air Drop",
        767 => "Estate of Decay",
        768 => "What the Eye Beholds",
        769 => "Conscript the Dead Ships",
        772 => "Ossuary of Unquiet Dead",
        775 => "Temple of the Forgotten God",
        776 => "Temple of the Forgotten God",
        777 => "Temple of the Forgotten God",
        778 => "Through the Looking Glass",
        779 => "Starving the Beast",
        780 => "Against the Corruption",
        781 => "The Source of Orr",
        782 => "Armor Guard",
        783 => "Blast from the Past",
        784 => "The Steel Tide",
        785 => "Further Into Orr",
        786 => "Ships of the Line",
        787 => "Source of Orr",
        788 => "Victory or Death",
        789 => "A Grisly Shipment",
        790 => "Blast from the Past",
        792 => "A Pup's Illness",
        793 => "Hunters and Prey",
        795 => "Legacy of the Foefire",
        796 => "The Informant",
        797 => "A Traitor's Testimony",
        799 => "Follow the Trail",
        806 => "Awakening",
        807 => "Eye of the North",
        820 => "The Omphalos Chamber",
        821 => "The Omphalos Chamber",
        825 => "Codebreaker",
        827 => "Caer Aval",
        828 => "The Durmand Priory",
        830 => "Vigil Headquarters",
        833 => "Ash Tribune Quarters",
        845 => "Shattered Light",
        862 => "Reaper's Rumble",
        863 => "Ascent to Madness",
        864 => "Lunatic Inquisition",
        865 => "Mad King's Clock Tower",
        866 => "Mad King's Labyrinth",
        872 => "Fractals of the Mists",
        873 => "Southsun Cove",
        875 => "Temple of the Silent Storm",
        877 => "Snowball Mayhem",
        878 => "Tixx's Infinirarium",
        880 => "Toypocalypse",
        881 => "Bell Choir Ensemble",
        882 => "Winter Wonderland",
        894 => "Spirit Watch",
        895 => "Super Adventure Box",
        896 => "North Nolan Hatchery",
        897 => "Cragstead",
        899 => "Obsidian Sanctum",
        900 => "Skyhammer",
        901 => "Molten Furnace",
        905 => "Crab Toss",
        911 => "Dragon Ball Arena",
        912 => "Ceremony and Acrimony—Memorials on the Pyre",
        913 => "Hard Boiled—The Scene of the Crime",
        914 => "The Dead End",
        915 => "Aetherblade Retreat",
        917 => "No More Secrets—The Scene of the Crime",
        918 => "Aspect Arena",
        919 => "Sanctum Sprint",
        920 => "Southsun Survival",
        922 => "Labyrinthine Cliffs",
        924 => "Grandmaster of Om",
        929 => "The Crown Pavilion",
        930 => "Opening Ceremony",
        931 => "Scarlet's Playhouse",
        932 => "Closing Ceremony",
        934 => "Super Adventure Box",
        935 => "Super Adventure Box",
        937 => "Scarlet's End",
        943 => "The Tower of Nightmares (Public)",
        945 => "The Nightmare Ends",
        947 => "Fractals of the Mists",
        948 => "Fractals of the Mists",
        949 => "Fractals of the Mists",
        950 => "Fractals of the Mists",
        951 => "Fractals of the Mists",
        952 => "Fractals of the Mists",
        953 => "Fractals of the Mists",
        954 => "Fractals of the Mists",
        955 => "Fractals of the Mists",
        956 => "Fractals of the Mists",
        957 => "Fractals of the Mists",
        958 => "Fractals of the Mists",
        959 => "Fractals of the Mists",
        960 => "Fractals of the Mists",
        964 => "Scarlet's Secret Lair",
        965 => "The Origins of Madness: A Moment's Peace",
        968 => "Edge of the Mists",
        971 => "The Dead End: A Study in Scarlet",
        973 => "The Evacuation of Lion's Arch",
        980 => "The Dead End: Celebration",
        984 => "Courtyard",
        987 => "Lion's Arch: Honored Guests",
        988 => "Dry Top",
        989 => "Prosperity's Mystery",
        990 => "Cornered",
        991 => "Disturbance in Brisban Wildlands",
        992 => "Fallen Hopes",
        993 => "Scarlet's Secret Room",
        994 => "The Concordia Incident",
        997 => "Discovering Scarlet's Breakthrough",
        998 => "The Machine",
        999 => "Trouble at Fort Salma",
        1000 => "The Waypoint Conundrum",
        1001 => "Summit Invitations",
        1002 => "Mission Accomplished",
        1003 => "Rallying Call",
        1004 => "Plan of Attack",
        1005 => "Party Politics",
        1006 => "Foefire Cleansing",
        1007 => "Recalibrating the Waypoints",
        1008 => "The Ghosts of Fort Salma",
        1009 => "Taimi's Device",
        1010 => "The World Summit",
        1011 => "Battle of Champion's Dusk",
        1015 => "The Silverwastes",
        1016 => "Hidden Arcana",
        1017 => "Reunion with the Pact",
        1018 => "Caithe's Reconnaissance Squad",
        1019 => "Fort Trinity",
        1021 => "Into the Labyrinth",
        1022 => "Return to Camp Resolve",
        1023 => "Tracking the Aspect Masters",
        1024 => "No Refuge",
        1025 => "The Newly Awakened",
        1026 => "Meeting the Asura",
        1027 => "Pact Assaulted",
        1028 => "The Mystery Cave",
        1029 => "Arcana Obscura",
        1032 => "Prized Possessions",
        1033 => "Buried Insight",
        1037 => "The Jungle Provides",
        1040 => "Hearts and Minds",
        1041 => "Dragon's Stand",
        1042 => "Verdant Brink",
        1043 => "Auric Basin",
        1045 => "Tangled Depths",
        1046 => "Roots of Terror",
        1048 => "City of Hope",
        1050 => "Torn from the Sky",
        1051 => "Prisoners of the Dragon",
        1052 => "Verdant Brink",
        1054 => "Bitter Harvest",
        1057 => "Strange Observations",
        1058 => "Prologue: Rally to Maguuma",
        1062 => "Spirit Vale",
        1063 => "Southsun Crab Toss",
        1064 => "Claiming the Lost Precipice",
        1065 => "Angvar's Trove",
        1066 => "Claiming the Gilded Hollow",
        1067 => "Angvar's Trove",
        1068 => "Gilded Hollow",
        1069 => "Lost Precipice",
        1070 => "Claiming the Lost Precipice",
        1071 => "Lost Precipice",
        1072 => "Southsun Crab Toss",
        1073 => "Guild Initiative Office",
        1074 => "Blightwater Shatterstrike",
        1075 => "Proxemics Lab",
        1076 => "Lost Precipice",
        1078 => "Claiming the Gilded Hollow",
        1079 => "Deep Trouble",
        1080 => "Branded for Termination",
        1081 => "Langmar Estate",
        1082 => "Langmar Estate",
        1083 => "Deep Trouble",
        1084 => "Southsun Crab Toss",
        1086 => "Save Our Supplies",
        1087 => "Proxemics Lab",
        1088 => "Claiming the Gilded Hollow",
        1089 => "Angvar's Trove",
        1090 => "Langmar Estate",
        1091 => "Save Our Supplies",
        1092 => "Scratch Sentry Defense",
        1093 => "Angvar's Trove",
        1094 => "Save Our Supplies",
        1095 => "Dragon's Stand (Heart of Thorns)",
        1097 => "Proxemics Lab",
        1098 => "Claiming the Gilded Hollow",
        1099 => " Desert Borderlands",
        1100 => "Scratch Sentry Defense",
        1101 => "Gilded Hollow",
        1104 => "Lost Precipice",
        1105 => "Langmar Estate",
        1106 => "Deep Trouble",
        1107 => "Gilded Hollow",
        1108 => "Gilded Hollow",
        1109 => "Angvar's Trove",
        1110 => "Scrap Rifle Field Test",
        1111 => "Scratch Sentry Defense",
        1112 => "Branded for Termination",
        1113 => "Scratch Sentry Defense",
        1115 => "Haywire Punch-o-Matic Battle",
        1116 => "Deep Trouble",
        1117 => "Claiming the Lost Precipice",
        1118 => "Save Our Supplies",
        1121 => "Gilded Hollow",
        1122 => "Claiming the Gilded Hollow",
        1123 => "Blightwater Shatterstrike",
        1124 => "Lost Precipice",
        1126 => "Southsun Crab Toss",
        1128 => "Scratch Sentry Defense",
        1129 => "Langmar Estate",
        1130 => "Deep Trouble",
        1131 => "Blightwater Shatterstrike",
        1132 => "Claiming the Lost Precipice",
        1133 => "Branded for Termination",
        1134 => "Blightwater Shatterstrike",
        1135 => "Branded for Termination",
        1136 => "Proxemics Lab",
        1137 => "Proxemics Lab",
        1138 => "Save Our Supplies",
        1139 => "Southsun Crab Toss",
        1140 => "Claiming the Lost Precipice",
        1142 => "Blightwater Shatterstrike",
        1146 => "Branded for Termination",
        1147 => "Spirit Vale",
        1149 => "Salvation Pass",
        1153 => "Tiger Den",
        1154 => "Special Forces Training Area",
        1155 => "Lion's Arch Aerodrome",
        1156 => "Stronghold of the Faithful",
        1158 => "Noble's Folly",
        1159 => "Research in Rata Novus",
        1161 => "Eir's Homestead",
        1163 => "Revenge of the Capricorn",
        1164 => "Fractals of the Mists",
        1165 => "Bloodstone Fen",
        1166 => "Confessor's Stronghold",
        1167 => "A Shadow's Deeds",
        1169 => "Rata Novus",
        1170 => "Taimi's Game",
        1171 => "Eternal Coliseum",
        1172 => "Dragon Vigil",
        1173 => "Taimi's Game",
        1175 => "Ember Bay",
        1176 => "Taimi's Game",
        1177 => "Fractals of the Mists",
        1178 => "Bitterfrost Frontier",
        1180 => "The Bitter Cold",
        1181 => "Frozen Out",
        1182 => "Precocious Aurene",
        1185 => "Lake Doric",
        1188 => "Bastion of the Penitent",
        1189 => "Regrouping with the Queen",
        1190 => "A Meeting of Ministers",
        1191 => "Confessor's End",
        1192 => "The Second Vision",
        1193 => "The First Vision",
        1194 => "The Sword Regrown",
        1195 => "Draconis Mons",
        1196 => "Heart of the Volcano",
        1198 => "Taimi's Pet Project",
        1200 => "Hall of the Mists",
        1201 => "Asura Arena",
        1202 => "White Mantle Hideout",
        1203 => "Siren's Landing",
        1204 => "Palace Temple",
        1205 => "Fractals of the Mists",
        1206 => "Mistlock Sanctuary",
        1207 => "The Last Chance",
        1208 => "Shining Blade Headquarters",
        1209 => "The Sacrifice",
        1210 => "Crystal Oasis",
        1211 => "Desert Highlands",
        1212 => "Office of the Chief Councilor",
        1214 => "Windswept Haven",
        1215 => "Windswept Haven",
        1217 => "Sparking the Flame",
        1219 => "Enemy of My Enemy: The Beastmarshal",
        1220 => "Sparking the Flame (Prologue)",
        1221 => "The Way Forward",
        1222 => "Claiming Windswept Haven",
        1223 => "Small Victory (Epilogue)",
        1224 => "Windswept Haven",
        1226 => "The Desolation",
        1227 => "Hallowed Ground: Tomb of Primeval Kings",
        1228 => "Elon Riverlands",
        1230 => "Facing the Truth: The Sanctum",
        1231 => "Claiming Windswept Haven",
        1232 => "Windswept Haven",
        1234 => "To Kill a God",
        1236 => "Claiming Windswept Haven",
        1240 => "Blazing a Trail",
        1241 => "Night of Fires",
        1242 => "Zalambur's Office",
        1243 => "Windswept Haven",
        1244 => "Claiming Windswept Haven",
        1245 => "The Departing",
        1246 => "Captain Kiel's Office",
        1247 => "Enemy of My Enemy",
        1248 => "Domain of Vabbi",
        1250 => "Windswept Haven",
        1252 => "Crystalline Memories",
        1253 => "Beast of War",
        1255 => "Enemy of My Enemy: The Troopmarshal",
        1256 => "The Dark Library",
        1257 => "Spearmarshal's Lament",
        1260 => "Eye of the Brandstorm",
        1263 => "Domain of Istan",
        1264 => "Hall of Chains",
        1265 => "The Hero of Istan",
        1266 => "Cave of the Sunspear Champion",
        1267 => "Fractals of the Mists",
        1268 => "Fahranur, the First City",
        1270 => "Toypocalypse",
        1271 => "Sandswept Isles",
        1274 => "The Charge",
        1275 => "Courtyard",
        1276 => "The Test Subject",
        1277 => "The Charge",
        1278 => "???",
        1279 => "ERROR: SIGNAL LOST",
        1281 => "A Kindness Repaid",
        1282 => "Tracking the Scientist",
        1283 => "???",
        1285 => "???",
        1288 => "Domain of Kourna",
        1289 => "Seized",
        1290 => "Fractals of the Mists",
        1291 => "Forearmed Is Forewarned",
        1292 => "Be My Guest",
        1294 => "Sun's Refuge",
        1295 => "Legacy",
        1296 => "Storm Tracking",
        1297 => "A Shattered Nation",
        1299 => "Storm Tracking",
        1300 => "From the Ashes—The Deadeye",
        1301 => "Jahai Bluffs",
        1302 => "Storm Tracking",
        1303 => "Mythwright Gambit",
        1304 => "Mad King's Raceway",
        1305 => "Djinn's Dominion",
        1306 => "Secret Lair of the Snowmen (Squad)",
        1308 => "Scion & Champion",
        1309 => "Fractals of the Mists",
        1310 => "Thunderhead Peaks",
        1313 => "The Crystal Dragon",
        1314 => "The Crystal Blooms",
        1315 => "Armistice Bastion",
        1316 => "Mists Rift",
        1317 => "Dragonfall",
        1318 => "Dragonfall",
        1319 => "Descent",
        1320 => "The End",
        1321 => "Dragonflight",
        1322 => "Epilogue",
        1323 => "The Key of Ahdashim",
        1326 => "Dragon Bash Arena",
        1327 => "Dragon Arena Survival",
        1328 => "Auric Span",
        1329 => "Coming Home",
        1330 => "Grothmar Valley",
        1331 => "Strike Mission: Shiverpeaks Pass (Public)",
        1332 => "Strike Mission: Shiverpeaks Pass (Squad)",
        1334 => "Deeper and Deeper",
        1336 => "A Race to Arms",
        1338 => "Bad Blood",
        1339 => "Weekly Strike Mission: Boneskinner (Squad)",
        1340 => "Weekly Strike Mission: Voice of the Fallen and Claw of the Fallen (Public)",
        1341 => "Weekly Strike Mission: Fraenir of Jormag (Squad)",
        1342 => "The Invitation",
        1343 => "Bjora Marches",
        1344 => "Weekly Strike Mission: Fraenir of Jormag (Public)",
        1345 => "What's Left Behind",
        1346 => "Weekly Strike Mission: Voice of the Fallen and Claw of the Fallen (Squad)",
        1349 => "Silence",
        1351 => "Weekly Strike Mission: Boneskinner (Public)",
        1352 => "Secret Lair of the Snowmen (Public)",
        1353 => "Celestial Challenge",
        1355 => "Voice in the Deep",
        1356 => "Chasing Ghosts",
        1357 => "Strike Mission: Whisper of Jormag (Public)",
        1358 => "Eye of the North",
        1359 => "Strike Mission: Whisper of Jormag (Squad)",
        1361 => "The Nightmare Incarnate",
        1362 => "Forging Steel (Public)",
        1363 => "New Friends, New Enemies—North Nolan Hatchery",
        1364 => "The Battle for Cragstead",
        1366 => "Darkrime Delves",
        1368 => "Forging Steel (Squad)",
        1369 => "Canach's Lair",
        1370 => "Eye of the North",
        1371 => "Drizzlewood Coast",
        1372 => "Turnabout",
        1373 => "Pointed Parley",
        1374 => "Strike Mission: Cold War (Squad)",
        1375 => "Snapping Steel",
        1376 => "Strike Mission: Cold War (Public)",
        1378 => "Behind Enemy Lines",
        1379 => "One Charr, One Dragon, One Champion",
        1380 => "Epilogue",
        1382 => "Arena of the Wolverine",
        1383 => "A Simple Negotiation",
        1384 => "Fractals of the Mists",
        1385 => "Caledon Forest (Private)",
        1386 => "Thunderhead Peaks (Private)",
        1387 => "Bloodtide Coast (Public)",
        1388 => "Snowden Drifts (Private)",
        1389 => "Snowden Drifts (Public)",
        1390 => "Fireheart Rise (Public)",
        1391 => "Brisban Wildlands (Private)",
        1392 => "Primordus Rising",
        1393 => "Lake Doric (Public)",
        1394 => "Bloodtide Coast (Private)",
        1395 => "Thunderhead Peaks (Public)",
        1396 => "Gendarran Fields (Public)",
        1397 => "Metrica Province (Public)",
        1398 => "Fields of Ruin (Public)",
        1399 => "Brisban Wildlands (Public)",
        1400 => "Fields of Ruin (Private)",
        1401 => "Metrica Province (Private)",
        1402 => "Lake Doric (Private)",
        1403 => "Caledon Forest (Public)",
        1404 => "Fireheart Rise (Private)",
        1405 => "Gendarran Fields (Private)",
        1407 => "Council Level",
        1408 => "Wildfire",
        1409 => "Dragonstorm (Private Squad)",
        1410 => "Champion's End",
        1411 => "Dragonstorm (Public)",
        1412 => "Dragonstorm",
        1413 => "The Twisted Marionette (Public)",
        1414 => "The Twisted Marionette (Private Squad)",
        1415 => "The Future in Jade: Power Plant",
        1416 => "Deepest Secrets: Yong Reactor",
        1419 => "Isle of Reflection",
        1420 => "Fallout: Nika's Blade",
        1421 => "???",
        1422 => "Dragon's End",
        1426 => "Isle of Reflection",
        1427 => "Weight of the World: Lady Joon's Estate",
        1428 => "Arborstone",
        1429 => "The Cycle, Reborn: Arborstone",
        1430 => "Claiming the Isle of Reflection",
        1432 => "Strike Mission: Aetherblade Hideout",
        1433 => "Old Friends",
        1434 => "Empty",
        1435 => "Isle of Reflection",
        1436 => "Extraction Point: Command Quarters",
        1437 => "Strike Mission: Harvest Temple",
        1438 => "New Kaineng City",
        1439 => "The Only One",
        1440 => "Laying to Rest",
        1442 => "Seitung Province",
        1444 => "Isle of Reflection",
        1445 => "The Future in Jade: Nahpui Lab",
        1446 => "Aetherblade Armada",
        1448 => "The Cycle, Reborn: The Dead End Bar",
        1449 => "Aurene's Sanctuary",
        1450 => "Strike Mission: Xunlai Jade Junkyard",
        1451 => "Strike Mission: Kaineng Overlook",
        1452 => "The Echovald Wilds",
        1453 => "Ministry of Security: Main Office",
        1454 => "The Scenic Route: Kaineng Docks",
        1456 => "Claiming the Isle of Reflection",
        1457 => "Detention Facility",
        1458 => "Aurene's Sanctuary",
        1459 => "Claiming the Isle of Reflection",
        1460 => "Empress Ihn's Court",
        1461 => "Zen Daijun Hideaway",
        1462 => "Isle of Reflection",
        1463 => "Claiming the Isle of Reflection",
        1464 => "Fallout: Arborstone",
        1465 => "Thousand Seas Pavilion",
        1466 => "A Quiet Celebration—Knut Whitebear's Loft",
        1467 => "New Friends, New Enemies—The Command Core",
        1468 => "The Battle for Cragstead—Knut Whitebear's Loft",
        1469 => "New Friends, New Enemies—Blood Tribune Quarters",
        1470 => "A Quiet Celebration—Citadel Stockade",
        1471 => "Case Closed—The Dead End",
        1472 => "Hard Boiled—The Dead End",
        1474 => "Picking Up the Pieces",
        1477 => "The Tower of Nightmares (Private Squad)",
        1478 => "The Battle for Lion's Arch (Private Squad)",
        1480 => "The Twisted Marionette",
        1481 => "Battle on the Breachmaker",
        1482 => "The Battle For Lion's Arch (Public)",
        1483 => "Memory of Old Lion's Arch",
        1484 => "North Evacuation Camp",
        1485 => "Strike Mission: Old Lion's Court",
        1487 => "The Aether Escape",
        1488 => "On the Case: Excavation Yard",
        1489 => "A Raw Deal: Red Duck Tea House",
        1490 => "Gyala Delve",
        1491 => "Deep Trouble: Excavation Yard",
        1492 => "Deep Trouble: The Deep",
        1494 => "Entrapment: The Deep",
        1495 => "A Plan Emerges: Power Plant",
        1496 => "Emotional Release: Jade Pools",
        1497 => "Emotional Release: Command Quarters",
        1498 => "Full Circle: Red Duck Tea House",
        1499 => "Forward",
        1500 => "Fractals of the Mists",

        _ => {
            return None;
        }
    })
}
