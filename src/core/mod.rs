use std::path::PathBuf;

use egui::CtxRef;
use serde::{Deserialize, Serialize};

use crate::core::mlink::{MumbleConfig, MumbleSource};

use self::{
    fm::FileManager,
    input::InputManager,
    mlink::MumbleManager,
    painter::Renderer,
    window::glfw_window::{OverlayWindow, OverlayWindowConfig},
};

pub mod fm;
pub mod input;
pub mod mlink;
pub mod painter;
pub mod window;

pub struct JokoCore {
    pub im: InputManager,
    pub mbm: MumbleManager,
    pub fm: FileManager,
    pub rr: Renderer,
    pub ow: OverlayWindow,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JokoConfig {
    pub overlay_window_config: OverlayWindowConfig,
    pub mumble_config: MumbleConfig,
    pub assets_folder_path: PathBuf,
    pub log_file_path: PathBuf,
    pub file_log_level: String,
    pub term_log_level: String,
    pub egui_cache_path: PathBuf,
}
impl Default for JokoConfig {
    fn default() -> Self {
        let current_dir = std::env::current_dir().unwrap();
        let assets_folder_path = current_dir.join("assets");
        let log_file_path = current_dir.join("jokolay.log");
        let file_log_level = "trace".to_string();
        let term_log_level = "debug".to_string();
        let egui_cache_path = current_dir.join("egui_cache.json");

        Self {
            overlay_window_config: Default::default(),
            mumble_config: Default::default(),
            assets_folder_path,
            log_file_path,
            file_log_level,
            term_log_level,
            egui_cache_path,
        }
    }
}

impl JokoCore {
    pub fn new(joko_config: &mut JokoConfig) -> (Self, CtxRef) {
        let mut mumble_src = MumbleSource::new(&joko_config.mumble_config.link_name);

        let config = joko_config.overlay_window_config;
        let (ow, events, glfw, gl) = OverlayWindow::create(config, &mut mumble_src).unwrap();
        let mbm = MumbleManager::new(mumble_src).unwrap();

        let im = InputManager::new(events, glfw);
        // start setting up egui initial state
        let ctx = CtxRef::default();
        if let Ok(f) = std::fs::File::open(&joko_config.egui_cache_path).map_err(|e| {
            log::error!(
                "failed to open egui_cache path at {:?} due to error: {:?}",
                &joko_config.egui_cache_path,
                &e
            );
            e
        }) {
            if let Ok(memory) = serde_json::from_reader(f).map_err(|e| {
                log::error!(
                    "failed to parse memory from file {:?} due ot error {:?}",
                    &joko_config.egui_cache_path,
                    &e
                );
                e
            }) {
                *ctx.memory() = memory;
            }
        }

        let rr = Renderer::new(gl.clone());
        let fm = FileManager::new(joko_config.assets_folder_path.clone());
        (
            Self {
                im,
                mbm,
                fm,
                rr,
                ow,
            },
            ctx,
        )
    }
    pub fn tick(&mut self, ctx: &CtxRef) {
        self.mbm.tick();
        self.ow.tick(ctx);
    }
}
