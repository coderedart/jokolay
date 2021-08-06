use std::{rc::Rc, sync::Arc};

use egui::ClippedMesh;
use glm::Vec2;
use glow::Context;

use crate::painter::{egui_renderer::EguiGL, marker_renderer::MarkerGl, opengl::texture::TextureManager};

pub mod opengl;
pub mod egui_renderer;
pub mod marker_renderer;
pub struct Painter {
    pub egui_gl: EguiGL,
    // pub marker_gl: MarkerGl,
    pub tm: TextureManager,
}

impl Painter {
    pub fn new(gl: Rc<Context>, t: Arc<egui::Texture>) -> Self {
        let egui_gl = EguiGL::new(gl.clone());
        
        let tm = TextureManager::new(gl, t);

        Self {
            egui_gl,
            tm
        }



    }
    pub fn draw_egui(&mut self, meshes: Vec<ClippedMesh>, screen_size: Vec2 ) {
        self.egui_gl.draw_meshes(meshes, screen_size, &mut self.tm).unwrap();
    }
}