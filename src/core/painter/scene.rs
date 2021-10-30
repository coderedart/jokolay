use std::rc::Rc;

use crate::core::painter::{
    egui_renderer::{EguiMesh, EguiSceneState},
    opengl::buffer::Buffer,
};

/// This struct will store the data needed to render. when there's Some(data), it means we want to update the gl buffers.
/// when its None, it means there's no update and should continue as usual. and when there's Some(Vec::new()) like emptry data, it means we
/// want to clear the buffers and not draw that.
pub struct Scene {
    pub buffer_pool: Vec<Buffer>,
    pub egui_scene: EguiSceneState,
    pub gl: Rc<glow::Context>,
    // pub marker_2d_static: Instant,
    // pub marker_2d_dynamic: Instant,
    // pub trail_static: Instant,
    // pub trail_dynamic: Instant,
    // pub marker_3d_static: Instant,
    // pub marker_3d_dynamic: Instant,
}

impl Scene {
    pub fn new(gl: Rc<glow::Context>) -> Self {
        let egui_scene = EguiSceneState::new(gl.clone());
        Self {
            egui_scene,
            buffer_pool: vec![],
            gl,
        }
    }

    pub fn update_egui_scene_state(&mut self, meshes: Vec<EguiMesh>) {
        let mut previous_meshes = vec![];
        previous_meshes.append(&mut self.egui_scene.meshes);
        let mut previous_buffers: Vec<Buffer> = previous_meshes
            .into_iter()
            .map(|(_, b1, b2)| [b1, b2])
            .flatten()
            .collect();
        // make sure to have atleast the required number of buffers, or create/get new buffers
        if meshes.len() * 2 > previous_buffers.len() {
            let extra_buffers_required = meshes.len() * 2 - previous_buffers.len();
            for _ in 0..extra_buffers_required {
                if let Some(b) = self.buffer_pool.pop() {
                    previous_buffers.push(b);
                } else {
                    previous_buffers.push(Buffer::new(self.gl.clone()));
                }
            }
        }

        // update the data into the buffers
        for mesh in meshes {
            let vb = previous_buffers
                .pop()
                .expect("couldn't get vertex buffer to update egui data");
            let ib = previous_buffers
                .pop()
                .expect("couldn't get index buffer to update egui data");
            vb.update(bytemuck::cast_slice(&mesh.vertices), glow::STREAM_DRAW);
            ib.update(bytemuck::cast_slice(&mesh.indices), glow::STREAM_DRAW);
            self.egui_scene.meshes.push((mesh, vb, ib));
        }

        // if previous buffers are still remaining, lets empty them and deposit into pool
        for pb in previous_buffers {
            pb.update(&[], glow::STATIC_DRAW);
            self.buffer_pool.push(pb);
        }
    }
    pub fn draw_egui(&self) {
        self.egui_scene.draw();
    }
}
