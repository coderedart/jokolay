use serde::{Deserialize, Serialize};
use validator::Validate;

#[serde_with::skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Marker {
    pub alpha: Option<u8>,
    pub cat: u16,
    pub color: Option<[u8; 4]>,
    pub fade_range: Option<[f32; 2]>,
    pub dynamic_props: Option<Dynamic>,
    pub texture: Option<u16>,
    pub flags: MarkerFlags,
    pub map_display_size: Option<u16>,
    pub map_fade_out_scale_level: Option<f32>,
    pub max_size: Option<u16>,
    pub min_size: Option<u16>,
    pub position: [f32; 3],
    pub scale: Option<f32>,
}

bitflags::bitflags! {
    #[derive(Default, Serialize, Deserialize)]
    pub struct MarkerFlags: u8 {
        const AUTO_TRIGGER  = 0b00000001;
        const COUNT_DOWN  = 0b00000010;
        const IN_GAME_VISIBILITY  = 0b00000100;
        const MAP_SCALE  = 0b00001000;
        const MAP_VISIBILITY = 0b00010000;
        const MINI_MAP_EDGE_HERD = 0b00100000;
        const MINI_MAP_VISIBILITY = 0b01000000;
    }
}
#[serde_with::skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Dynamic {
    pub trigger: Option<Trigger>,
    pub achievement: Option<Achievement>,
    pub info: Option<Info>,
}

#[serde_with::skip_serializing_none]
#[derive(Debug, Clone, Deserialize, Serialize, Copy, Validate)]
pub struct Trigger {
    pub range: f32,
    pub behavior: Option<Behavior>,
    pub toggle_cat: Option<u16>,
}

#[serde_with::skip_serializing_none]
#[derive(Debug, Clone, Deserialize, Serialize, Copy, Validate)]
pub struct Achievement {
    pub id: u16,
    pub bit: u8,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Info {
    pub text: String,
    pub range: Option<f32>,
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
