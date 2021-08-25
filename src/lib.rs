use std::{
    path::PathBuf,
    time::{Duration, Instant},
};

use egui::{CtxRef, Pos2, RawInput, Rect, Visuals};
use glm::vec2;
use glow::HasContext;
use log::LevelFilter;

use uuid::Uuid;
use window::glfw_window::GlfwWindow;

use crate::{
    fm::FileManager, input::InputManager, mlink::MumbleManager, painter::Painter,
    tactical::localtypes::manager::MarkerManager,
};

pub mod fm;
pub mod gui;
pub mod input;
pub mod mlink;
pub mod painter;
pub mod tactical;
pub mod window;
pub struct JokolayApp {
    pub ctx: CtxRef,
    pub input_manager: InputManager,
    pub mumble_manager: MumbleManager,
    pub marker_manager: MarkerManager,
    pub file_manager: FileManager,
    pub painter: Painter,
    pub overlay_window: GlfwWindow,
    state: EState,
}

#[derive(Debug, Clone, Default)]
pub struct EState {
    pub show_mumble_window: bool,
    pub show_marker_manager: bool,
    pub input: RawInput,
}
impl JokolayApp {
    pub fn new() -> anyhow::Result<Self> {
        let (overlay_window, events, glfw) = GlfwWindow::create(true)?;
        let overlay_window = overlay_window;
        let gl = overlay_window.get_gl_context();

        unsafe {
            let e = gl.get_error();
            if e != glow::NO_ERROR {
                println!("glerror {} at {} {} {}", e, file!(), line!(), column!());
            }
            gl.enable(glow::MULTISAMPLE);
            gl.enable(glow::BLEND);
        }
  
        let input_manager = InputManager::new(events, glfw);
        let mumble_manager = MumbleManager::new("MumbleLink").unwrap();
        // start setting up egui initial state
        let mut ctx = CtxRef::default();
        let (width, height) = overlay_window.window_size;

        let mut input = RawInput::default();
        input.screen_rect = Some(Rect::from_two_pos(
            Pos2::new(0.0, 0.0),
            Pos2::new(width as f32, height as f32),
        ));
        input.pixels_per_point = Some(1.0);
        let mut visuals = Visuals::dark();

        visuals.window_shadow.extrusion = 0.0;
        visuals.window_corner_radius = 0.0;
        ctx.set_visuals(visuals);

        ctx.begin_frame(input.take());

        let state = EState {
            show_mumble_window: false,
            show_marker_manager: false,
            input,
        };
        let t = ctx.texture();
        let painter = Painter::new(gl.clone(), t);
        let _ = ctx.end_frame();
        let file_manager = FileManager::new();
        let marker_manager = MarkerManager::new(&file_manager);
        Ok(JokolayApp {
            overlay_window,
            ctx,
            input_manager,
            mumble_manager,
            marker_manager,
            file_manager,
            painter,
            state,
        })
    }

    pub fn run(mut self) -> anyhow::Result<()> {
        let gl = self.overlay_window.get_gl_context();

        //fps counter
        let mut fps = 0;
        let mut timer = Instant::now();
        let mut average_egui = Duration::default();
        let mut average_draw_call = Duration::default();
        loop {
            // starting loop timer
            let et = Instant::now();

            if self.overlay_window.should_close() {
                break;
            }
            if timer.elapsed() > Duration::from_secs(1) {
                dbg!(fps, average_egui, average_draw_call);
                fps = 0;
                timer = Instant::now();
            }
            fps += 1;

            
            gl_error!(gl);
            unsafe {
                gl.disable(glow::SCISSOR_TEST);
                gl.clear_color(0.0, 0.0, 0.0, 0.0);
                gl.clear(glow::COLOR_BUFFER_BIT | glow::DEPTH_BUFFER_BIT);
            }
            let meshes = self.tick();
            // ending loop timer
            average_egui = (average_egui + et.elapsed()) / 2;
            // start draw call timer
            let dt = Instant::now();
            
            if self.marker_manager.draw_markers {
                self.painter.draw_markers(
                    &mut self.marker_manager,
                    self.mumble_manager.get_link(),
                    &self.file_manager,
                    &self.overlay_window
                );
                self.painter.draw_trails(
                    &mut self.marker_manager,
                    self.mumble_manager.get_link(),
                    &self.file_manager,
                );
            }
            self.painter.draw_egui(
                meshes,
                vec2(
                    self.overlay_window.window_size.0 as f32,
                    self.overlay_window.window_size.1 as f32,
                ),
                &self.file_manager,
            );
            average_draw_call = (average_draw_call + dt.elapsed()) / 2;

            self.overlay_window.redraw_request();
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
    let config = ConfigBuilder::new().set_location_level(LevelFilter::Error).build();

    CombinedLogger::init(vec![
        TermLogger::new(
            term_filter,
            config,
            TerminalMode::Mixed,
            ColorChoice::Auto,
        ),
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
