use std::{
    collections::{BTreeMap, HashMap},
    rc::Rc,
};

use anyhow::Context as _;
use egui::{ClippedMesh, Rect, TextureId};
use glow::{Context, HasContext, UNSIGNED_INT};
use nalgebra_glm::Vec2;

use crate::glc::renderer::{buffer::Buffer, material::{Material, MaterialUniforms}, scene::{Renderable, SceneNodeUniform}, shader::ShaderProgram, texture::Texture, vertex_array::VertexArrayObject};

pub struct EguiScene {
    pub vao: VertexArrayObject,
    pub vb: Buffer,
    pub ib: Buffer,
    pub material: Material,
    pub texture_versions: HashMap<TextureId, Texture>,
    pub gl: Rc<glow::Context>,

}

impl EguiScene {
    pub fn new(gl: Rc<Context>) -> EguiScene {
        let vao = VertexArrayObject::new(gl.clone());
        let vb = Buffer::new(gl.clone(), glow::ARRAY_BUFFER);
        let ib = Buffer::new(gl.clone(), glow::ELEMENT_ARRAY_BUFFER);
        let program = ShaderProgram::new(
            gl.clone(),
            EGUI_VERTEX_SHADER_SRC,
            EGUI_FRAGMENT_SHADER_SRC,
            None,
        );

        let mut uniforms: BTreeMap<MaterialUniforms, u32> = BTreeMap::new();

        unsafe {
            let u_sampler = gl.get_uniform_location(program.id, "u_sampler").unwrap();
            let screen_size = gl.get_uniform_location(program.id, "screen_size").unwrap();
            uniforms.insert(MaterialUniforms::EguiScreenSize, screen_size);
            uniforms.insert(MaterialUniforms::EguiSampler, u_sampler);
        }

        let material = Material {
            program,
            uniforms,
            gl: gl.clone(),
        };

        let egui_scene_node = EguiScene {
            vao,
            vb,
            ib,
            material,
            gl: gl.clone(),
            texture_versions: HashMap::new(),
        };
        egui_scene_node.bind();
        let layout = VertexRgba::get_layout();
        layout.set_layout(gl.clone());

        return egui_scene_node;
    }

    pub fn draw_meshes(&mut self, meshes: &Vec<ClippedMesh>, screen_size: Vec2, u_sampler: u32) -> anyhow::Result<()> {
        self.bind();

        unsafe {
            self.gl.enable(glow::FRAMEBUFFER_SRGB);
            self.gl.disable(glow::DEPTH_TEST);
            self.gl.enable(glow::BLEND);
            self.gl.blend_func(glow::ONE, glow::ONE_MINUS_SRC_ALPHA);
            self.gl.enable(glow::SCISSOR_TEST);
        }

        for clipped_mesh in meshes {
            self.update_uniforms(SceneNodeUniform::EguiSceneNodeUniform {
                screen_size,
                u_sampler,
            });
            unsafe {
                let e = self.gl.get_error();
                if e != glow::NO_ERROR {
                    println!("glerror {} at {} {} {}", e, file!(), line!(), column!());
                }
            }
            self.draw_mesh(clipped_mesh, screen_size)?;
        }


        Ok(())
    }
    pub fn draw_mesh(&mut self, clipped_mesh: &ClippedMesh, screen_size: Vec2) -> anyhow::Result<()> {
        Self::set_scissor(clipped_mesh.0, self.gl.clone(), screen_size);
        let mesh = &clipped_mesh.1;
        let vertices: Vec<VertexRgba> = mesh.vertices.iter().map(|v| VertexRgba::from(v)).collect();
        let indices = &mesh.indices;

        self.update_buffers(
            Some((bytemuck::cast_slice(&vertices), glow::DYNAMIC_DRAW)),
            Some((bytemuck::cast_slice(indices), glow::DYNAMIC_DRAW)),
        );

        self.texture_versions.get(&mesh.texture_id).context("no such texture to bind in egui draw call")?.bind();
        self.render(indices.len() as u32, 0);
        Ok(())
    }
    fn set_scissor(clip_rect: Rect, gl: Rc<glow::Context>, screen_size: Vec2)  {
        //clip rectangle copy pasted from glium
        let clip_min_x = clip_rect.min.x;
        let clip_min_y = clip_rect.min.y;
        let clip_max_x = clip_rect.max.x;
        let clip_max_y = clip_rect.max.y;

        // Make sure clip rect can fit within a `u32`:
        let clip_min_x = clip_min_x.clamp(0.0, screen_size.x );
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
                (screen_size.y - clip_max_y as f32)as i32,
                (clip_max_x - clip_min_x) as i32,
                (clip_max_y - clip_min_y) as i32,
            );
        }
    }
}
impl Renderable for EguiScene {
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
                u_sampler,
            } => unsafe {
                self.gl.uniform_2_f32_slice(
                    Some(
                        self.material
                            .uniforms
                            .get(&MaterialUniforms::EguiScreenSize)
                            .unwrap(),
                    ),
                    screen_size.as_slice(),
                );
                //sampler uniforms are i32
                self.gl.uniform_1_i32(
                    Some(
                        self.material
                            .uniforms
                            .get(&MaterialUniforms::EguiSampler)
                            .unwrap(),
                    ),
                    u_sampler as i32,
                );
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
        self.material.unbind();
    }
}

use egui::{epaint::Vertex, Pos2};

use crate::glc::renderer::buffer::{VertexBufferLayout, VertexBufferLayoutTrait};

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
        vbl.push_u8(4);
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

const EGUI_FRAGMENT_SHADER_SRC: &str = r#"
#version 450

uniform sampler2D u_sampler;

in vec2 v_tc;
in vec4 v_color;
out vec4 f_color;

void main() {
  f_color =  v_color * texture(u_sampler, v_tc) ;
}"#;

const EGUI_VERTEX_SHADER_SRC: &str = r#"
#version 450

layout(location = 0) in vec2 pos;
layout(location = 1) in vec2 tc;
layout(location = 2) in vec4 color;

out vec2 v_tc;
out vec4 v_color;

uniform vec2 screen_size;

vec3 linear_from_srgb(vec3 srgb) {
  bvec3 cutoff = lessThan(srgb, vec3(10.31475));
  vec3 lower = srgb / vec3(3294.6);
  vec3 higher = pow((srgb + vec3(14.025)) / vec3(269.025), vec3(2.4));
  return mix(higher, lower, vec3(cutoff));
}

vec4 linear_from_srgba(vec4 srgba) {
  return vec4(linear_from_srgb(srgba.rgb), srgba.a / 255.0);
}

void main() {

    gl_Position = vec4(2.0 * pos.x / screen_size.x - 1.0, 1.0 - 2.0 * pos.y / screen_size.y, 0.0, 1.0);
    v_tc = tc;
    v_color = linear_from_srgba(color);

}
"#;
