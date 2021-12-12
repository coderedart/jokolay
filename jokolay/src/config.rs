use crate::{client::mm::MarkerConfig, core::window::OverlayWindowConfig};
use jokolink::MumbleConfig;
use serde::{Deserialize, Serialize};
use std::{io::Write, path::Path};

use egui::CtxRef;

use tokio::{fs::File, io::AsyncWriteExt};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct JokoConfig {
    pub overlay_window_config: OverlayWindowConfig,
    pub mumble_config: MumbleConfig,
    pub input_config: InputConfig,
    pub marker_config: MarkerConfig,
    pub auto_attach_to_gw2: bool,
    pub file_log_level: String,
    pub term_log_level: String,
    pub style: egui::Style,
}

impl Default for JokoConfig {
    fn default() -> Self {
        let file_log_level = "trace".to_string();
        let term_log_level = "debug".to_string();

        Self {
            overlay_window_config: Default::default(),
            mumble_config: MumbleConfig::default(),
            input_config: InputConfig::default(),
            marker_config: MarkerConfig::default(),
            auto_attach_to_gw2: true,
            file_log_level,
            term_log_level,
            style: egui::Style::default(),
        }
    }
}
impl JokoConfig {
    pub fn save_config(&self, path: &Path) -> anyhow::Result<()> {
        let mut config_file = std::fs::File::create(path)?;
        let config_string = serde_json::to_string_pretty(self)?;
        config_file.write_all(config_string.as_bytes())?;
        Ok(())
    }
}
pub async fn save_egui_memory(ctx: CtxRef, path: &Path) -> anyhow::Result<()> {
    let mut egui_cache = File::create(path).await?;
    let memory = ctx.memory().clone();
    let string = serde_json::to_string_pretty(&memory)?;
    egui_cache.write_all(string.as_bytes()).await?;
    Ok(())
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct InputConfig {
    // how much should we scroll. increase for more scrolling when you move scroll wheel, decrease for less.
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
