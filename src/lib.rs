use std::{
    cell::RefCell,
    collections::BTreeMap,
    net::UdpSocket,
    rc::Rc,
    time::{Duration, Instant},
};

use egui::{epaint::Shadow, Color32, Event};
use glc::{eglfw::{eguiapp::EguiApp, scene::EguiScene}, renderer::texture::Texture};
use glfw::Context as _;
use glow::HasContext;
use gw::{category::MarkerCategory, marker::Marker, trail::Trail};
use mlink::MumbleCache;
use nalgebra_glm::make_vec2;
use window::OverlayWindow;

use crate::mlink::GetMLMode;

pub mod glc;
pub mod gw;
pub mod mlink;
pub mod window;

pub struct JokolayApp {
    // pub marker_categories: BTreeMap<String, MarkerCategory>,
    // pub markers: BTreeMap<u32, Vec<Marker>>,
    // pub trails: BTreeMap<u32, Vec<Trail>>,
    pub markers_overlay_show: bool,
    pub egui_app: Rc<EguiApp>,
    pub overlay_window: Rc<OverlayWindow>,
}

impl JokolayApp {
    pub fn new() -> anyhow::Result<Self> {
        let overlay_window = Rc::new( OverlayWindow::init()?);
        let gl = overlay_window.gl.clone();
        
        
        // let (marker_categories, markers, trails) = load_markers();
        let egui_app = Rc::new(EguiApp::new(gl.clone(), overlay_window.clone()));

        unsafe {
            let e = gl.get_error();
            if e != glow::NO_ERROR {
                println!("glerror {} at {} {} {}", e, file!(), line!(), column!());
            }
        }

        Ok(JokolayApp {
            overlay_window,
            egui_app,
            markers_overlay_show: false,
            // marker_categories: BTreeMap::new(),
            // markers: BTreeMap::new(),
            // trails: BTreeMap::new(),
            
        })
    }
    pub fn run(&mut self) -> anyhow::Result<()> {
        let gl = self.overlay_window.gl.clone();
        let egui_app = self.egui_app.clone();
        let overlay_window = self.overlay_window.clone();

        unsafe {
            gl.active_texture(glow::TEXTURE0);
        }

        let mut previous = std::time::Instant::now();
        let mut fps = 0;
       
       loop {
            if overlay_window.window.borrow().should_close() {
                break;
            }
            fps += 1;
            if previous.elapsed() > std::time::Duration::from_secs(1) {
                previous = std::time::Instant::now();
                // dbg!(fps);
                fps = 0;
            }
            unsafe {
                let e = gl.get_error();
                if e != glow::NO_ERROR {
                    println!("glerror {} at {} {} {}", e, file!(), line!(), column!());
                }
            }

            if self.overlay_window.process_events() {
                break;
            }
            unsafe {
                gl.clear_color(0.0, 0.0, 0.0, 0.0);
                gl.clear(
                    glow::COLOR_BUFFER_BIT | glow::DEPTH_BUFFER_BIT | glow::STENCIL_BUFFER_BIT,
                );
            }
            // let (width, height) = overlay_window.global_input_state.borrow().window_size;
            //     self.overlay_window.query_input_events(width, height);
           egui_app.update()?;

            overlay_window.window.borrow_mut().swap_buffers();

            overlay_window.glfw.borrow_mut().poll_events();
        }
        Ok(())
    }
}

// pub struct EguiApp {

// }

// impl epi::App for EguiApp {
//     fn setup(
//         &mut self,
//         _ctx: &egui::CtxRef,
//         _frame: &mut epi::Frame<'_>,
//         _storage: Option<&dyn epi::Storage>,
//     ) {
//     }

//     fn warm_up_enabled(&self) -> bool {
//         false
//     }

//     fn save(&mut self, _storage: &mut dyn epi::Storage) {}

//     fn on_exit(&mut self) {}

//     fn auto_save_interval(&self) -> std::time::Duration {
//         std::time::Duration::from_secs(30)
//     }

