use std::{rc::Rc, sync::Arc};

use egui::ClippedMesh;
use glm::{cross, make_vec3, normalize, Vec2};
use glow::{Context, HasContext};
use jokolink::mlink::MumbleLink;

use crate::{
    painter::{
        egui_renderer::EguiGL,
        marker_renderer::{marker::Vertex, MarkerGl},
        opengl::texture::TextureManager,
    },
    tactical::localtypes::manager::MarkerManager,
};

pub mod egui_renderer;
pub mod marker_renderer;
pub mod opengl;
pub struct Painter {
    pub egui_gl: EguiGL,
    pub marker_gl: MarkerGl,
    pub tm: TextureManager,
}

impl Painter {
    pub fn new(gl: Rc<Context>, t: Arc<egui::Texture>) -> Self {
        let egui_gl = EguiGL::new(gl.clone());

        let tm = TextureManager::new(gl.clone(), t);
        let marker_gl = MarkerGl::new(gl.clone());
        Self {
            egui_gl,
            marker_gl,
            tm,
        }
    }
    pub fn draw_egui(&mut self, meshes: Vec<ClippedMesh>, screen_size: Vec2) {
        self.egui_gl
            .draw_meshes(meshes, screen_size, &mut self.tm)
            .unwrap();
    }
    pub fn draw_markers(&mut self, mm: &mut MarkerManager, link: &MumbleLink) {
        unsafe {
            self.marker_gl.gl.enable(glow::DEPTH_TEST);
            self.marker_gl.gl.disable(glow::FRAMEBUFFER_SRGB);
            
        }
        let mut triangles: Vec<Vec<Vertex>> = vec![Vec::new(); TextureManager::NUM_OF_ARRAYS];
        let camera_position = glm::Vec3::from(link.f_camera_position);
        let camera_dvec = camera_position + glm::Vec3::from(link.f_camera_front);
        let view = glm::look_at_lh(&camera_position, &camera_dvec, &glm::vec3(0.0, 1.0, 0.0));
        let projection = glm::perspective_fov_lh(link.identity.fov, 1920.0, 1080.0, 1.0, 10000.0);
        let vp = projection * view;
        for (pi, ci, mid) in mm.active_markers.iter() {
            let marker = mm.packs.get(*pi).unwrap().global_pois.get(mid).unwrap();
            let position = [marker.xpos, marker.ypos, marker.zpos];
            let mut tex_path = mm
                .packs
                .get(*pi)
                .unwrap()
                .path
                .to_str()
                .unwrap()
                .to_string();
            tex_path = tex_path + "/" + &marker.icon_file.clone().unwrap_or_else(|| {
                mm.packs.get(*pi).unwrap().global_cats.get(*ci).unwrap().inherited_template.icon_file.clone().unwrap_or_else(|| { "tex.png".to_string()})
                
            });
            let (slot, x, y, z) = self.tm.get_image(&tex_path);
            let z = z as f32;
            // starting making a billboard from position and texture coordinates
            // let billboard_height: f32 = 1.0;
            let billboard_width: f32 = 1.0;
            let pos = make_vec3(&position);

            let to_camera = normalize(&(camera_position - pos));
            let up = make_vec3(&[0.0, 1.0, 0.0]);
            let right = cross(&to_camera, &up);
            // left bottom vertex
            let vtx = pos - (right * billboard_width / 2.0);
            let mut vtx = vtx.push(1.0);
            let vpvtx = vp * vtx;
            let lb = vpvtx.xyz() / vpvtx.w;
            if lb.x > 2.0 || lb.x < -2.0 {
                continue;
            }
            triangles[slot as usize].push(Vertex {
                vpos: lb.into(),
                tex_coords: [0.0, 0.0, z],
            });
            // left top vertex
            vtx.y = vtx.y + billboard_width;
            let vpvtx = vp * vtx;
            let lt = vpvtx.xyz()/ vpvtx.w;
            triangles[slot as usize].push(Vertex {
                vpos: lt.into(),
                tex_coords: [0.0, y, z],
            });
            //right top vertex
            vtx = (vtx.xyz() + (right * billboard_width )).push(1.0);
            let vpvtx = vp * vtx;
            let rt = vpvtx.xyz()/ vpvtx.w;
            triangles[slot as usize].push(Vertex {
                vpos: rt.into(),
                tex_coords: [x, y, z],
            });
            // duplicate the previous vertices for second triangle :)
            //left bottom
            triangles[slot as usize].push(Vertex {
                vpos: lb.into(),
                tex_coords: [0.0, 0.0, z],
            });
            // right top
            triangles[slot as usize].push(Vertex {
                vpos: rt.into(),
                tex_coords: [x, y, z],
            });
            // right bottom vertex
            vtx.y = vtx.y - billboard_width;
            let vpvtx = vp * vtx;
            let rb = vpvtx.xyz()/ vpvtx.w;
            triangles[slot as usize].push(Vertex {
                vpos: rb.into(),
                tex_coords: [x, 0.0, z],
            });
            // triangles[slot as usize].push(Vertex {
            //         vpos: [0.0, 0.0, 0.5],
            //         tex_coords: [0.0, 0.0, z],
            //     });
            //     triangles[slot as usize].push(Vertex {
            //         vpos: [1.0, 0.0, 0.5],
            //         tex_coords: [x, 0.0, z],
            //     });
            //     triangles[slot as usize].push(Vertex {
            //         vpos: [0.5, 1.0, 0.5],
            //         tex_coords: [x / 0.5, y, z],
            //     });
        }
        self.marker_gl.bind();
        for (s, t) in triangles.into_iter().enumerate() {
            if t.is_empty() {
                continue;
            }
            self.marker_gl.vb.update(bytemuck::cast_slice(&t[..]), glow::STREAM_DRAW);
            unsafe {
                self.marker_gl.gl.uniform_1_i32(Some(&self.marker_gl.u_sampler), s as i32);
                self.marker_gl.gl.draw_arrays(glow::TRIANGLES, 0, t.len() as i32 );
            }
        }
    }
}
