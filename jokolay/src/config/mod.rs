use anyhow::Context;
use glm::{I32Vec2, U32Vec2};
use jokolink::MumbleConfig;
use serde::{Deserialize, Serialize};
use std::io::{Read, Write};
use std::path::PathBuf;
use time::OffsetDateTime;

use std::fs::File;

pub struct ConfigManager {
    pub path: PathBuf,
    pub config: JokoConfig,
    pub last_saved: OffsetDateTime,
    pub needs_save: bool,
}
impl ConfigManager {
    pub fn new(path: PathBuf) -> anyhow::Result<Self> {
        if std::fs::metadata(&path).is_err() {
            let default_config = JokoConfig::default();
            std::fs::File::create(&path)?
                .write_all(
                    serde_json::to_string_pretty(&default_config)
                        .context("failed to serialize config default")?
                        .as_bytes(),
                )
                .context("failed to create default config file")?;
        }
        let mut config_src = String::new();
        std::fs::File::open(&path)
            .context("failed to open config file")?
            .read_to_string(&mut config_src)
            .context("failed to read config file")?;
        let config =
            serde_json::from_str(&config_src).context("failed to deserialize config from file")?;
        Ok(Self {
            path,
            config,
            last_saved: OffsetDateTime::now_utc(),
            needs_save: false,
        })
    }
    pub fn save_config(&mut self) -> anyhow::Result<()> {
        if self.needs_save {
            let mut config_file = File::create(&self.path)?;
            let config_string = serde_json::to_string_pretty(&self.config)?;
            config_file.write_all(config_string.as_bytes())?;
            self.needs_save = false;
            self.last_saved = OffsetDateTime::now_utc();
        }

        Ok(())
    }
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct JokoConfig {
    pub overlay_window_config: OverlayWindowConfig,
    pub mumble_config: MumbleConfig,
    pub input_config: InputConfig,
    pub auto_attach_to_gw2: bool,
    pub theme_name: String,
    pub log_level: String,
}

impl Default for JokoConfig {
    fn default() -> Self {
        Self {
            overlay_window_config: Default::default(),
            mumble_config: MumbleConfig::default(),
            input_config: InputConfig::default(),
            auto_attach_to_gw2: true,
            theme_name: "default".to_string(),
            log_level: "info".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct InputConfig {
    /// how much should we scroll. increase for more scrolling when you move scroll wheel, decrease for less.
    /// gets multiplied with pixels_per_point, so hidpi screens automatically scroll more pixels.
    pub scroll_power: f32,
}
impl InputConfig {
    pub const SCROLL_POWER: f32 = 20.0;
}

impl Default for InputConfig {
    fn default() -> Self {
        Self {
            scroll_power: Self::SCROLL_POWER,
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub enum VsyncMode {
    Immediate,
    Fifo,
}
impl Default for VsyncMode {
    fn default() -> Self {
        VsyncMode::Fifo
    }
}
/// Overlay Window Configuration. lightweight and Copy. so, we can pass this around to functions that need the window size/position
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct OverlayWindowConfig {
    /// window position in screen coordinates
    pub position: I32Vec2,
    /// framebuffer size in pixels
    pub size: U32Vec2,
    /// vsync mode
    pub vsync: VsyncMode,
}
impl OverlayWindowConfig {
    pub const FRAMEBUFFER_SIZE: U32Vec2 = U32Vec2::new(800, 600);
    pub const VSYNC: VsyncMode = VsyncMode::Fifo;
    pub const WINDOW_POSITION: I32Vec2 = I32Vec2::new(0, 0);
}
impl Default for OverlayWindowConfig {
    fn default() -> Self {
        Self {
            position: Self::WINDOW_POSITION,
            size: Self::FRAMEBUFFER_SIZE,
            vsync: Self::VSYNC,
        }
    }
}
