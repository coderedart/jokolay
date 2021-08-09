use crate::painter::opengl::buffer::{VertexBufferLayout, VertexBufferLayoutTrait};


#[derive(Debug, Clone, Copy, Default)]
pub struct Vertex {
    pub vpos: [f32; 3],
    pub tex_coords: [f32; 3]
}

impl VertexBufferLayoutTrait for Vertex {
    fn get_layout() -> crate::painter::opengl::buffer::VertexBufferLayout {
        let mut layout = VertexBufferLayout::default();
        layout.push_f32(3, false);
        layout.push_f32(3, false);
        layout
    }
}

unsafe impl bytemuck::Zeroable for Vertex {
    fn zeroed() -> Self {
    unsafe { core::mem::zeroed() }
  }
}
unsafe impl bytemuck::Pod for Vertex {}