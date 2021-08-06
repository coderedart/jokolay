

use std::rc::Rc;

use crate::painter::opengl::{buffer::Buffer, shader::ShaderProgram, vertex_array::VertexArrayObject};

// use super::xmltypes::xml_marker::Marker;
pub struct MarkerGl {
    pub vao: VertexArrayObject,
    pub vb: Buffer,
    pub sp: ShaderProgram,
    pub gl: Rc<glow::Context>,
}
impl MarkerGl {

    pub fn bind(&self) {
        self.vao.bind();
        self.vb.bind();
        self.sp.bind();
    }

    pub fn unbind(&self) {
        self.vao.unbind();
        self.vb.unbind();
        self.sp.unbind();
    }
}

