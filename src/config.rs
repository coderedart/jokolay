use crate::core::window::OverlayWindowConfig;
use jokolink::MumbleConfig;
use serde::{Deserialize, Serialize};
use std::path::Path;

use egui::CtxRef;

use tokio::{fs::File, io::AsyncWriteExt};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JokoConfig {
    pub overlay_window_config: OverlayWindowConfig,
    pub mumble_config: MumbleConfig,
    pub auto_attach_to_gw2: bool,
    pub file_log_level: String,
    pub term_log_level: String,
}

impl Default for JokoConfig {
    fn default() -> Self {
        let file_log_level = "trace".to_string();
        let term_log_level = "debug".to_string();

        Self {
            overlay_window_config: Default::default(),
            mumble_config: MumbleConfig::default(),
            auto_attach_to_gw2: true,
            file_log_level,
            term_log_level,
        }
    }
}

pub async fn save_egui_memory(ctx: CtxRef, path: &Path) -> anyhow::Result<()> {
    let mut egui_cache = File::create(path).await?;
    let memory = ctx.memory().clone();
    let string = serde_json::to_string_pretty(&memory)?;
    egui_cache.write_all(string.as_bytes()).await?;
    Ok(())
}

pub async fn save_config(config: &JokoConfig, path: &Path) -> anyhow::Result<()> {
    let mut config_file = File::create(path).await?;
    let config_string = serde_json::to_string_pretty(config)?;
    config_file.write_all(config_string.as_bytes()).await?;
    Ok(())
}
