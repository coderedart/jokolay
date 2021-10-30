use std::rc::Rc;

use glow::{Context, HasContext};

use crate::{
    core::painter::{egui_renderer::EguiMesh, scene::Scene},
    gl_error,
};

use self::opengl::texture::TextureServer;

pub mod egui_renderer;
// pub mod marker_renderer;
pub mod opengl;
pub mod scene;
// pub mod trail_renderer;
pub struct Renderer {
    pub scene: Scene,
    // pub marker_gl: MarkerGl,
    // pub trail_gl: TrailGl,
    pub ts: TextureServer,
    pub gl: Rc<glow::Context>,
}

impl Renderer {
    pub fn new(gl: Rc<Context>) -> Self {
        unsafe {
            gl.enable(glow::MULTISAMPLE);
            gl.enable(glow::BLEND);
        }
        unsafe {
            gl_error!(gl);
        }
        let scene = Scene::new(gl.clone());
        unsafe {
            gl_error!(gl);
        }

        let ts = TextureServer::new(gl.clone());
        // let marker_gl = MarkerGl::new(gl.clone());
        // let trail_gl = TrailGl::new(gl);
        Self {
            scene,
            // marker_gl,
            // trail_gl,
            ts,
            gl,
        }
    }
    pub fn clear(&self) {
        unsafe {
            self.gl.disable(glow::SCISSOR_TEST);
            self.gl.clear_color(0.0, 0.0, 0.0, 0.0);
            self.gl
                .clear(glow::COLOR_BUFFER_BIT | glow::DEPTH_BUFFER_BIT);
        }
    }
    pub fn update_egui_scene(&mut self, meshes: Vec<EguiMesh>) {
        self.scene.update_egui_scene_state(meshes);
    }

    pub fn draw_egui(&self) {
        self.scene.draw_egui();
    }
    // pub fn draw_markers(&mut self, mm: &mut MarkerManager, link: &MumbleLink, fm: &FileManager, wc: OverlayWindowConfig) {
    //     self.marker_gl.draw_markers(&mut self.tm, mm, link, fm, wc);
    // }
    // pub fn draw_trails(&mut self, mm: &mut MarkerManager, link: &MumbleLink, fm: &FileManager, ctx: CtxRef) {
    //     self.trail_gl.draw_trails(
    //         mm,
    //         link,
    //         fm,
    //         &mut self.tm,
    //         ctx
    //     )
    // }
}

#[derive(Debug, Clone)]
pub enum RenderCommand {
    UpdateEguiScene(Vec<EguiMesh>),
}
