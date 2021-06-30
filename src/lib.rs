use std::{ cell::RefCell, collections::{BTreeMap, BTreeSet}, net::UdpSocket, rc::Rc, sync::mpsc::Receiver, time::{Duration, Instant}};

use copypasta::ClipboardContext;
use device_query::DeviceState;
use egui::{Color32, Event, Pos2, RawInput, Rect, epaint::Shadow};
use glc::{
    eglfw::{egui_node::EguiSceneNode, input::GlobalInputState},
    renderer::texture::Texture,
};
use glfw::{Action, Glfw, Key, WindowEvent};
use glow::HasContext;
use gw::{category::MarkerCategory, marker::Marker, trail::Trail};
use mlink::MumbleCache;
use nalgebra_glm::make_vec2;

use crate::mlink::GetMLMode;

pub mod glc;
pub mod gw;
pub mod mlink;

pub struct JokolayApp {
    pub glfw: Glfw,
    pub gl: Rc<glow::Context>,
    pub window: glfw::Window,
    pub glfw_events: Receiver<(f64, WindowEvent)>,
    pub marker_categories: BTreeMap<String, MarkerCategory>,
    pub markers: BTreeMap<u32, Vec<Marker>>,
    pub trails: BTreeMap<u32, Vec<Trail>>,
    pub ctx: Rc<RefCell<egui::CtxRef>>,
    pub e_renderer: Rc<RefCell<glc::eglfw::egui_node::EguiSceneNode>>,
    pub input_state: Rc<RefCell<GlobalInputState>>,
}

