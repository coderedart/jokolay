pub mod joko_script;
pub mod jokoapi;
pub mod jokolink;

use std::path::PathBuf;

use cap_std::{ambient_authority, fs::Dir};
use egui_backend::{
    egui::{self, Grid, Ui},
    raw_window_handle::HasRawWindowHandle,
    BackendConfig, GfxBackend, UserApp, WindowBackend,
};
use egui_window_glfw_passthrough::{GlfwBackend, GlfwConfig};
use miette::{miette, Context, IntoDiagnostic, Result};

use jmf::manager::MarkerManager;
use joko_render::{JokoRenderer, MARKER_MAX_VISIBILITY_DISTANCE};
use jokolink::{MumbleManager, WindowDimensions};

use tracing::{info, warn};

pub struct Jokolay {
    pub last_check: f64,
    pub fps: u32,
    pub frame_count: u64,
    pub frame_reset_seconds_timestamp: u64,
    pub jdir: Dir,
    pub jpath: PathBuf,
    pub mumble: Option<MumbleManager>,
    pub marker_manager: Option<MarkerManager>,
    pub window_dimensions: WindowDimensions,
    pub joko_renderer: JokoRenderer,
    pub egui_context: egui::Context,
    pub window_backend: GlfwBackend,
}

impl Jokolay {
    fn new(
        mut window_backend: GlfwBackend,
        joko_renderer: JokoRenderer,
        jpath: PathBuf,
        jdir: Dir,
    ) -> Self {
        let mumble = MumbleManager::new("MumbleLink", window_backend.window.raw_window_handle())
            .map_err(|e| {
                warn!("error creating Mumble Manager: {}", e);
            })
            .map(|mut mumble| {
                match mumble.get_latest_window_dimensions() {
                    Ok(wd) => {
                        let (x, y) = window_backend.window.get_pos();
                        let (width, height) = window_backend.window.get_size();
                        if x != wd.x
                            || y != wd.y
                            || width as u32 != wd.width
                            || height as u32 != wd.height
                        {
                            window_backend.window.set_pos(wd.x, wd.y);
                            window_backend
                                .window
                                .set_size(wd.width as i32, wd.height as i32);
                        }
                    }
                    Err(e) => tracing::error!("failed ot get window dimensions: {e}"),
                }
                mumble
            })
            .ok();
        let marker_manager = MarkerManager::new(&jdir)
            .map_err(|e| {
                warn!("error creating Marker Manager: {}", e);
            })
            .ok();

        Self {
            mumble,
            marker_manager,
            last_check: 0.0,
            joko_renderer,
            egui_context: Default::default(),
            frame_count: 0,
            frame_reset_seconds_timestamp: 0,
            fps: 0,
            window_dimensions: WindowDimensions::default(),
            window_backend,
            jdir,
            jpath,
        }
    }
}
impl UserApp for Jokolay {
    fn gui_run(&mut self) {
        let Self {
            last_check,
            fps,
            frame_count,
            frame_reset_seconds_timestamp,
            mumble,
            marker_manager,
            window_dimensions,
            joko_renderer,
            egui_context,
            window_backend,
            jdir: _,
            jpath: _,
        } = self;
        // for ev in window_backend.frame_events.iter() {
        //     match ev {
        //         egui_window_glfw_passthrough::glfw::WindowEvent::Focus(_)
        //         | egui_window_glfw_passthrough::glfw::WindowEvent::Key(_, _, _, _) => {
        //             dbg!(ev);
        //         }
        //         _ => {}
        //     }
        // }
        let latest_time = window_backend.glfw.get_time();
        *frame_count += 1;
        if latest_time - *frame_reset_seconds_timestamp as f64 > 1.0 {
            *fps = *frame_count as u32;
            *frame_count = 0;
            *frame_reset_seconds_timestamp = latest_time as u64;
        }
        if let Some(mumble) = mumble {
            let _ = mumble.tick();
        }
        egui_context.request_repaint();
        let cursor_position = egui_context.pointer_latest_pos();
        egui::Window::new("egui window")
            .default_width(300.0)
            .show(egui_context, |ui| {
                ui.label(&format!("cursor position: {cursor_position:?}"));
                ui.label(&format!("fps: {}", *fps));
                let mut is_passthrough = window_backend.window.is_mouse_passthrough();
                ui.checkbox(&mut is_passthrough, "is window passthrough?");
                if let Some(mumble) = mumble {
                    if let Some(link) = mumble.get_mumble_link() {
                        mumble_ui(ui, link);
                    }
                }
                ui.label(format!(
                    "number of markers drawn: {}",
                    joko_renderer.markers.len()
                ));
            });
        if let Some(marker_manager) = marker_manager {
            marker_manager.tick(egui_context, latest_time);
        }
        if let Some(mumble) = mumble {
            if let Some(link) = mumble.get_mumble_link() {
                if let Some(_marker_manager) = marker_manager {
                    // marker_manager.render(link.context.map_id as u16, joko_renderer);
                }
                let ratio = window_backend.framebuffer_size_physical[0] as f32
                    / window_backend.framebuffer_size_physical[1] as f32;
                let center = link.f_camera_position + link.f_camera_front;
                let v = glam::Mat4::look_at_lh(
                    link.f_camera_position,
                    center,
                    glam::vec3(0.0, 1.0, 0.0),
                );
                let p = glam::Mat4::perspective_lh(
                    link.identity.fov,
                    ratio,
                    1.0,
                    MARKER_MAX_VISIBILITY_DISTANCE,
                );
                joko_renderer.set_mvp(p * v);
                joko_renderer.camera_position = link.f_camera_position;
                joko_renderer.mvp = p * v;
                joko_renderer.player_position = link.f_avatar_position;
            }
        }
        // if it doesn't require either keyboard or pointer, set passthrough to true
        window_backend.window.set_mouse_passthrough(
            !(egui_context.wants_keyboard_input() || egui_context.wants_pointer_input()),
        );
        if latest_time - *last_check > 10. {
            *last_check = latest_time;
            if let Some(mumble) = mumble {
                if let Ok(wd) = mumble.get_latest_window_dimensions() {
                    *window_dimensions = wd;
                    let (x, y) = window_backend.window.get_pos();
                    let (width, height) = window_backend.window.get_size();
                    if x != wd.x
                        || y != wd.y
                        || width as u32 != wd.width
                        || height as u32 != wd.height
                    {
                        info!("resizing/repositioning our window from {x},{y},{width},{height} to match gw2 window dimensions: {wd:?}");
                        window_backend.window.set_pos(wd.x, wd.y);
                        window_backend
                            .window
                            .set_size(wd.width as i32, wd.height as i32);
                    }
                }
            }
        }
    }

