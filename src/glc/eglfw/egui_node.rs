use egui::ClippedMesh;
use glow::{HasContext, UNSIGNED_INT};
use nalgebra_glm::Vec2;

use crate::glc::renderer::{
    buffer::Buffer,
    material::{Material, MaterialUniforms},
    scene::{Renderable, SceneNodeUniform},
    vertex_array::VertexArrayObject,
};

use super::VertexRgba;

pub struct EguiSceneNode<'a> {
    pub vao: VertexArrayObject<'a>,
    pub vb: Buffer<'a>,
    pub ib: Buffer<'a>,
    pub material: Material<'a>,
    pub gl: &'a glow::Context,
}

impl EguiSceneNode<'_> {
    pub fn draw_meshes(&self, meshes: &Vec<ClippedMesh>, screen_size: Vec2, etex_sampler: u32) {
        self.bind();
        for clipped_mesh in meshes {
            self.update_uniforms(SceneNodeUniform::EguiSceneNodeUniform{screen_size, etex_sampler});
            self.draw_mesh(clipped_mesh);
        }
    }
    pub fn draw_mesh(&self, clipped_mesh: &ClippedMesh) {
        let _clip_rect = clipped_mesh.0;
        let mesh = &clipped_mesh.1;
        let vertices: Vec<VertexRgba> = mesh.vertices.iter().map(|v| VertexRgba::from(v)).collect();
        
        let indices = &mesh.indices;
        let _texture_id = mesh.texture_id;

        self.update_buffers(Some((bytemuck::cast_slice(&vertices), glow::STREAM_DRAW)), Some((bytemuck::cast_slice(indices), glow::STREAM_DRAW)));
        self.render(indices.len() as u32, 0);

    }
}
impl Renderable<'_> for EguiSceneNode<'_> {
    fn bind(&self) {
        self.vao.bind();
        self.vb.bind();
        self.ib.bind();
        self.material.bind();
    }
    fn update_buffers(&self, vb: Option<(&[u8], u32)>, ib: Option<(&[u8], u32)>) {
        if let Some((data, usage)) = vb {
            self.vb.update(data, usage);
        }
        if let Some((data, usage)) = ib {
            self.ib.update(data, usage);
        }
    }

    fn update_uniforms(&self, uniform_data: SceneNodeUniform) {
        match uniform_data {
            SceneNodeUniform::MarkerSceneNodeUniform {
                vp: _,
                cam_pos: _,
                player_pos: _,
                samplers: _,
            } => unimplemented!(),
            SceneNodeUniform::EguiSceneNodeUniform {
                screen_size,
                etex_sampler,
            } => unsafe {
                self.gl.uniform_2_f32_slice(
                    self.material
                        .uniforms
                        .get(&MaterialUniforms::EguiScreenSize),
                    screen_size.as_slice(),
                );
                self.gl.uniform_1_u32(
                    self.material
                        .uniforms
                        .get(&MaterialUniforms::EguiEtexSampler),
                    etex_sampler,
                )
            },
        }
    }

    fn render(&self, count: u32, offset: u32) {
        unsafe {
            self.gl
                .draw_elements(glow::TRIANGLES, count as i32, UNSIGNED_INT, offset as i32);
        }
    }
    fn unbind(&self) {
        self.vao.unbind();
        self.vb.unbind();
        self.ib.unbind();
        self.material.unbind();
    }
}
