use glow::HasContext;

use super::vertex_buffer::{VertexBuffer, VertexBufferLayout, VertexBufferLayoutElement};

pub struct VertexArrayObject<'a> {
    pub id: u32,
    active_vertex_buffer: VertexBuffer<'a>,
    active_buffer_layout: VertexBufferLayout,
    active_index_buffer: u32,
    gl: &'a glow::Context,
}

impl VertexArrayObject<'_> {
    pub fn new<'a>(
        gl: &'a glow::Context,
        vb: VertexBuffer<'a>,
        vblayout: VertexBufferLayout,
    ) -> VertexArrayObject<'a> {
        unsafe {
            let id = gl.create_vertex_array().unwrap();
            gl.bind_vertex_array(Some(id));
            vb.bind();
            vblayout.set_layout(gl);
            VertexArrayObject {
                id,
                active_vertex_buffer: vb,
                active_buffer_layout: vblayout,
                active_index_buffer: 0,
                gl,
            }
        }
    }

    pub fn bind(&self) {
        unsafe {
            self.gl.bind_vertex_array(Some(self.id));
        }
    }
}
