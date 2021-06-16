use cgmath::{Vector3};

/// Markers in the xml files are described under the <POIs> tag under the root <OverlayData> tag. The <POI> tag describes a marker.
pub struct Marker {
    /// position of the marker in world space.
    position: Option<Vector3<f32>>,
    /// Describes which map the marker is located on.
    map_id: Option<u32>,
    /// base64 encoded string, optional. This is a unique identifier for the marker used in tracking activation of markers through the activationdata.xml file. If this doesn't exist for a marker, one will be generated automatically and added on the next export.
    guid: Option<String>,
    /// The icon to be displayed for the marker. If not given, this defaults to the image shown at the start of this article. This should point to a .png file. The overlay looks for the image files both starting from the root directory and the POIs directory for convenience. Make sure you don't use too high resolution (above 128x128) images because the texture atlas used for these is limited in size and it's a needless waste of resources to fill it quickly.
    icon_file : Option<String>,
    /// The size of the icon in the game world. Default is 1.0 if this is not defined. Note that the "screen edges herd icons" option will limit the size of the displayed images for technical reasons.
    icon_size: Option<f32>,
    /// How opaque the displayed icon should be. The default is 1.0
    alpha: Option<f32>,
    /// it describes the way the marker will behave when a player presses 'F' over it.
    behavior: Option<Behavior>,
    /// Specifies how high above the ground the marker is displayed. Default value is 1.5
    height_offset: Option<f32>,
    /// Determines how far the marker will start to fade out. If below 0, the marker won't disappear at any distance. Default is -1. This value is in game units (inches).
    fade_near: Option<u32>,
    /// Determines how far the marker will completely disappear. If below 0, the marker won't disappear at any distance. Default is -1. FadeFar needs to be higher than fadeNear for sane results. This value is in game units (inches).
    fade_far: Option<u32>,
    /// Determines the minimum size of a marker on the screen, in pixels.
    min_size: Option<u32>,
    /// Determines the maximum size of a marker on the screen, in pixels.
    max_size: Option<u32>,
    /// For behavior 4 this tells how long the marker should be invisible after pressing 'F'. For behavior 5 this will tell how long a map cycle is.
    reset_length: Option<u32>,
    /// hex value. The color tint of the marker
    color: Option<u32>,
    /// Determines if going near the marker triggers it
    auto_trigger: Option<bool>,
    /// Determines if a marker has a countdown timer display when triggered
    has_countdown: Option<bool>,
    /// Determines the range from where the marker is triggered
    trigger_range: Option<f32>,
    /// An ID for an achievement from the GW2 API. Markers with the corresponding achievement ID will be hidden if the ID is marked as "done" for the API key that's entered in TacO.
    achievement_id: Option<u32>,
    /// This is similar to achievementId, but works for partially completed achievements as well, if the achievement has "bits", they can be individually referenced with this.
    achievement_bit: Option<u32>,
    /// his can be a multiline string, it will show up on screen as a text when the player is inside of infoRange of the marker
    info: Option<String>,
    /// This determines how far away from the marker the info string will be visible
    info_range: Option<f32>,
    map_visibility: Option<bool>,
    mini_map_visibility: Option<bool>,
    /// This text is used to display the type of the marker. It can contain spaces.
    display_name: Option<String>,
}

/*
behavior - integer. This is an important one, it describes the way the marker will behave when a player presses 'F' over it. The following values are valid for this parameter:
    0: the default value. Marker is always visible.
    1: 'Reappear on map change' - this is not implemented yet, it will be useful for markers that need to reappear if the player changes the map instance.
    2: 'Reappear on daily reset' - these markers disappear if the player presses 'F' over them, and reappear at the daily reset. These were used for the orphan markers during wintersday.
    3: 'Only visible before activation' - these markers disappear forever once the player pressed 'F' over them. Useful for collection style markers like golden lost badges, etc.
    4: 'Reappear after timer' - This behavior makes the marker reappear after a fix amount of time given in 'resetLength'.
    5: 'Reappear on map reset' - not implemented yet. This will make the marker reappear when the map cycles. In this case 'resetLength' will define the map cycle length in seconds, and 'resetOffset' will define when the first map cycle of the day begins after the daily reset, in seconds.
    6: 'Once per instance' - these markers disappear when triggered but reappear if you go into another instance of the map
    7: 'Once daily per character' - these markers disappear when triggered, but reappear with the daily reset, and can be triggered separately for every character

*/
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
