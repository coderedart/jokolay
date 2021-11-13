use std::rc::Rc;

use egui::{Rect, TextureId};
use glm::Vec2;
use glow::{Context, HasContext, NativeUniformLocation, UNSIGNED_INT};

use crate::gl_error;

use super::opengl::{buffer::Buffer, shader::ShaderProgram, vertex_array::VertexArrayObject};

#[derive(Debug)]
pub struct EguiGL {
    pub vao: VertexArrayObject,
    pub sp: ShaderProgram,
    pub u_sampler: NativeUniformLocation,
    pub u_tcx_offset: NativeUniformLocation,
    pub u_tcy_offset: NativeUniformLocation,
    pub u_tcx_scale: NativeUniformLocation,
    pub u_tcy_scale: NativeUniformLocation,
    pub u_sampler_layer: NativeUniformLocation,
    pub u_screen_size: NativeUniformLocation,
    pub gl: Rc<glow::Context>,
}

impl EguiGL {
    pub fn new(gl: Rc<Context>) -> EguiGL {
        let layout = VertexRgba::get_layout();
        let vao = VertexArrayObject::new(gl.clone(), layout);

        let program = ShaderProgram::new(
            gl.clone(),
            EGUI_VERTEX_SHADER_SRC,
            EGUI_FRAGMENT_SHADER_SRC,
            None,
        );

        let u_sampler;
        let u_sampler_layer;
        let u_screen_size;
        let u_tcx_offset;
        let u_tcy_offset;
        let u_tcx_scale;
        let u_tcy_scale;
        unsafe {
            u_sampler = gl.get_uniform_location(program.id, "sampler").unwrap();
            u_sampler_layer = gl
                .get_uniform_location(program.id, "sampler_layer")
                .unwrap();
            u_screen_size = gl.get_uniform_location(program.id, "screen_size").unwrap();
            u_tcx_offset = gl.get_uniform_location(program.id, "tc_x_offset").unwrap();
            u_tcy_offset = gl.get_uniform_location(program.id, "tc_y_offset").unwrap();
            u_tcx_scale = gl.get_uniform_location(program.id, "tc_x_scale").unwrap();
            u_tcy_scale = gl.get_uniform_location(program.id, "tc_y_scale").unwrap();
            gl_error!(gl);
        }

        let egui_gl = EguiGL {
            vao,
            sp: program,
            u_sampler,
            u_tcx_offset,
            u_tcy_offset,
            u_tcx_scale,
            u_tcy_scale,
            u_sampler_layer,
            u_screen_size,
            gl: gl.clone(),
        };
        egui_gl.bind();
        unsafe {
            gl_error!(gl);
        }

        return egui_gl;
    }

    pub fn enable_egui_state(&self) {
        self.bind();

        unsafe {
            self.gl.enable(glow::FRAMEBUFFER_SRGB);
            self.gl.disable(glow::DEPTH_TEST);
            self.gl.blend_func(glow::ONE, glow::ONE_MINUS_SRC_ALPHA);
            self.gl.enable(glow::SCISSOR_TEST);
        }
    }
    pub fn disable_egui_state(&self) {
        unsafe {
            self.gl.disable(glow::SCISSOR_TEST);
            self.gl.disable(glow::FRAMEBUFFER_SRGB);
        }
    }
    pub fn update_uniforms(
        &self,
        sampler: i32,
        sampler_layer: i32,
        tcx_offset: f32,
        tcy_offset: f32,
        tcx_scale: f32,
        tcy_scale: f32,
        screen_size: Vec2,
    ) {
        // update the scaling/offsets of texture in the texture array of atlasses
        unsafe {
            //sampler uniforms are i32
            self.gl.uniform_1_i32(Some(&self.u_sampler), sampler);
            self.gl
                .uniform_2_f32_slice(Some(&self.u_screen_size), screen_size.as_slice());
            self.gl
                .uniform_1_i32(Some(&self.u_sampler_layer), sampler_layer as i32);
            self.gl.uniform_1_f32(Some(&self.u_tcx_offset), tcx_offset);
            self.gl.uniform_1_f32(Some(&self.u_tcy_offset), tcy_offset);
            self.gl.uniform_1_f32(Some(&self.u_tcx_scale), tcx_scale);
            self.gl.uniform_1_f32(Some(&self.u_tcy_scale), tcy_scale);
        }
    }

