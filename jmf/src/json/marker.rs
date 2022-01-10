use std::num::NonZeroU16;

use serde::{Deserialize, Serialize};
use validator::{Validate};
use jokotypes::*;

pub const MIN_RANGE: f32 = 0.0;

#[serde_with::skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Marker {
    pub id: MarkerID,
    pub position: [f32; 3],
    pub alpha: Option<u8>,
    pub color: Option<rgb::RGBA8>,
    pub fade_range: Option<[f32; 2]>,
    pub dynamic_props: Option<Dynamic>,
    pub image: Option<ImageID>,
    pub flags: Option<MarkerFlags>,
    pub map_display_size: Option<u16>,
    pub map_fade_out_scale_level: Option<f32>,
    pub max_size: Option<u16>,
    pub min_size: Option<u16>,
    pub scale: Option<f32>,
}

bitflags::bitflags! {
    #[derive(Default, Serialize, Deserialize)]
    pub struct MarkerFlags: u8 {
        const IN_GAME_VISIBILITY = 0b00000001;
        const MAP_VISIBILITY = 0b00000010;
        const MINI_MAP_VISIBILITY = 0b00000100;
        const MINI_MAP_EDGE_HERD = 0b00001000;
        const MAP_SCALE = 0b00010000;
        const AUTO_TRIGGER = 0b00100000;
        const COUNT_DOWN = 0b01000000;
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
    pub toggle_cat: Option<CategoryID>,
}

#[serde_with::skip_serializing_none]
#[derive(Debug, Clone, Deserialize, Serialize, Copy, Validate)]
pub struct Achievement {
    pub id: NonZeroU16,
    pub bit: Option<u8>,
}

#[derive(Debug, Clone, Deserialize, Serialize, Validate)]
pub struct Info {
    #[validate(length(min = 1))]
    pub text: String,
    #[validate(range(min = "MIN_RANGE"))]
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
