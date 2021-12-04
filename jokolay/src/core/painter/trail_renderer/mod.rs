use std::rc::Rc;
use std::sync::Arc;
use std::time::Instant;

use egui::CtxRef;
use glm::cross;
use glm::make_vec3;
use glm::normalize;
use glow::HasContext;
use glow::NativeUniformLocation;
use jokolink::mlink::MumbleLink;
use uuid::Uuid;

use crate::core::fm::FileManager;
use crate::core::fm::RID;
use crate::tactical::localtypes::manager::MarkerManager;

use self::trail::TrailMeshBuffer;

use super::opengl::buffer::Buffer;
use super::opengl::buffer::VertexBufferLayout;
use super::opengl::buffer::VertexBufferLayoutTrait;
use super::opengl::shader::ShaderProgram;
use super::opengl::texture::TextureManager;
use super::opengl::vertex_array::VertexArrayObject;

pub mod trail;
/// A Renderer that can draw and buffer trail meshes in gpu. 
/// 
pub struct TrailGl {
    /// with DSA, we can get away with one single VAO by using buffer binding points
    pub vao: VertexArrayObject,
    /// This is the cache of buffers. we will only retain a mesh if it is drawn in this frame. 
    pub vb: Vec<TrailMeshBuffer>,
    pub sp: ShaderProgram,
    pub u_anim_speed: NativeUniformLocation,
    pub u_color: NativeUniformLocation,
    pub u_alpha: NativeUniformLocation,
    pub u_fade_near: NativeUniformLocation,
    pub u_fade_far: NativeUniformLocation,
    pub u_sampler: NativeUniformLocation,
    pub u_vp: NativeUniformLocation,
    pub gl: Rc<glow::Context>,
}
impl TrailGl {
    pub fn new(gl: Rc<glow::Context>) -> Self {
        let vao = VertexArrayObject::new(gl.clone(), TrailVertex::get_layout());
        let sp = ShaderProgram::new(gl.clone(), VERTEX_SHADER_SRC, FRAG_SHADER_SRC, None);
        let u_sampler = sp.get_uniform_id("sampler").unwrap();
        Self {
            vao,
            vb: vec![],
            sp,
            u_sampler,
            gl: gl.clone(),
        }

    }
    pub fn draw_trails(
        &mut self,
        mm: &mut MarkerManager,
        link: &MumbleLink,
        fm: &FileManager,
        tm: &mut TextureManager,
        ctx: CtxRef,
    ) {
        unsafe {
            self.gl.enable(glow::DEPTH_TEST);
            self.gl
                .blend_func(glow::SRC_ALPHA, glow::ONE_MINUS_SRC_ALPHA);
            self.gl.disable(glow::FRAMEBUFFER_SRGB);
        }
        self.bind();
        let camera_position = glm::Vec3::from(link.f_camera_position);
        let camera_dvec = camera_position + glm::Vec3::from(link.f_camera_front);
        let view = glm::look_at_lh(&camera_position, &camera_dvec, &glm::vec3(0.0, 1.0, 0.0));
        let projection = glm::perspective_fov_lh(link.identity.fov, 1920.0, 1080.0, 0.1, 5000.0);
        let vp = projection * view;
        let mut new_vec = vec![];
        for (t, b) in self.vb {
            for &(pi, ci, tid) in mm.active_trails.iter() {
                if t == tid {
                    new_vec.push((t, b));
                    break;
                }
                
            }
        }
        self.vb = new_vec;
        for &(pi, ci, tid) in mm.active_trails.iter() {
            let b = self.vb.iter().find(|(t, b)| t == tid  );
            match b {
                Some((id, buffer)) => buffer.bind(),
                None => {
                    let buffer = Buffer::new(self.gl.clone(), glow::ARRAY_BUFFER);
                    
                },
            }
            
        }
        
    }
    pub fn trail_generate_vertices(
        tid: Uuid,
        pi: usize,
        ci: usize,
        tm: &mut TextureManager,
        ctx: CtxRef,
        fm: &FileManager,
        mm: &mut MarkerManager,
    ) -> Vec<TrailVertex> {
        let trail = mm.packs.get(pi).unwrap().global_trails.get(&tid).unwrap();
        let &icon_vid = &trail.texture.unwrap_or_else(|| {
            mm.packs
                .get(pi)
                .unwrap()
                .global_cats
                .get(ci)
                .unwrap()
                .inherited_template
                .icon_file
                .unwrap_or(RID::TrailTexture)
        });
        let (slot, x, y, z) = tm.get_image(icon_vid, fm, ctx);
        let z = z as f32;
        let mut vertices: Vec<TrailVertex> = vec![];
        // starting making a path quadrilateral between 2 nodes
        let nodes = &trail.tdata.nodes[1..];
        let mut previous_node = make_vec3(trail.tdata.nodes.get(0).unwrap());
        let trail_width: f32 = 1.0;
        for n in nodes {
            let present_node = make_vec3(n);
            let up = make_vec3(&[0.0, 1.0, 0.0]);
            let direction = normalize(&(&present_node - &previous_node));
            let right = normalize(&cross(&direction, &up));
            let distance = glm::distance(&present_node, &present_node);

            let x = f32::max(x, f32::round(distance));
            // left bottom vertex
            let lb = previous_node - (right * trail_width / 2.0);
            // right bottom vertex
            let rb = previous_node + (right * trail_width / 2.0);
            // left top vertex
            let lt = present_node - (right * trail_width / 2.0);
            // right top vertex
            let rt = present_node + (right * trail_width / 2.0);

            // first triangle
            // left top vertex
            vertices.push(TrailVertex {
                vpos: lt.into(),
                tex_coords: [0.0, y, z],
            });
            //right top vertex
            vertices.push(TrailVertex {
                vpos: rt.into(),
                tex_coords: [x, y, z],
            });
            // left bottom
            vertices.push(TrailVertex {
                vpos: lb.into(),
                tex_coords: [0.0, 0.0, z],
            });
            // second triangle
            // duplicate the previous vertices for second triangle :)
            //left bottom
            vertices.push(TrailVertex {
                vpos: lb.into(),
                tex_coords: [0.0, 0.0, z],
            });
            // right top
            vertices.push(TrailVertex {
                vpos: rt.into(),
                tex_coords: [x, y, z],
            });
            // right bottom vertex
            vertices.push(TrailVertex {
                vpos: rb.into(),
                tex_coords: [x, 0.0, z],
            });

            previous_node = present_node;
        }
        vertices
    }
    pub fn bind(&self) {
        self.vao.bind();
        self.sp.bind();
    }

    pub fn unbind(&self) {
        self.vao.unbind();
        self.sp.unbind();
    }
}
pub const VERTEX_SHADER_SRC: &str = include_str!("shader.vs");
pub const FRAG_SHADER_SRC: &str = include_str!("shader.fs");

