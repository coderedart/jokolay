use std::path::PathBuf;




use crate::config::JokoConfig;

use self::{
    input::InputManager,
    painter::Renderer,
    window::glfw_window::{OverlayWindow},
};

pub mod input;
pub mod painter;
pub mod window;

pub struct JokoCore {
    pub im: InputManager,
    pub rr: Renderer,
    pub ow: OverlayWindow,
}

impl JokoCore {
    pub fn new(joko_config: &mut JokoConfig, _assets_path: PathBuf) -> anyhow::Result<Self> {
        let config = joko_config.overlay_window_config;
        let (ow, events, glfw, gl) = OverlayWindow::create(config)?;
        let im = InputManager::new(events, glfw);
        // start setting up egui initial state
        // let ctx = CtxRef::default();
        // if let Ok(f) = fm.egui_cache_path.open_file().map_err(|e| {
        //     log::error!(
        //         "failed to open egui_cache path at {:?} due to error: {:?}",
        //         &fm.egui_cache_path,
        //         &e
        //     );
        //     e
        // }) {
        //     if let Ok(memory) = serde_json::from_reader(f).map_err(|e| {
        //         log::error!(
        //             "failed to parse memory from file {:?} due ot error {:?}",
        //             &fm.egui_cache_path,
        //             &e
        //         );
        //         e
        //     }) {
        //         *ctx.memory() = memory;
        //     }
        // }

        let rr = Renderer::new(gl.clone());
        Ok(Self { im, rr, ow })
    }
    pub fn tick(&mut self) {
        
    }
    pub fn run(&mut self) -> anyhow::Result<()> {
        todo!()
    }
}

pub enum InputCommand {
    SetClipBoard(String),
}

pub enum WindowCommand {
    Resize(u32, u32),
    Repos(u32, u32),
    Transparent(bool),
    Passthrough(bool),
    Decorated(bool),
    AlwaysOnTop(bool),
    ShouldClose(bool),
    SwapInterval(u32),
    SetTransientFor(u32),
}

pub enum TextureCommand {
    Upload {
        pixels: Vec<u8>,
        x_offset: i32,
        y_offset: i32,
        z_offset: i32,
        width: i32,
        height: i32,
    },
    BumpTextureArraySize,
    Reset,
}

pub enum RenderCommand {
    Draw,
}
pub enum GlobalCommand {}
