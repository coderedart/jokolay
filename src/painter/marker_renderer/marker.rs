use crate::painter::opengl::buffer::{VertexBufferLayout, VertexBufferLayoutTrait};

#[derive(Debug, Clone, Copy, Default)]
pub struct MarkerVertex {
    pub vpos: [f32; 4],
    pub tex_coords: [f32; 3],
    pub alpha: f32,
}

impl VertexBufferLayoutTrait for MarkerVertex {
    fn get_layout() -> crate::painter::opengl::buffer::VertexBufferLayout {
        let mut layout = VertexBufferLayout::default();
        layout.push_f32(4, false);
        layout.push_f32(3, false);
        layout.push_f32(1, false);
        layout
    }
}

unsafe impl bytemuck::Zeroable for MarkerVertex {
    fn zeroed() -> Self {
        unsafe { core::mem::zeroed() }
    }
}
unsafe impl bytemuck::Pod for MarkerVertex {}
