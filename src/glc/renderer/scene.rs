use std::path::Path;

use crate::glc::eglfw;

use super::{
    node::Node,
    shader::ShaderProgram,
    vertex_array::VertexArrayObject,
    vertex_buffer::{VertexBuffer, VertexBufferLayout},
};
use egui::epaint::Vertex;
use glow::{DYNAMIC_DRAW, HasContext, STREAM_DRAW, UNSIGNED_INT, UniformLocation};
use nalgebra_glm::{vec3, Mat4, Vec3};

pub struct Scene<'a> {
    pub vao_node: VertexArrayObject<'a>,
    pub vao_egui: VertexArrayObject<'a>,
    pub gl: &'a glow::Context,
    pub vp_uniform_node: UniformLocation,
    pub cam_pos_uniform_node: UniformLocation,
}

impl<'a> Scene<'_> {
    pub fn new(gl: &'a glow::Context) -> Scene<'a> {
        
        let vao_node = setup_vao_node(&gl);
        vao_node.bind();
        let vp_uniform_node = vao_node.sp.get_uniform_id("VP").unwrap();
        let cam_pos_uniform_node = vao_node.sp.get_uniform_id("cam_pos").unwrap();
        vao_node.unbind();
        let vao_egui = setup_vao_egui(&gl);
        vao_egui.unbind();
        Scene {
            vao_node,
            vao_egui,
            gl,
            vp_uniform_node,
            cam_pos_uniform_node,
        }
    }

    pub fn clear_screen(&self) {
        unsafe {
            self.gl.clear_color(0.0, 0.0, 0.0, 0.0);
            self.gl
                .clear(glow::COLOR_BUFFER_BIT | glow::DEPTH_BUFFER_BIT );
        }
    }
    pub fn render_nodes(&self, view_projection: &Mat4, cam_pos: &Vec3, num: i32) {
        unsafe {
            self.vao_node.bind();
            self.gl.uniform_matrix_4_f32_slice(
                Some(&self.vp_uniform_node),
                false,
                view_projection.as_slice(),
            );
            self.gl
                .uniform_3_f32_slice(Some(&self.cam_pos_uniform_node), cam_pos.as_slice());

            self.gl.draw_arrays(glow::POINTS, 0, num);
            // self.gl.draw_arrays(glow::POINTS, 0, 200);
            
            self.vao_node.unbind();
        }
    }
    pub fn render_egui(
        &self,
        count: i32
        // vertices: &Vec<egui::paint::Vertex>,
        // indices: &Vec<u32>,
        // tex_slot: u32,
    ) {
        unsafe {
            
            self.vao_egui.bind();
            
            self.gl
                .draw_elements(glow::TRIANGLES, count , UNSIGNED_INT, 0);

            // self.gl.draw_arrays(glow::TRIANGLES, 0, count);
            self.vao_egui.unbind();
        }
    }
    pub fn update_egui_buffers(&self, vertices: &Vec<Vertex>, indices: &Vec<u32>) {
        self.vao_egui.update(
            unsafe {
            std::slice::from_raw_parts(
                vertices.as_ptr() as *const u8,
                vertices.len() * std::mem::size_of::<Vertex>(),
            )},
            DYNAMIC_DRAW,
            Some(bytemuck::cast_slice(indices)),
            DYNAMIC_DRAW
        );
    }
    pub fn update_nodes(&mut self, nodes: &Vec<Node>) {
        // let ib: Vec<u32> = (0..nodes.len() as u32).collect();
        self.vao_node.update(bytemuck::cast_slice(&nodes),DYNAMIC_DRAW, None, DYNAMIC_DRAW);
    }
}

fn setup_vao_node<'a>(gl: &'a glow::Context) -> VertexArrayObject<'a> {
    let vb = VertexBuffer::new(gl);
    let vblayout = Node::get_buffer_layout();
    let mut ib_id = None;
    unsafe {
        ib_id = Some(gl.create_buffer().unwrap());
    }
    let program_node = ShaderProgram::new(
        &gl,
        Path::new("./res/node.vs"),
        Some(Path::new("./res/node.gs")),
        Path::new("./res/node.fs"),
    );
    let vao = VertexArrayObject::new(gl, vb, vblayout, ib_id, program_node);
    vao
}




fn setup_vao_egui<'a>(gl: &'a glow::Context) -> VertexArrayObject<'a> {
    let vb = VertexBuffer::new(gl);
    let vblayout = eglfw::get_egui_vertex_buffer_layout();
    let mut ib_id = None;
    unsafe {
        ib_id = Some(gl.create_buffer().unwrap());
    }
    let program_egui = ShaderProgram::new(
        &gl,
        Path::new("./res/egui.vs"),
        None,
        Path::new("./res/egui.fs"),
    );
    let vao = VertexArrayObject::new(gl, vb, vblayout, ib_id, program_egui);
    vao
}
