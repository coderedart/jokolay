use serde::{Deserialize, Serialize};

use jokotypes::*;

#[serde_with::skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Marker {
    pub id: MarkerID,
    pub position: [f32; 3],
    pub achievement: Option<Achievement>,
    pub alpha: Option<f32>,
    pub color: Option<[u8; 4]>,
    pub fade_range: Option<[f32; 2]>,
    pub dynamic_props: Option<Dynamic>,
    pub image: Option<ImageHash>,
    pub in_game_visibility: Option<bool>,
    pub keep_on_map_edge: Option<bool>,
    pub map_display_size: Option<u16>,
    pub map_fade_out_scale_level: Option<f32>,
    pub map_visibility: Option<bool>,
    pub max_size: Option<u16>,
    pub min_size: Option<u16>,
    pub mini_map_visibility: Option<bool>,
    pub scale: Option<f32>,
    pub scale_on_map_with_zoom: Option<bool>,
}

#[serde_with::skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Dynamic {
    pub trigger: Option<Trigger>,
    pub info: Option<Info>,
}

#[serde_with::skip_serializing_none]
#[derive(Debug, Clone, Deserialize, Serialize, Copy, Default)]
pub struct Trigger {
    pub auto_trigger: Option<bool>,
    pub count_down: Option<bool>,
    pub range: f32,
    pub behavior: Option<Behavior>,
    pub toggle_cat: Option<CategoryID>,
}

#[serde_with::skip_serializing_none]
#[derive(Debug, Clone, Deserialize, Serialize, Copy, Default)]
pub struct Achievement {
    pub id: u32,
    pub bit: Option<u32>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Info {
    pub text: String,
    pub range: f32,
}

#[derive(Debug, Clone, Deserialize, Serialize, Copy)]
pub enum Behavior {
    AlwaysVisible,
    ReappearOnMapChange,
    ReappearOnDailyReset,
    OnlyVisibleBeforeActivation,
    ReappearAfterTimer {
        reset_length: u32, // in seconds
    },
    ReappearOnMapReset {
        map_cycle_length: u32,             // length of a map cycle in seconds
        map_cycle_offset_after_reset: u32, // how many seconds after daily reset does the new map cycle start in seconds
    },
    OncePerInstance,
    DailyPerChar,
    OncePerInstancePerChar,
    WvWObjective,
}
impl Default for Behavior {
    fn default() -> Self {
        Behavior::AlwaysVisible
    }
}
