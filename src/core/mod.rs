use std::path::PathBuf;

use egui::CtxRef;
use serde::{Deserialize, Serialize};

use crate::core::mlink::{MumbleConfig};

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
            file_log_level,
            term_log_level,
        }
    }
}

impl JokoCore {
    pub fn new(joko_config: &mut JokoConfig, assets_path: PathBuf) -> (Self, CtxRef) {
        let mut mbm = MumbleManager::new(joko_config.mumble_config.clone()).unwrap();
        while mbm.get_link().ui_tick == 0 {
            mbm.tick().unwrap()
        }
        let config = joko_config.overlay_window_config;
        let (ow, events, glfw, gl) =
            OverlayWindow::create(config, mbm.src.as_mut().unwrap()).unwrap();
        let fm = FileManager::new(assets_path);
        let im = InputManager::new(events, glfw);
        // start setting up egui initial state
        let ctx = CtxRef::default();
        if let Ok(f) = fm.egui_cache_path.open_file().map_err(|e| {
            log::error!(
                "failed to open egui_cache path at {:?} due to error: {:?}",
                &fm.egui_cache_path,
                &e
            );
            e
        }) {
            if let Ok(memory) = serde_json::from_reader(f).map_err(|e| {
                log::error!(
                    "failed to parse memory from file {:?} due ot error {:?}",
                    &fm.egui_cache_path,
                    &e
                );
                e
            }) {
                *ctx.memory() = memory;
            }
        }

        let rr = Renderer::new(gl.clone());
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
        self.mbm
            .tick()
            .map_err(|e| {
                log::error!("mumble might not be valid");
                e
            })
            .unwrap();
        self.ow.tick(ctx);
    }
}
