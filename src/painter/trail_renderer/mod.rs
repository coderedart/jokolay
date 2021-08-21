use std::rc::Rc;

use glow::NativeUniformLocation;

use crate::painter::{marker_renderer::marker::MarkerVertex, opengl::{buffer::{Buffer, VertexBufferLayout, VertexBufferLayoutTrait}, shader::ShaderProgram, vertex_array::VertexArrayObject}};
// use super::xmltypes::xml_marker::Marker;
pub mod trail;
pub struct TrailGl {
    pub vao: VertexArrayObject,
    pub vb: Buffer,
    pub sp: ShaderProgram,
    pub u_sampler: NativeUniformLocation,
    pub gl: Rc<glow::Context>,
}
impl TrailGl {
    pub fn new(gl: Rc<glow::Context>) -> Self {
        let vao = VertexArrayObject::new(gl.clone());
        let vb = Buffer::new(gl.clone(), glow::ARRAY_BUFFER);
        let sp = ShaderProgram::new(gl.clone(), VERTEX_SHADER_SRC, FRAG_SHADER_SRC, None);
        let u_sampler = sp.get_uniform_id("sampler").unwrap();
        let trail_gl = Self {
            vao,
            vb,
            sp,
            u_sampler,
            gl: gl.clone(),
        };
        trail_gl.bind();
        let layout = MarkerVertex::get_layout();
        layout.set_layout(gl);
        trail_gl

    }
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
pub const VERTEX_SHADER_SRC: &str = include_str!("shader.vs");
pub const FRAG_SHADER_SRC: &str = include_str!("shader.fs");
