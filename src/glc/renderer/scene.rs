use std::{convert::TryInto, path::Path};

use super::{
    node::Node,
    quad,
    shader::ShaderProgram,
    vertex_array::VertexArrayObject,
    vertex_buffer::{VertexBuffer, VertexBufferLayout},
};
use glow::{HasContext, UniformLocation};
use nalgebra_glm::{vec3, Mat4, Vec3};

pub struct Scene<'a> {
    pub vao: VertexArrayObject<'a>,
    pub program: ShaderProgram<'a>,
    pub gl: &'a glow::Context,
    pub view_projection: Mat4,
    pub cam_pos: Vec3,
    pub nodes: Vec<Node>,
    pub vp_uniform: UniformLocation,
    pub cam_pos_uniform: UniformLocation,
}

impl<'a> Scene<'_> {
    pub fn new(gl: &'a glow::Context, nodes: Vec<Node>) -> Scene<'a> {
        let program = ShaderProgram::new(
            &gl,
            Path::new("./res/shader.vs"),
            Path::new("./res/shader.gs"),
            Path::new("./res/shader.fs"),
        );
        let vao = setup_buffers(&gl, &quad);
        let view_projection = Mat4::identity();
        let cam_pos = vec3(0.0, 0.0, 0.0);
        program.bind();
        vao.bind();
        let vp_uniform = program.get_uniform_id("VP").unwrap();
        let cam_pos_uniform = program.get_uniform_id("cam_pos").unwrap();
        Scene {
            vao,
            program,
            gl,
            view_projection,
            cam_pos,
            nodes,
            vp_uniform,
            cam_pos_uniform,
        }
    }
    pub fn render(&self) {
        unsafe {
            self.gl.clear_color(0.0, 0.0, 0.0, 0.0);
            self.gl
                .clear(glow::COLOR_BUFFER_BIT | glow::DEPTH_BUFFER_BIT | glow::STENCIL_BUFFER_BIT);
            self.gl.uniform_matrix_4_f32_slice(
                Some(&self.vp_uniform),
                false,
                self.view_projection.as_slice(),
            );
            self.gl
                .uniform_3_f32_slice(Some(&self.cam_pos_uniform), self.cam_pos.as_slice());
                // dbg!(self.nodes.len());
                let num_of_points = self.nodes.len() as i32;
            self.gl
                .draw_arrays(glow::POINTS, 0, 100);
        }
    }
}

fn setup_buffers<'a>(gl: &'a glow::Context, vertices: &[f32]) -> VertexArrayObject<'a> {
    let vb = VertexBuffer::new(gl, bytemuck::cast_slice(&vertices));
    let mut vblayout = VertexBufferLayout::default();
    vblayout.push_float(3, false);
    let vao = VertexArrayObject::new(gl, vb, vblayout);
    vao
}