impl JokolayApp {
    pub fn new() -> Self {
        let (glfw, gl, window, glfw_events) = glfw_window_init();
        // let (marker_categories, markers, trails) = load_markers();
        let ctx = Rc::new(RefCell::new(egui::CtxRef::default()));
        let e_renderer = Rc::new(RefCell::new(EguiSceneNode::new(gl.clone())));

        ctx.borrow_mut().begin_frame(RawInput::default());
        let t = ctx.borrow_mut().texture();
        let new_texture = Texture::new(gl.clone(), glow::TEXTURE_2D);
        e_renderer.borrow_mut().material.textures.push(new_texture);
        let index = e_renderer.borrow().material.textures.len() - 1;
        e_renderer
            .borrow_mut()
            .texture_versions
            .insert(egui::TextureId::Egui, index);
        {
            let tex_id = &e_renderer.borrow().material.textures[0];
            tex_id.bind();
            let mut pixels = Vec::new();
            for &alpha in &t.pixels {
                let srgba = Color32::from_white_alpha(alpha);
                pixels.push(srgba.r());
                pixels.push(srgba.g());
                pixels.push(srgba.b());
                pixels.push(srgba.a());
            }
            tex_id.update_pixels(&[&pixels], t.width as u32, t.height as u32);
        }
        let _ = ctx.borrow_mut().end_frame();

        unsafe {
            let e = gl.get_error();
            if e != glow::NO_ERROR {
                println!("glerror {} at {} {} {}", e, file!(), line!(), column!());
            }
        }

        let input_state = Rc::new(RefCell::new(GlobalInputState::new()));
        JokolayApp {
            glfw,
            gl,
            window,
            glfw_events,
            marker_categories: BTreeMap::new(),
            markers: BTreeMap::new(),
            trails: BTreeMap::new(),
            ctx,
            e_renderer,
            input_state,
        }
    }
    pub fn run(&mut self) {
        let gl = self.gl.clone();
        let ctx = self.ctx.clone();
        let renderer = self.e_renderer.clone();
        let mut input = RawInput::default();
        input.pixels_per_point = Some(1_f32);
        input.predicted_dt = 1.0 / 75.0;
        input.screen_rect = Some(Rect::from_two_pos(
            Pos2::new(0.0, 0.0),
            Pos2::new(800.0, 600.0),
        ));
        unsafe {
            gl.active_texture(glow::TEXTURE0);
        }

        let mut previous = std::time::Instant::now();
        let mut fps = 0;
        let mut rendering = std::time::Duration::from_secs(0);
        let mut input_gather = std::time::Duration::from_secs(0);
        loop {
            unsafe {
                let e = self.gl.get_error();
                if e != glow::NO_ERROR {
                    println!("glerror {} at {} {} {}", e, file!(), line!(), column!());
                }
            }
            fps += 1;
            if previous.elapsed() > std::time::Duration::from_secs(1) {
                previous = std::time::Instant::now();
                dbg!(fps, rendering, input_gather);
                fps = 0;
            }

            process_events(
                &mut self.input_state.borrow_mut(),
                &self.glfw_events,
                gl.clone(),
            );
            unsafe {
                gl.clear_color(0.0, 0.0, 0.0, 0.0);
                self.gl.clear(
                    glow::COLOR_BUFFER_BIT | glow::DEPTH_BUFFER_BIT | glow::STENCIL_BUFFER_BIT,
                );
            }
            let new_input = Instant::now();
            self.query_input_events();
            if self.input_state.borrow().egui_input.events.iter().any(|k| {
                matches!(
                    k,
                    Event::Key {
                        key: egui::Key::ArrowRight,
                        ..
                    }
                )
            }) {
                self.window.set_size(self.input_state.borrow().dimensions.0 as i32 + 20, self.input_state.borrow().dimensions.1 as i32)
            }
            // if !input.events.is_empty() {
            // dbg!(&input.events);}
            input_gather = (input_gather + new_input.elapsed()) / 2;
            ctx.borrow_mut()
                .begin_frame(self.input_state.borrow_mut().egui_input.take());
            // egui::CentralPanel::default().show(ctx, |ui| {
            //     ui.add(egui::Label::new("whatever, big text. look at me sempai"));

            // });
            let mut frame = egui::Frame::default().fill(Color32::BLACK).multiply_with_opacity(0.5);
            frame.shadow = Shadow::small_dark();
            
            egui::Window::new("egui window").frame(frame).show(&ctx.borrow(), |ui| {
                ui.add(egui::Label::new(
                    "label inside window. please look at me sempai",
                ));
                if ui.button("click me").clicked() {
                    println!("clicked");
                }
            });
            // egui::SidePanel::left("best panel ever").show(&ctx.borrow(), |ui| {
            //     ui.add(egui::Label::new("ffs. what's with the blur"));
            // });
            let end_frame = Instant::now();
            let (_, shapes) = ctx.borrow_mut().end_frame();

            let meshes = egui::Context::tessellate(&ctx.borrow(), shapes);

            renderer.borrow_mut().draw_meshes(
                &meshes,
                make_vec2(&[
                    self.input_state.borrow().dimensions.0,
                    self.input_state.borrow().dimensions.1,
                ]),
                0,
            );
            rendering = (rendering + end_frame.elapsed()) / 2;

            glfw::Context::swap_buffers(&mut self.window);

            self.glfw.poll_events();
        }
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

pub fn process_events(
    input_state: &mut GlobalInputState,
    events: &Receiver<(f64, glfw::WindowEvent)>,
    gl: Rc<glow::Context>,
) {
    for (_, event) in glfw::flush_messages(events) {
        match event {
            glfw::WindowEvent::FramebufferSize(width, height) => {
                // make sure the viewport matches the new window dimensions; note that width and
                // height will be significantly larger than specified on retina displays.
                unsafe {
                    gl.viewport(0, 0, width, height);
                }
                input_state.dimensions = (width as f32, height as f32);
                input_state.egui_input.screen_rect = Some(Rect::from_two_pos(
                    Pos2::default(),
                    Pos2::new(width as f32, height as f32),
                ));
            }
            glfw::WindowEvent::Close => {
                std::process::exit(0);
            }
            _ => {}
        }
    }
}

pub fn glfw_window_init() -> (
    Glfw,
    Rc<glow::Context>,
    glfw::Window,
    std::sync::mpsc::Receiver<(f64, WindowEvent)>,
) {
    let scr_height: u32 = 600;
    let scr_width: u32 = 800;
    let mut glfw = glfw::init(glfw::FAIL_ON_ERRORS).unwrap();
    glfw.window_hint(glfw::WindowHint::ContextVersion(4, 6));
    glfw.window_hint(glfw::WindowHint::OpenGlProfile(
        glfw::OpenGlProfileHint::Core,
    ));
    glfw.window_hint(glfw::WindowHint::TransparentFramebuffer(true));
    glfw.window_hint(glfw::WindowHint::Floating(true));
    glfw.window_hint(glfw::WindowHint::Decorated(false));
    //glfw.window_hint(glfw::WindowHint::MousePassthrough(true));
    // glfw.window_hint(glfw::WindowHint::DoubleBuffer(false));

    let (mut window, events) = glfw
        .create_window(
            scr_width,
            scr_height,
            "Egui Experimentation",
            glfw::WindowMode::Windowed,
        )
        .expect("Failed to create GLFW window");

    window.set_key_polling(true);
    glfw::Context::make_current(&mut window);
    window.set_framebuffer_size_polling(true);
    let gl =
        unsafe { glow::Context::from_loader_function(|s| window.get_proc_address(s) as *const _) };
    let gl = Rc::new(gl);

    (glfw, gl, window, events)
}

pub fn create_mlink_cache(key: &str) -> MumbleCache {
    let retry_times = 50_u32;

    for _ in 0..retry_times {
        let socket = UdpSocket::bind("127.0.0.1:0").expect("failed to bind to socket");
        socket
            .connect("127.0.0.1:7187")
            .expect("failed to connect to socket");
        let mc = MumbleCache::new(key, Duration::from_millis(20), GetMLMode::UdpSync(socket));
        if mc.is_ok() {
            return mc.unwrap();
        }
        std::thread::sleep(Duration::from_secs(1));
    }
    panic!("couldn't get mumblelink after 50 retries");
}
