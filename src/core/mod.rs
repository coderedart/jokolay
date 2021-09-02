use egui::{CtxRef, Pos2, RawInput, Rect, Visuals};

use self::{fm::FileManager, input::InputManager, mlink::MumbleManager, painter::Renderer, window::glfw_window::{OverlayWindow, OverlayWindowConfig}};

pub mod fm;
pub mod input;
pub mod painter;
pub mod window;
pub mod mlink;


#[derive(Debug)]
pub struct JokoCore {
    pub im: InputManager,
    pub mbm: MumbleManager,
    pub fm: FileManager,
    pub rr: Renderer,
    pub ow: OverlayWindow,
}



impl JokoCore {
    pub fn new() -> (Self, CtxRef) {
        let (ow, events, glfw, gl) = OverlayWindow::create(OverlayWindowConfig::default())?;

       
  
        let im = InputManager::new(events, glfw);
        let mbm = MumbleManager::new("MumbleLink").unwrap();
        // start setting up egui initial state
        let mut ctx = CtxRef::default();
        let width = ow.config.framebuffer_width;
        let height = ow.config.framebuffer_height;

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

        let t = ctx.texture();
        let rr = Renderer::new(gl.clone(), t);
        let _ = ctx.end_frame();
        let fm = FileManager::new();
        (Self {
            im,
            mbm,
            fm,
            rr,
            ow,
        }, ctx)
    }
}