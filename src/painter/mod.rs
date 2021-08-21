use std::{rc::Rc, sync::Arc};

use egui::ClippedMesh;
use glm::{cross, make_vec3, normalize, Vec2};
use glow::{Context, HasContext};
use jokolink::mlink::MumbleLink;

use crate::{
    fm::{FileManager, VID},
    painter::{
        egui_renderer::EguiGL,
        marker_renderer::{marker::MarkerVertex, MarkerGl},
        opengl::texture::TextureManager,
        trail_renderer::TrailGl,
    },
    tactical::localtypes::manager::MarkerManager,
};

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
    pub fn draw_markers(&mut self, mm: &mut MarkerManager, link: &MumbleLink, fm: &FileManager) {
        unsafe {
            self.marker_gl.gl.enable(glow::DEPTH_TEST);

            self.marker_gl
                .gl
                .blend_func(glow::SRC_ALPHA, glow::ONE_MINUS_SRC_ALPHA);
        }
        let mut triangles: Vec<Vec<MarkerVertex>> = vec![Vec::new(); TextureManager::NUM_OF_ARRAYS];
        let camera_position = glm::Vec3::from(link.f_camera_position);
        let camera_dvec = camera_position + glm::Vec3::from(link.f_camera_front);
        let view = glm::look_at_lh(&camera_position, &camera_dvec, &glm::vec3(0.0, 1.0, 0.0));
        let projection = glm::perspective_fov_lh(link.identity.fov, 1920.0, 1080.0, 0.1, 5000.0);
        let vp = projection * view;
        for (pi, ci, mid) in mm.active_markers.iter() {
            let marker = mm.packs.get(*pi).unwrap().global_pois.get(mid).unwrap();
            let position = marker.pos;

            let &vid = &marker.icon_file.unwrap_or_else(|| {
                mm.packs
                    .get(*pi)
                    .unwrap()
                    .global_cats
                    .get(*ci)
                    .unwrap()
                    .inherited_template
                    .icon_file
                    .unwrap_or(VID(0))
            });
            let (slot, x, y, z) = self.tm.get_image(vid, fm);
            let z = z as f32;
            // starting making a billboard from position and texture coordinates
            // let billboard_height: f32 = 1.0;
            let billboard_width: f32 = 2.0;
            let pos = make_vec3(&position);

            let to_camera = normalize(&(camera_position - pos));
            let up = make_vec3(&[0.0, 1.0, 0.0]);
            let right = cross(&to_camera, &up);
            // change position to left bottom vtx by moving it half width left and half width down.
            let pos = (pos - (right * billboard_width / 2.0)) - (up * billboard_width / 2.0);
            // left bottom vertex
            let vpvtx = vp * pos.push(1.0);
            // let lb = vpvtx.xyz() / vpvtx.w;
            let lb = vpvtx;
            // move it one width up to get left top
            let pos = pos + (up * billboard_width);
            // left top vertex
            let vpvtx = vp * pos.push(1.0);
            let lt = vpvtx;
            // let lt = vpvtx.xyz() / vpvtx.w;
            // move it one width right to get right top
            let pos = pos + (right * billboard_width);
            //right top vertex
            let vpvtx = vp * pos.push(1.0);
            let rt = vpvtx;
            // let rt = vpvtx.xyz() / vpvtx.w;
            // move it to one width down to get right bottom
            let pos = pos - (up * billboard_width);
            // right bottom vertex
            let vpvtx = vp * pos.push(1.0);
            let rb = vpvtx;
            // let rb = vpvtx.xyz() / vpvtx.w;

            {
                // Sometimes billboard spans across behind point of camera, as view matrix has discontinuity exactly before the camera.
                // it will cause HUGE billboards. so we check that the distance between top two points must be less than two times the billboard width (usually its one, but if it spans across entire screen, it can be 2.0)
                // if glm::distance(&lt, &rt) > 2.0 {
                //     continue;
                // }
            }

            // first triangle
            triangles[slot as usize].push(MarkerVertex {
                vpos: lb.into(),
                tex_coords: [0.0, 0.0, z],
                alpha: 1.0,
            });

            triangles[slot as usize].push(MarkerVertex {
                vpos: lt.into(),
                tex_coords: [0.0, y, z],
                alpha: 1.0,
            });

            triangles[slot as usize].push(MarkerVertex {
                vpos: rt.into(),
                tex_coords: [x, y, z],
                alpha: 1.0,
            });
            // second triangle
            // duplicate the previous vertices for second triangle :)
            //left bottom
            triangles[slot as usize].push(MarkerVertex {
                vpos: lb.into(),
                tex_coords: [0.0, 0.0, z],
                alpha: 1.0,
            });
            // right top
            triangles[slot as usize].push(MarkerVertex {
                vpos: rt.into(),
                tex_coords: [x, y, z],
                alpha: 1.0,
            });

            triangles[slot as usize].push(MarkerVertex {
                vpos: rb.into(),
                tex_coords: [x, 0.0, z],
                alpha: 1.0,
            });
        }
        self.marker_gl.bind();
        for (s, t) in triangles.into_iter().enumerate() {
            if t.is_empty() {
                continue;
            }
            self.marker_gl
                .vb
                .update(bytemuck::cast_slice(&t[..]), glow::STREAM_DRAW);
            unsafe {
                self.marker_gl
                    .gl
                    .uniform_1_i32(Some(&self.marker_gl.u_sampler), s as i32);
                self.marker_gl
                    .gl
                    .draw_arrays(glow::TRIANGLES, 0, t.len() as i32);
            }
        }
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
                    .unwrap_or(VID(0))
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
