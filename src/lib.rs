use std::{
    path::PathBuf,
    time::{Duration, Instant},
};

use egui::{CtxRef, Pos2, RawInput, Rect, Visuals};
use glm::vec2;
use glow::HasContext;
use log::LevelFilter;
use tokio::{runtime::Handle, sync::oneshot::channel};

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
    pub handle: Handle,
    pub mumble_manager: MumbleManager,
    pub marker_manager: MarkerManager,
    pub file_manager: FileManager,
    pub painter: Painter,
    pub overlay_window: GlfwWindow,
    state: EState,
    shutdown_tx: tokio::sync::oneshot::Sender<u32>,
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
        // we don't do much, but we can use this async handle to spawn tasks in future
        let (shutdown_tx, shutdown_rx) = channel::<u32>();
        let (handle_tx, handle_rx) = std::sync::mpsc::channel();

        std::thread::spawn(move || {
            let rt = tokio::runtime::Runtime::new().unwrap();

            let hndl = rt.handle();
            handle_tx.send(hndl.clone()).unwrap();
            rt.block_on(async {
                shutdown_rx.await.unwrap();
            })
        });
        let handle = handle_rx.recv().unwrap();
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
            handle,
            state,
            shutdown_tx,
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

            self.mumble_manager.update();

            self.input_manager
                .process_events(&mut self.overlay_window, &mut self.state.input);
            gl_error!(gl);
            unsafe {
                gl.disable(glow::SCISSOR_TEST);
                gl.clear_color(0.0, 0.0, 0.0, 0.0);
                gl.clear(glow::COLOR_BUFFER_BIT | glow::DEPTH_BUFFER_BIT);
            }
            let meshes = self.tick();
            self.painter.draw_egui(
                meshes,
                vec2(
                    self.overlay_window.window_size.0 as f32,
                    self.overlay_window.window_size.1 as f32,
                ),
                &self.file_manager,
            );
            if self.marker_manager.draw_markers {
                self.painter.draw_markers(
                    &mut self.marker_manager,
                    self.mumble_manager.get_link(),
                    &self.file_manager,
                );
                self.painter.draw_trails(
                    &mut self.marker_manager,
                    self.mumble_manager.get_link(),
                    &self.file_manager,
                );
            }
            // ending loop timer
            average_egui = (average_egui + et.elapsed()) / 2;
            // start draw call timer
            let dt = Instant::now();
            self.overlay_window.redraw_request();
            average_draw_call = (average_draw_call + dt.elapsed()) / 2;
        }
        self.shutdown_tx.send(0).unwrap();
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

    CombinedLogger::init(vec![
        TermLogger::new(
            term_filter,
            Config::default(),
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