    pub fn draw_mesh(&self, count: u32, vb: &Buffer, ib: &Buffer) -> anyhow::Result<()> {
        unsafe {
            // set the buffers as the vertex array binding points
            self.gl.vertex_array_vertex_buffer(
                self.vao.id,
                0,
                Some(vb.id),
                0,
                std::mem::size_of::<egui::epaint::Vertex>() as i32,
            );
            self.gl
                .vertex_array_element_buffer(self.vao.id, Some(ib.id));
        }

        self.render(count, 0);
        Ok(())
    }
    pub fn set_scissor(clip_rect: Rect, gl: Rc<glow::Context>, screen_size: Vec2) {
        //clip rectangle copy pasted from glium
        let clip_min_x = clip_rect.min.x;
        let clip_min_y = clip_rect.min.y;
        let clip_max_x = clip_rect.max.x;
        let clip_max_y = clip_rect.max.y;

        // Make sure clip rect can fit within a `u32`:
        let clip_min_x = clip_min_x.clamp(0.0, screen_size.x);
        let clip_min_y = clip_min_y.clamp(0.0, screen_size.y);
        let clip_max_x = clip_max_x.clamp(clip_min_x, screen_size.x);
        let clip_max_y = clip_max_y.clamp(clip_min_y, screen_size.y);

        let clip_min_x = clip_min_x.round() as u32;
        let clip_min_y = clip_min_y.round() as u32;
        let clip_max_x = clip_max_x.round() as u32;
        let clip_max_y = clip_max_y.round() as u32;

        unsafe {
            gl.scissor(
                clip_min_x as i32,
                (screen_size.y - clip_max_y as f32) as i32,
                (clip_max_x - clip_min_x) as i32,
                (clip_max_y - clip_min_y) as i32,
            );
        }
    }
}
impl EguiGL {
    pub fn bind(&self) {
        self.vao.bind();
        self.sp.bind();
    }

    fn render(&self, count: u32, offset: u32) {
        unsafe {
            self.gl
                .draw_elements(glow::TRIANGLES, count as i32, UNSIGNED_INT, offset as i32);
        }
    }
    fn _unbind(&self) {
        self.vao.unbind();
        self.sp.unbind();
    }
}

#[derive(Debug, Clone)]
pub struct EguiMesh {
    pub sampler: i32,
    pub sampler_layer: i32,
    pub tcx_offset: f32,
    pub tcy_offset: f32,
    pub tcx_scale: f32,
    pub tcy_scale: f32,
    pub screen_size: Vec2,
    pub clip_rect: egui::Rect,
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>,
    pub tid: TextureId,
}
#[derive(Debug)]
pub struct EguiSceneState {
    pub egui_gl: EguiGL,
    pub meshes: Vec<(EguiMesh, Buffer, Buffer)>,
}
impl EguiSceneState {
    pub fn new(gl: Rc<glow::Context>) -> Self {
        Self {
            egui_gl: EguiGL::new(gl.clone()),
            meshes: vec![],
        }
    }
    pub fn draw(&self) {
        self.egui_gl.enable_egui_state();
        for (m, vb, ib) in self.meshes.iter() {
            self.egui_gl.update_uniforms(
                0,
                m.sampler_layer,
                m.tcx_offset,
                m.tcy_offset,
                m.tcx_scale,
                m.tcy_scale,
                m.screen_size,
            );
            EguiGL::set_scissor(m.clip_rect, self.egui_gl.gl.clone(), m.screen_size);
            self.egui_gl
                .draw_mesh(m.indices.len() as u32, vb, ib)
                .unwrap();
        }
        self.egui_gl.disable_egui_state();
    }
}

use egui::{epaint::Vertex, Pos2};

use crate::core::painter::opengl::buffer::{VertexBufferLayout, VertexBufferLayoutTrait};

#[derive(Debug, Clone, Copy)]
pub struct VertexRgba {
    pub pos: Pos2,
    pub uv: Pos2,
    pub color: [u8; 4],
}

impl VertexBufferLayoutTrait for VertexRgba {
    fn get_layout() -> VertexBufferLayout {
        let mut vbl = VertexBufferLayout::default();
        vbl.push_f32(2, false);
        vbl.push_f32(2, false);
        vbl.push_u8(4, false);
        vbl
    }
}
impl From<&Vertex> for VertexRgba {
    fn from(vert: &Vertex) -> Self {
        VertexRgba {
            pos: vert.pos,
            uv: vert.uv,
            color: vert.color.to_array(),
        }
    }
}
impl From<Vertex> for VertexRgba {
    fn from(vert: Vertex) -> Self {
        VertexRgba {
            pos: vert.pos,
            uv: vert.uv,
            color: vert.color.to_array(),
        }
    }
}

unsafe impl bytemuck::Zeroable for VertexRgba {
    fn zeroed() -> Self {
        unsafe { core::mem::zeroed() }
    }
}
unsafe impl bytemuck::Pod for VertexRgba {}

const EGUI_FRAGMENT_SHADER_SRC: &str = include_str!("shader.fs");

const EGUI_VERTEX_SHADER_SRC: &str = include_str!("shader.vs");