    type UserGfxBackend = JokoRenderer;

    type UserWindowBackend = GlfwBackend;

    fn get_all(
        &mut self,
    ) -> (
        &mut Self::UserWindowBackend,
        &mut Self::UserGfxBackend,
        &egui::Context,
    ) {
        (
            &mut self.window_backend,
            &mut self.joko_renderer,
            &self.egui_context,
        )
    }
}

pub async fn start_jokolay() {
    install_miette_panic_hooks().unwrap();
    let (jokolay_dir_path, jdir) = get_jokolay_dir().unwrap();
    let _log_file_flush_guard = install_tracing(&jdir).unwrap();
    info!("using {jokolay_dir_path:?} as the jokolay data directory");

    let mut glfw_backend = GlfwBackend::new(
        GlfwConfig {
            glfw_callback: Box::new(|glfw_context| {
                glfw_context.window_hint(
                    egui_window_glfw_passthrough::glfw::WindowHint::SRgbCapable(true),
                );
                glfw_context.window_hint(egui_window_glfw_passthrough::glfw::WindowHint::Floating(
                    true,
                ));
            }),
            ..Default::default()
        },
        BackendConfig {
            transparent: Some(true),
            is_opengl: false,
            ..Default::default()
        },
    );

    let joko_renderer = JokoRenderer::new(&mut glfw_backend, Default::default());
    // remove decorations
    glfw_backend.window.set_decorated(false);
    let jokolay = Jokolay::new(glfw_backend, joko_renderer, jokolay_dir_path, jdir);
    <Jokolay as UserApp>::UserWindowBackend::run_event_loop(jokolay);
}

