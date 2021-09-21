use std::path::PathBuf;

use egui::{CtxRef, Pos2, RawInput, Rect, Visuals};
use log::trace;
use x11rb::{
    protocol::xproto::{
        change_property, get_atom_name, get_property, intern_atom, reparent_window, Atom, AtomEnum,
        ConnectionExt, PropMode,
    },
    rust_connection::RustConnection,
};

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

#[derive(Debug)]
pub struct JokoConfig {
    overlay_window_config: OverlayWindowConfig,
    mumble_config: MumbleConfig,
    assets_folder_path: PathBuf,
}
impl Default for JokoConfig {
    fn default() -> Self {
        let mut assets_folder_path = std::env::current_dir().unwrap();
        assets_folder_path.push("assets");

        Self {
            overlay_window_config: Default::default(),
            mumble_config: Default::default(),
            assets_folder_path,
        }
    }
}
impl JokoCore {
    pub fn new() -> (Self, CtxRef) {
        let joko_config = JokoConfig::default();
        let mut mumble_src = MumbleSource::new(&joko_config.mumble_config.link_name);

        let config = joko_config.overlay_window_config;
        let (ow, events, glfw, gl) = OverlayWindow::create(config, &mut mumble_src).unwrap();
        let mbm = MumbleManager::new(mumble_src).unwrap();

        let im = InputManager::new(events, glfw);
        // start setting up egui initial state
        let ctx = CtxRef::default();

        let mut visuals = Visuals::dark();
        visuals.window_shadow.extrusion = 0.0;
        visuals.window_corner_radius = 0.0;
        ctx.set_visuals(visuals);

        let rr = Renderer::new(gl.clone());
        let fm = FileManager::new(joko_config.assets_folder_path);
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
