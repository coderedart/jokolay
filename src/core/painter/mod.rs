use std::rc::Rc;

use egui::{ClippedMesh, CtxRef};
use glm::Vec2;
use glow::{Context, HasContext};

use crate::gl_error;

use self::{egui_renderer::EguiGL, opengl::texture::TextureManager};

use super::fm::FileManager;

pub mod egui_renderer;
// pub mod marker_renderer;
pub mod opengl;
// pub mod trail_renderer;
pub struct Renderer {
    pub egui_gl: EguiGL,
    // pub marker_gl: MarkerGl,
    // pub trail_gl: TrailGl,
    pub tm: TextureManager,
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
        let egui_gl = EguiGL::new(gl.clone());
        unsafe {
            gl_error!(gl);
        }

        let tm = TextureManager::new(gl.clone());
        // let marker_gl = MarkerGl::new(gl.clone());
        // let trail_gl = TrailGl::new(gl);
        Self {
            egui_gl,
            // marker_gl,
            // trail_gl,
            tm,
        }
    }
    pub fn draw_egui(
        &mut self,
        meshes: Vec<ClippedMesh>,
        screen_size: Vec2,
        fm: &FileManager,
        ctx: CtxRef,
    ) {
        unsafe {
            self.egui_gl.gl.disable(glow::SCISSOR_TEST);
            self.egui_gl.gl.clear_color(0.0, 0.0, 0.0, 0.0);
            self.egui_gl
                .gl
                .clear(glow::COLOR_BUFFER_BIT | glow::DEPTH_BUFFER_BIT);
        }
        self.egui_gl
            .draw_meshes(meshes, screen_size, &mut self.tm, fm, ctx)
            .unwrap();
        let gl = self.egui_gl.gl.clone();
        unsafe {
            gl_error!(gl);
        }
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