fn mumble_ui(ui: &mut Ui, link: &jokolink::MumbleLink) {
    Grid::new("link grid").num_columns(2).show(ui, |ui| {
        ui.label("ui tick: ");
        ui.label(format!("{}", link.ui_tick));
        ui.end_row();
        ui.label("character: ");
        ui.label(&link.identity.name);
        ui.end_row();
        ui.label("map: ");
        ui.label(format!("{}", link.context.map_id));
        ui.end_row();
        ui.label(format!("pid: {}", link.context.process_id));
        // ui.label("player position");
        // ui.label(format!(
        //     "{:.2} {:.2} {:.2}",
        //     link.f_avatar_position.x, link.f_avatar_position.y, link.f_avatar_position.z
        // ));
        ui.end_row();
    });
}

/// Jokolay Configuration
/// We will read a path from env `JOKOLAY_DATA_DIR` or create a folder at data_local_dir/jokolay, where data_local_dir is platform specific
/// Inside this directory, we will store all of jokolay's data like configuration files, themes, logs etc..
fn get_jokolay_dir() -> Result<(PathBuf, cap_std::fs::Dir)> {
    let authoratah = ambient_authority();
    let jokolay_data_local_dir_path = if let Some(env_dir) = std::env::var("JOKOLAY_DATA_DIR").ok()
    {
        match PathBuf::try_from(&env_dir) {
            Ok(jokolay_dir) => jokolay_dir,
            Err(e) => return Err(miette!("failed to parse JOKOLAY_DATA_DIR: {e}")),
        }
    } else {
        match directories_next::ProjectDirs::from("com.jokolay", "", "jokolay") {
            Some(pd) => pd.data_local_dir().to_path_buf(),
            None => return Err(miette!("getting project dirs failed for some reason")),
        }
    };
    if jokolay_data_local_dir_path.to_str().is_none() {
        return Err(miette!(
            "jokolay data dir is not utf-8: {jokolay_data_local_dir_path:?}"
        ));
    }
    if let Err(e) =
        cap_std::fs::Dir::create_ambient_dir_all(&jokolay_data_local_dir_path, authoratah)
    {
        return Err(miette!(
            "failed to create jokolay directory at {jokolay_data_local_dir_path:?} due to error: {e}"
        ));
    }
    let jdir = match Dir::open_ambient_dir(&jokolay_data_local_dir_path, authoratah) {
        Ok(jdir) => jdir,
        Err(e) => {
            return Err(miette!(
                "failed to open jokolay data dir at {jokolay_data_local_dir_path:?} due to {e}"
            ))
        }
    };

    Ok((jokolay_data_local_dir_path, jdir))
}