//     fn max_size_points(&self) -> egui::Vec2 {
//         // Some browsers get slow with huge WebGL canvases, so we limit the size:
//         egui::Vec2::new(1024.0, 2048.0)
//     }

//     fn clear_color(&self) -> egui::Rgba {
//         // NOTE: a bright gray makes the shadows of the windows look weird.
//         // We use a bit of transparency so that if the user switches on the
//         // `transparent()` option they get immediate results.
//         egui::Color32::from_rgba_unmultiplied(12, 12, 12, 180).into()
//     }

//     fn update(&mut self, ctx: &egui::CtxRef, frame: &mut epi::Frame<'_>) {
//         egui::Ui::new(ctx, layer_id, id, max_rect, clip_rect)
//     }

//     fn name(&self) -> &str {
//         "egui app "
//     }
// }

// pub fn process_events(
//     input_state: &mut GlobalInputState,
//     events: &Receiver<(f64, glfw::WindowEvent)>,
//     gl: Rc<glow::Context>,
// ) {
//     for (_, event) in glfw::flush_messages(events) {
//         match event {
//             glfw::WindowEvent::FramebufferSize(width, height) => {
//                 // make sure the viewport matches the new window dimensions; note that width and
//                 // height will be significantly larger than specified on retina displays.
//                 unsafe {
//                     gl.viewport(0, 0, width, height);
//                 }
//                 input_state.dimensions = (width as f32, height as f32);
//                 input_state.egui_input.screen_rect = Some(Rect::from_two_pos(
//                     Pos2::default(),
//                     Pos2::new(width as f32, height as f32),
//                 ));
//             }
//             glfw::WindowEvent::Close => {
//                 std::process::exit(0);
//             }
//             _ => {}
//         }
//     }
// }

// pub fn glfw_window_init() -> (
//     Glfw,
//     Rc<glow::Context>,
//     glfw::Window,
//     std::sync::mpsc::Receiver<(f64, WindowEvent)>,
// ) {
//     let scr_height: u32 = 600;
//     let scr_width: u32 = 800;
//     let mut glfw = glfw::init(glfw::FAIL_ON_ERRORS).unwrap();
//     glfw.window_hint(glfw::WindowHint::ContextVersion(4, 6));
//     glfw.window_hint(glfw::WindowHint::OpenGlProfile(
//         glfw::OpenGlProfileHint::Core,
//     ));
//     glfw.window_hint(glfw::WindowHint::TransparentFramebuffer(true));
//     glfw.window_hint(glfw::WindowHint::Floating(true));
//     // glfw.window_hint(glfw::WindowHint::Decorated(false));
//     //glfw.window_hint(glfw::WindowHint::MousePassthrough(true));
//     // glfw.window_hint(glfw::WindowHint::DoubleBuffer(false));

//     let (mut window, events) = glfw
//         .create_window(
//             scr_width,
//             scr_height,
//             "Egui Experimentation",
//             glfw::WindowMode::Windowed,
//         )
//         .expect("Failed to create GLFW window");

//     window.set_key_polling(true);
//     glfw::Context::make_current(&mut window);
//     window.set_framebuffer_size_polling(true);
//     let gl =
//         unsafe { glow::Context::from_loader_function(|s| window.get_proc_address(s) as *const _) };
//     let gl = Rc::new(gl);

//     (glfw, gl, window, events)
// }

// pub fn create_mlink_cache(key: &str) -> MumbleCache {
//     let retry_times = 50_u32;

//     for _ in 0..retry_times {
//         let socket = UdpSocket::bind("127.0.0.1:0").expect("failed to bind to socket");
//         socket
//             .connect("127.0.0.1:7187")
//             .expect("failed to connect to socket");
//         let mc = MumbleCache::new(key, Duration::from_millis(20), GetMLMode::UdpSync(socket));
//         if mc.is_ok() {
//             return mc.unwrap();
//         }
//         std::thread::sleep(Duration::from_secs(1));
//     }
//     panic!("couldn't get mumblelink after 50 retries");
// }
