use std::{cell::RefCell, path::PathBuf, rc::Rc, time::Instant};

use egui::{Pos2, RawInput, Rect};
use glow::HasContext;
use log::LevelFilter;
use tokio::{runtime::Handle, sync::oneshot::channel};
use window::glfw_window::GlfwWindow;

use crate::{gui::eapp::EguiApp, input::InputManager, mlink::MumbleManager};

pub mod gltypes;
pub mod gui;
pub mod input;
pub mod mlink;
pub mod tactical;
pub mod window;

pub struct JokolayApp {
    pub app: EguiApp,
    pub overlay_window: GlfwWindow,
    pub input_manager: InputManager,
    pub handle: Handle,
    pub mumble_manager: MumbleManager,
    shutdown_tx: tokio::sync::oneshot::Sender<u32>,
}

impl JokolayApp{
    pub fn new() -> anyhow::Result<Self> {
        let (overlay_window, events, glfw) = GlfwWindow::create(true, true, false, true)?;
        let overlay_window = overlay_window;
        let gl = overlay_window.get_gl_context();

        let app = EguiApp::new(gl.clone());
        unsafe {
            let e = gl.get_error();
            if e != glow::NO_ERROR {
                println!("glerror {} at {} {} {}", e, file!(), line!(), column!());
            }
        }
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
        Ok(JokolayApp {
            overlay_window,
            app,
            input_manager,
            mumble_manager,
            handle,
            shutdown_tx,
        })
    }

    pub fn run(mut self) -> anyhow::Result<()> {
        let gl = self.overlay_window.get_gl_context();
        let (width, height) = self.overlay_window.window_size;

        let mut input = RawInput::default();
        input.screen_rect = Some(Rect::from_two_pos(
            Pos2::new(0.0, 0.0),
            Pos2::new(width as f32, height as f32),
        ));
        input.pixels_per_point = Some(1.0);
        loop {
            if self.overlay_window.should_close() {
                break;
            }

            self.mumble_manager.update();
            // let t = Instant::now();
            // bench = (bench + t.elapsed())/2 ;

            self.input_manager
                .process_events(&mut self.overlay_window, &mut input);
            gl_error!(gl);
            unsafe {
                gl.disable(glow::SCISSOR_TEST);
                gl.clear_color(0.0, 0.0, 0.0, 0.0);
                gl.clear(glow::COLOR_BUFFER_BIT | glow::DEPTH_BUFFER_BIT);
            }
            self.app
                .update(
                    &mut self.overlay_window,
                    self.mumble_manager.get_link(),
                    &mut input,
                )
                .unwrap();

            self.overlay_window.redraw_request();
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
