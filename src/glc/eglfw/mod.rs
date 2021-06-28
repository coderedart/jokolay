pub mod egui_node;

use egui::{epaint::Vertex, Pos2};

use super::renderer::buffer::{VertexBufferLayout, VertexBufferLayoutTrait};


#[derive(Debug, Clone, Copy)]
pub struct VertexRgba {
    pub position: Pos2,
    pub uv: Pos2,
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl VertexBufferLayoutTrait for VertexRgba {
    fn get_layout() -> VertexBufferLayout {
        let mut vbl = VertexBufferLayout::default();
        vbl.push_f32(2, false);
        vbl.push_f32(2, false);
        vbl.push_f32(4, false);
        vbl
    }
}
impl From<&Vertex> for VertexRgba {
    fn from(vert: &Vertex) -> Self {
        VertexRgba {
            position: vert.pos,
            uv: vert.uv,
            r: vert.color.r() as f32 ,
            g: vert.color.g() as f32 ,
            b: vert.color.b() as f32 ,
            a: vert.color.a() as f32 ,
        }
    }
}
impl From<Vertex> for VertexRgba {
    fn from(vert: Vertex) -> Self {
        VertexRgba {
            position: vert.pos,
            uv: vert.uv,
            r: vert.color.r() as f32 ,
            g: vert.color.g() as f32 ,
            b: vert.color.b() as f32 ,
            a: vert.color.a() as f32 ,
        }
    }
}

unsafe impl bytemuck::Zeroable for VertexRgba {
    fn zeroed() -> Self {
    unsafe { core::mem::zeroed() }
  }
}
unsafe impl bytemuck::Pod for VertexRgba {
    
}