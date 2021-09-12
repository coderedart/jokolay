use std::rc::Rc;

use egui::CtxRef;
use glm::{Vec3, cross, make_vec3, make_vec4, normalize};
use glow::{HasContext, NativeUniformLocation};
use jokolink::mlink::MumbleLink;

use crate::{core::{fm::{FileManager, RID}, painter::marker_renderer::marker::Quad, window::glfw_window::OverlayWindowConfig}, tactical::localtypes::manager::MarkerManager};

use self::marker::MarkerVertex;

use super::opengl::{buffer::{Buffer, VertexBufferLayoutTrait}, shader::ShaderProgram, texture::TextureManager, vertex_array::VertexArrayObject};


pub mod marker;
// use super::xmltypes::xml_marker::Marker;
pub struct MarkerGl {
    pub vao: VertexArrayObject,
    pub vb: Buffer,
    pub sp: ShaderProgram,
    pub u_sampler: NativeUniformLocation,
    pub znear: f32,
    /// should be roughly the length or half the length of the map. so that the icons on the other end can be just out of view frustum
    pub zfar: f32,
    pub gl: Rc<glow::Context>,
}
impl MarkerGl {
    pub fn new(gl: Rc<glow::Context>) -> Self {
        let layout = MarkerVertex::get_layout();
        let vao = VertexArrayObject::new(gl.clone(), layout);
        let vb = Buffer::new(gl.clone(), glow::ARRAY_BUFFER);
        let sp = ShaderProgram::new(gl.clone(), VERTEX_SHADER_SRC, FRAG_SHADER_SRC, None);
        let u_sampler = sp.get_uniform_id("sampler").unwrap();
        let marker_gl = Self {
            vao,
            vb,
            sp,
            u_sampler,
            zfar: 500.0,
            znear: 0.1,
            gl: gl.clone(),
        };
        marker_gl.bind();
        marker_gl
    }
    pub fn draw_markers(
        &self,
        tm: &mut TextureManager,
        mm: &mut MarkerManager,
        link: &MumbleLink,
        fm: &FileManager,
        wc: OverlayWindowConfig,
        ctx: CtxRef
    ) {
        unsafe {
            // self.gl.enable(glow::DEPTH_TEST);

            self.gl
                .blend_func(glow::SRC_ALPHA, glow::ONE_MINUS_SRC_ALPHA);
        }
        let mut billboards: Vec<Quad> = Vec::new();
        let camera_position = glm::Vec3::from(link.f_camera_position);
        let camera_dvec = camera_position + glm::Vec3::from(link.f_camera_front);
        let view = glm::look_at_lh(&camera_position, &camera_dvec, &glm::vec3(0.0, 1.0, 0.0));
        let projection = glm::perspective_fov_lh(link.identity.fov, wc.framebuffer_width as f32, wc.framebuffer_height as f32, self.znear, self.zfar);
        let vp = projection * view;
        for (pi, ci, mid) in mm.active_markers.iter() {
            let pack = mm.packs.get(*pi).unwrap();
            let marker = pack.global_pois.get(mid).unwrap();
            let cat = pack.global_cats.get(*ci).unwrap();
            let position = marker.pos;

            let &vid = &marker
                .icon_file
                .unwrap_or_else(|| cat.inherited_template.icon_file.unwrap_or(RID::MarkerTexture));
            
            let tc = tm.get_image(vid, fm, ctx);
            if let Some(q) = Quad::new(
                marker,
                cat,
                link,
                view,
                projection,
                wc,
                tc,
                self.znear,
                self.zfar

            ) {
                billboards[tc.0 as usize].push(q);
            }
        }
        self.bind();
        for (s, t) in billboards.into_iter().enumerate() {
            if t.is_empty() {
                continue;
            }
            self.vb
                .update(bytemuck::cast_slice(&t[..]), glow::STREAM_DRAW);
            unsafe {
                self.gl.uniform_1_i32(Some(&self.u_sampler), s as i32);
                self.gl.draw_arrays(glow::TRIANGLES, 0, 6 * t.len() as i32);
            }
        }
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
