use std::{rc::Rc, sync::Arc};

use egui::ClippedMesh;
use glm::{cross, make_vec3, normalize, Vec2};
use glow::{Context, HasContext};
use jokolink::mlink::MumbleLink;

use crate::{fm::{FileManager, VID}, painter::{
        egui_renderer::EguiGL,
        marker_renderer::{marker::MarkerVertex, MarkerGl},
        opengl::texture::TextureManager,
        trail_renderer::TrailGl,
    }, tactical::localtypes::manager::MarkerManager, window::glfw_window::GlfwWindow};

pub mod egui_renderer;
pub mod marker_renderer;
pub mod opengl;
pub mod trail_renderer;
pub struct Painter {
    pub egui_gl: EguiGL,
    pub marker_gl: MarkerGl,
    pub trail_gl: TrailGl,
    pub tm: TextureManager,
}

impl Painter {
    pub fn new(gl: Rc<Context>, t: Arc<egui::Texture>) -> Self {
        let egui_gl = EguiGL::new(gl.clone());

        let tm = TextureManager::new(gl.clone(), t);
        let marker_gl = MarkerGl::new(gl.clone());
        let trail_gl = TrailGl::new(gl);
        Self {
            egui_gl,
            marker_gl,
            trail_gl,
            tm,
        }
    }
    pub fn draw_egui(&mut self, meshes: Vec<ClippedMesh>, screen_size: Vec2, fm: &FileManager) {
        self.egui_gl
            .draw_meshes(meshes, screen_size, &mut self.tm, fm)
            .unwrap();
    }
    pub fn draw_markers(&mut self, mm: &mut MarkerManager, link: &MumbleLink, fm: &FileManager, window: &GlfwWindow) {
        self.marker_gl.draw_markers(&mut self.tm, mm, link, fm, window);
    }
    pub fn draw_trails(&mut self, mm: &mut MarkerManager, link: &MumbleLink, fm: &FileManager) {
        unsafe {
            self.marker_gl.gl.enable(glow::DEPTH_TEST);
            self.marker_gl
                .gl
                .blend_func(glow::SRC_ALPHA, glow::ONE_MINUS_SRC_ALPHA);
            self.marker_gl.gl.disable(glow::FRAMEBUFFER_SRGB);
        }
        self.trail_gl.bind();
        let camera_position = glm::Vec3::from(link.f_camera_position);
        let camera_dvec = camera_position + glm::Vec3::from(link.f_camera_front);
        let view = glm::look_at_lh(&camera_position, &camera_dvec, &glm::vec3(0.0, 1.0, 0.0));
        let projection = glm::perspective_fov_lh(link.identity.fov, 1920.0, 1080.0, 0.1, 5000.0);
        let vp = projection * view;
        for (pi, ci, tid) in mm.active_trails.iter() {
            let trail = mm.packs.get(*pi).unwrap().global_trails.get(tid).unwrap();
            let &icon_vid = &trail.texture.unwrap_or_else(|| {
                mm.packs
                    .get(*pi)
                    .unwrap()
                    .global_cats
                    .get(*ci)
                    .unwrap()
                    .inherited_template
                    .icon_file
                    .unwrap_or(VID(1))
            });
            let (slot, x, y, z) = self.tm.get_image(icon_vid, fm);
            let z = z as f32;
            let mut vertices: Vec<MarkerVertex> = vec![];
            // starting making a path quadrilateral between 2 nodes
            let nodes = &trail.tdata.nodes[1..];
            let mut previous_node = make_vec3(trail.tdata.nodes.get(0).unwrap());
            let trail_width: f32 = 1.0;
            for n in nodes {
                let present_node = make_vec3(n);
                let up = make_vec3(&[0.0, 1.0, 0.0]);
                let direction = normalize(&(&present_node - &previous_node));
                let right = normalize(&cross(&direction, &up));

                // left bottom vertex
                let lb = vp * (previous_node - (right * trail_width / 2.0)).push(1.0);
                // right bottom vertex
                let rb = vp * (previous_node + (right * trail_width / 2.0)).push(1.0);
                // left top vertex
                let lt = vp * (present_node - (right * trail_width / 2.0)).push(1.0);
                // right top vertex
                let rt = vp * (present_node + (right * trail_width / 2.0)).push(1.0);

                // first triangle
                // left top vertex
                vertices.push(MarkerVertex {
                    vpos: lt.into(),
                    tex_coords: [0.0, y, z],
                    alpha: 1.0,
                });
                //right top vertex
                vertices.push(MarkerVertex {
                    vpos: rt.into(),
                    tex_coords: [x, y, z],
                    alpha: 1.0,
                });
                // left bottom
                vertices.push(MarkerVertex {
                    vpos: lb.into(),
                    tex_coords: [0.0, 0.0, z],
                    alpha: 1.0,
                });
                // second triangle
                // duplicate the previous vertices for second triangle :)
                //left bottom
                vertices.push(MarkerVertex {
                    vpos: lb.into(),
                    tex_coords: [0.0, 0.0, z],
                    alpha: 1.0,
                });
                // right top
                vertices.push(MarkerVertex {
                    vpos: rt.into(),
                    tex_coords: [x, y, z],
                    alpha: 1.0,
                });
                // right bottom vertex
                vertices.push(MarkerVertex {
                    vpos: rb.into(),
                    tex_coords: [x, 0.0, z],
                    alpha: 1.0,
                });

                previous_node = present_node;
            }

            if vertices.is_empty() {
                continue;
            }
            self.trail_gl
                .vb
                .update(bytemuck::cast_slice(&vertices), glow::STREAM_DRAW);
            unsafe {
                self.trail_gl
                    .gl
                    .uniform_1_i32(Some(&self.trail_gl.u_sampler), slot as i32);
                self.trail_gl
                    .gl
                    .draw_arrays(glow::TRIANGLES, 0, vertices.len() as i32);
            }
        }
    }
}
