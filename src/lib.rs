use crate::core::{input::InputManager, mlink::MumbleManager, JokoCore};
use std::{
    path::PathBuf,
    time::{Duration, Instant},
};

use egui::{CtxRef, Pos2, RawInput, Rect, Visuals};
use glm::vec2;
use glow::HasContext;
use log::LevelFilter;

use tactical::localtypes::manager::MarkerManager;
use uuid::Uuid;

pub mod core;
pub mod gui;
pub mod tactical;
pub struct JokolayApp {
    pub core: JokoCore,
    pub ctx: CtxRef,
    pub mm: MarkerManager,
    state: EState,
}

#[derive(Debug, Clone, Default)]
pub struct EState {
    pub show_mumble_window: bool,
    pub show_marker_manager: bool,
}
impl JokolayApp {
    pub fn new() -> Self {
        let (mut core, ctx) = JokoCore::new();

        let mm = MarkerManager::new(&core.fm);
        JokolayApp {
            state: Default::default(),
            mm,
            ctx,
            core,
        }
    }

    pub fn run(mut self) -> anyhow::Result<()> {
        while !self.core.ow.should_close() {
            let t = self.tick();
            self.core.rr.draw_egui(
                t,
                vec2(
                    self.core.ow.config.framebuffer_width as f32,
                    self.core.ow.config.framebuffer_height as f32,
                ),
                &self.core.fm,
                self.ctx.clone(),
            );
            self.core.ow.swap_buffers();
        }
        Ok(())
    }
}
/// initializes global logging backend that is used by log macros
/// Takes in a filter for stdout/stderr, a filter for logfile and finally the path to logfile
pub fn log_init(
    term_filter: LevelFilter,
    file_filter: LevelFilter,
    file_path: PathBuf,
) -> anyhow::Result<()> {
    use simplelog::*;
    use std::fs::File;
    let config = ConfigBuilder::new()
        .set_location_level(LevelFilter::Error)
        .build();

    CombinedLogger::init(vec![
        TermLogger::new(term_filter, config, TerminalMode::Mixed, ColorChoice::Auto),
        WriteLogger::new(file_filter, Config::default(), File::create(file_path)?),
    ])?;
    Ok(())
}

#[macro_export]
macro_rules! gl_error {
    ($gl:expr) => {
        unsafe {
            let e = $gl.get_error();
            if e != glow::NO_ERROR {
                log::error!("glerror {} at {} {} {}", e, file!(), line!(), column!());
            }
        }
    };
}
