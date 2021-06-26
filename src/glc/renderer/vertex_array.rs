use glow::HasContext;

use super::{shader::ShaderProgram, vertex_buffer::{VertexBuffer, VertexBufferLayout}};

pub struct VertexArrayObject<'a> {
    pub id: u32,
    pub vertex_buffer: VertexBuffer<'a>,
    pub buffer_layout: VertexBufferLayout,
    pub index_buffer_id: Option<u32>,
    pub sp: ShaderProgram<'a>,
    gl: &'a glow::Context,
}

impl VertexArrayObject<'_> {
    pub fn new<'a>(
        gl: &'a glow::Context,
        vb: VertexBuffer<'a>,
        vblayout: VertexBufferLayout,
        ib_id_option: Option<u32>,
        sp: ShaderProgram<'a>
    ) -> VertexArrayObject<'a> {
        unsafe {
            let id = gl.create_vertex_array().unwrap();
            gl.bind_vertex_array(Some(id));
            vb.bind();
            vblayout.set_layout(gl);
            if let Some(ib_id) = ib_id_option {
                gl.bind_buffer(glow::ELEMENT_ARRAY_BUFFER, Some(ib_id));
            } else {
                gl.bind_buffer(glow::ELEMENT_ARRAY_BUFFER, None);
            }
            gl.bind_vertex_array(None);
            vb.unbind();
            sp.unbind();
            gl.bind_buffer(glow::ELEMENT_ARRAY_BUFFER, None);

            VertexArrayObject {
                id,
                vertex_buffer: vb,
                buffer_layout: vblayout,
                index_buffer_id: ib_id_option,
                gl,
                sp
            }
        }
    }

    pub fn bind(&self) {
        unsafe {
            self.gl.bind_vertex_array(Some(self.id));
            self.vertex_buffer.bind();
            self.sp.bind();
            self.gl.bind_buffer(glow::ELEMENT_ARRAY_BUFFER, self.index_buffer_id);
        }
    }
    pub fn unbind(&self) {
        unsafe {
            self.gl.bind_vertex_array(None);
            self.vertex_buffer.unbind();
            self.sp.unbind();
            self.gl.bind_buffer(glow::ELEMENT_ARRAY_BUFFER, None);
        }
    }
    pub fn update(&self, vb_slice: &[u8], vb_usage: u32, ib_slice: Option<&[u8]>, ib_usage: u32) {
        self.bind();
        unsafe {
            if ib_slice.is_some() && self.index_buffer_id.is_some() {
                self.gl.buffer_data_u8_slice(
                    glow::ELEMENT_ARRAY_BUFFER,
                    bytemuck::cast_slice(ib_slice.unwrap()),
                    ib_usage,
                );
            }
            self.gl.buffer_data_u8_slice(
                glow::ARRAY_BUFFER,
                bytemuck::cast_slice(&vb_slice),
                vb_usage,
            );
        }
        self.unbind();
    }
}

impl Drop for VertexArrayObject<'_> {
    fn drop(&mut self) {
        unsafe {
            if let Some(ib_id) = self.index_buffer_id {
                self.gl.delete_buffer(ib_id);
            }
            self.gl.delete_vertex_array(self.id);
        }
    }
}
