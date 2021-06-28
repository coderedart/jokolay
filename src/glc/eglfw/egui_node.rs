use std::{collections::BTreeMap, path::Path, rc::Rc};

use egui::ClippedMesh;
use glow::{Context, HasContext, UNSIGNED_INT};
use nalgebra_glm::Vec2;

use crate::glc::renderer::{
    buffer::Buffer,
    material::{Material, MaterialUniforms},
    scene::{Renderable, SceneNodeUniform},
    shader::ShaderProgram,
    texture::Texture,
    vertex_array::VertexArrayObject,
};


pub struct EguiSceneNode {
    pub vao: VertexArrayObject,
    pub vb: Buffer,
    pub ib: Buffer,
    pub material: Material,
    pub gl: Rc<glow::Context>,
}

impl EguiSceneNode {
    pub fn new(gl: Rc<Context>) -> EguiSceneNode {
        let vao = VertexArrayObject::new(gl.clone());
        let vb = Buffer::new(gl.clone(), glow::ARRAY_BUFFER);
        let ib = Buffer::new(gl.clone(), glow::ELEMENT_ARRAY_BUFFER);
        let program = ShaderProgram::new(
            gl.clone(),
            &Path::new("./res/egui.vs"),
            None,
            &Path::new("./res/egui.fs"),
        );
        let texture = Texture::new(gl.clone(), glow::TEXTURE_2D);
        let texture = vec![texture];
        let mut uniforms: BTreeMap<MaterialUniforms, u32> = BTreeMap::new();
        unsafe {
            let ss = gl.get_uniform_location(program.id, "screen_size").unwrap();
            let sampler = gl.get_uniform_location(program.id, "m_sampler").unwrap();
            uniforms.insert(MaterialUniforms::EguiScreenSize, ss);
            uniforms.insert(MaterialUniforms::EguiEtexSampler, sampler);
        }
        
        let material = Material {
            program,
            texture,
            uniforms,
            gl: gl.clone(),
        };
        let egui_scene_node = 
        EguiSceneNode {
            vao,
            vb,
            ib,
            material,
            gl: gl.clone(),
        };
        egui_scene_node.bind();
        let layout = VertexRgba::get_layout();
        layout.set_layout(gl.clone());

        return egui_scene_node;

    }

    pub fn draw_meshes(&self, meshes: &Vec<ClippedMesh>, screen_size: Vec2, etex_sampler: u32) {
        self.bind();
        unsafe {
            let e = self.gl.get_error();
            if e != glow::NO_ERROR {
                println!("glerror {} at {} {} {}",e, file!(), line!(), column!());
            }
        }
        unsafe {
            self.gl.enable(glow::FRAMEBUFFER_SRGB);
            // IF we use clip rectangle
            // self.gl.enable(glow::SCISSOR_TEST);
        }
        unsafe {
            let e = self.gl.get_error();
            if e != glow::NO_ERROR {
                println!("glerror {} at {} {} {}",e, file!(), line!(), column!());
            }
        }
        for clipped_mesh in meshes {
            self.update_uniforms(SceneNodeUniform::EguiSceneNodeUniform {
                screen_size,
                etex_sampler,
            });
            unsafe {
                let e = self.gl.get_error();
                if e != glow::NO_ERROR {
                    println!("glerror {} at {} {} {}",e, file!(), line!(), column!());
                }
            }
            self.draw_mesh(clipped_mesh);
            unsafe {
                let e = self.gl.get_error();
                if e != glow::NO_ERROR {
                    println!("glerror {} at {} {} {}",e, file!(), line!(), column!());
                }
            }
        }
        unsafe {
            self.gl.disable(glow::FRAMEBUFFER_SRGB);
        }
    }
    pub fn draw_mesh(&self, clipped_mesh: &ClippedMesh) {
        let _clip_rect = clipped_mesh.0;

        let mesh = &clipped_mesh.1;
        let vertices: Vec<VertexRgba> = mesh.vertices.iter().map(|v| VertexRgba::from(v)).collect();

        let indices = &mesh.indices;
        let _texture_id = mesh.texture_id;

        self.update_buffers(
            Some((bytemuck::cast_slice(&vertices), glow::DYNAMIC_DRAW)),
            Some((bytemuck::cast_slice(indices), glow::DYNAMIC_DRAW)),
        );
        self.render(indices.len() as u32, 0);
    }
}
impl Renderable for EguiSceneNode {
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
                    Some(self.material
                        .uniforms
                        .get(&MaterialUniforms::EguiScreenSize).unwrap()),
                    screen_size.as_slice(),
                );
                unsafe {
                    let e = self.gl.get_error();
                    if e != glow::NO_ERROR {
                        println!("glerror {} at {} {} {}",e, file!(), line!(), column!());
                    }
                }
                // self.gl.uniform_1_u32(
                //     Some(self.material
                //         .uniforms
                //         .get(&MaterialUniforms::EguiEtexSampler).unwrap()),
                //     etex_sampler,
                // );
                unsafe {
                    let e = self.gl.get_error();
                    if e != glow::NO_ERROR {
                        println!("glerror {} at {} {} {}",e, file!(), line!(), column!());
                    }
                }
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

use egui::{epaint::Vertex, Pos2};

use crate::glc::renderer::buffer::{VertexBufferLayout, VertexBufferLayoutTrait};

#[derive(Debug, Clone, Copy)]
pub struct VertexRgba {
    pub position: Pos2,
    pub uv: Pos2,
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl VertexBufferLayoutTrait for VertexRgba {
    fn get_layout() -> VertexBufferLayout {
        let mut vbl = VertexBufferLayout::default();
        vbl.push_f32(2, false);
        vbl.push_f32(2, false);
        vbl.push_f32(4, false);
        vbl
    }
}
impl From<&Vertex> for VertexRgba {
    fn from(vert: &Vertex) -> Self {
        VertexRgba {
            position: vert.pos,
            uv: vert.uv,
            r: vert.color.r() as f32,
            g: vert.color.g() as f32,
            b: vert.color.b() as f32,
            a: vert.color.a() as f32,
        }
    }
}
impl From<Vertex> for VertexRgba {
    fn from(vert: Vertex) -> Self {
        VertexRgba {
            position: vert.pos,
            uv: vert.uv,
            r: vert.color.r() as f32,
            g: vert.color.g() as f32,
            b: vert.color.b() as f32,
            a: vert.color.a() as f32,
        }
    }
}

unsafe impl bytemuck::Zeroable for VertexRgba {
    fn zeroed() -> Self {
        unsafe { core::mem::zeroed() }
    }
}
unsafe impl bytemuck::Pod for VertexRgba {}
