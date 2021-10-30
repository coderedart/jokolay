use crate::core::window::glfw_window::OverlayWindowConfig;
use jokolink::MumbleConfig;
use serde::{Deserialize, Serialize};

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