fn install_tracing(
    jokolay_dir: &Dir,
) -> miette::Result<tracing_appender::non_blocking::WorkerGuard> {
    use tracing_error::ErrorLayer;
    use tracing_subscriber::prelude::*;
    use tracing_subscriber::{fmt, EnvFilter};
    // get the log level
    let filter_layer = EnvFilter::try_from_env("JOKOLAY_LOG")
        .or_else(|_| EnvFilter::try_new("info,wgpu=warn,naga=warn"))
        .unwrap();
    // create log file in the data dir. This will also serve as a check that the directory is "writeable" by us
    let writer = std::io::BufWriter::new(
        jokolay_dir
            .create("jokolay.log")
            .into_diagnostic()
            .wrap_err("failed to create jokolay.log file")?,
    );
    let (nb, guard) = tracing_appender::non_blocking(writer);
    let fmt_layer = fmt::layer().with_target(false).with_writer(nb);

    tracing_subscriber::registry()
        .with(filter_layer)
        .with(fmt_layer)
        .with(ErrorLayer::default())
        .init();
    Ok(guard)
}
/// code stolen from miette::set_panic_hook
fn install_miette_panic_hooks() -> Result<()> {
    miette::set_hook(Box::new(|diagnostic| {
        let handler = Box::new(miette::NarratableReportHandler::new());
        let mut panic_report = String::new();
        if let Err(e) = handler.render_report(&mut panic_report, diagnostic) {
            tracing::error!("failed to render report: {e}");
        }
        tracing::error!("crashing: {:#?}", &panic_report);
        handler
    }))
    .wrap_err("failed to install miette hook")?;

    #[derive(Debug, thiserror::Error, miette::Diagnostic)]
    #[error("{0}{}", Panic::backtrace())]
    #[diagnostic(help("set the `RUST_BACKTRACE=1` environment variable to display a backtrace."))]
    struct Panic(String);
    impl Panic {
        fn backtrace() -> String {
            use std::fmt::Write;
            if let Ok(var) = std::env::var("RUST_BACKTRACE") {
                if !var.is_empty() && var != "0" {
                    const HEX_WIDTH: usize = std::mem::size_of::<usize>() + 2;
                    // Padding for next lines after frame's address
                    const NEXT_SYMBOL_PADDING: usize = HEX_WIDTH + 6;
                    let mut backtrace = String::new();
                    let trace = backtrace::Backtrace::new();
                    let frames = backtrace_ext::short_frames_strict(&trace).enumerate();
                    for (idx, (frame, sub_frames)) in frames {
                        let ip = frame.ip();
                        let _ = write!(backtrace, "\n{:4}: {:2$?}", idx, ip, HEX_WIDTH);

                        let symbols = frame.symbols();
                        if symbols.is_empty() {
                            let _ = write!(backtrace, " - <unresolved>");
                            continue;
                        }

                        for (idx, symbol) in symbols[sub_frames].iter().enumerate() {
                            // Print symbols from this address,
                            // if there are several addresses
                            // we need to put it on next line
                            if idx != 0 {
                                let _ = write!(backtrace, "\n{:1$}", "", NEXT_SYMBOL_PADDING);
                            }

                            if let Some(name) = symbol.name() {
                                let _ = write!(backtrace, " - {}", name);
                            } else {
                                let _ = write!(backtrace, " - <unknown>");
                            }

                            // See if there is debug information with file name and line
                            if let (Some(file), Some(line)) = (symbol.filename(), symbol.lineno()) {
                                let _ = write!(
                                    backtrace,
                                    "\n{:3$}at {}:{}",
                                    "",
                                    file.display(),
                                    line,
                                    NEXT_SYMBOL_PADDING
                                );
                            }
                        }
                    }
                    return backtrace;
                }
            }
            "".into()
        }
    }

    std::panic::set_hook(Box::new(|panic_info| {
        // code stolen from miette::set_panic_hook
        let mut message = "Something went wrong".to_string();
        let payload = panic_info.payload();
        if let Some(msg) = payload.downcast_ref::<&str>() {
            message = msg.to_string();
        }
        if let Some(msg) = payload.downcast_ref::<String>() {
            message = msg.clone();
        }
        let mut report: miette::Result<()> = Err(Panic(message).into());
        if let Some(loc) = panic_info.location() {
            report = report
                .with_context(|| format!("at {}:{}:{}", loc.file(), loc.line(), loc.column()));
        }
        if let Err(err) = report.with_context(|| "Main thread panicked.".to_string()) {
            eprintln!("Error: {:?}", err);
            tracing::error!("crashing: {:?}", &err);
            if let Err(e) = notify_rust::Notification::new()
                .appname("Jokolay")
                .body(&format!("{:?}", &err))
                .summary("Jokolay crashed")
                .timeout(0)
                .finalize()
                .show()
            {
                tracing::error!("failed to display notification");
                eprintln!("failed to display notification, {e:?}");
            }
        }
    }));
    Ok(())
}
