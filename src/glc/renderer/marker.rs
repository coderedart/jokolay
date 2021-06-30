use std::rc::Rc;

use glow::HasContext;
use nalgebra_glm::{Mat4, Vec3};

use crate::{glc::renderer::material::MaterialUniforms, gw::marker::Marker};

use super::{buffer::{Buffer, VertexBufferLayout, VertexBufferLayoutTrait}, material::Material, scene::{Renderable, SceneNodeUniform}, vertex_array::VertexArrayObject};

#[derive(Debug, Clone, Copy)]
pub struct MarkerNode {
    pub position: [f32; 3],
    pub scale: f32,
    pub alpha: f32,
    pub height_offset: f32,
    pub fade_near: u32,
    pub fade_far: u32,
    pub min_size: u32,
    pub max_size: u32,
    pub color: [u8; 4],
    pub tex_layer: u32,
    pub tex_slot: u32,
}
pub struct MarkerSceneNode {
    pub vao: VertexArrayObject,
    pub vb: Buffer,
    pub ib: Buffer,
    pub material: Material,
    pub batches: Vec<Batch>,
    pub gl: Rc<glow::Context>,
}
const SAMPLERS_ARRAY: [u32; 16] = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15];
impl MarkerSceneNode {
    pub fn draw(&self, markers: Option<&Vec<MarkerNode>>, vp: Mat4, cam_pos: Vec3, player_pos: Vec3) {
        self.bind();
        if let Some(m) = markers {
            self.update_buffers(Some((bytemuck::cast_slice(m), glow::DYNAMIC_DRAW)), None);
        }

        self.update_uniforms(SceneNodeUniform::MarkerSceneNodeUniform{
                vp,
                cam_pos,
                player_pos,
                samplers: &SAMPLERS_ARRAY,
            });
        // for each batch in batches
        for (index, batch) in self.batches.iter().enumerate() {
            // batches are organized based on texture index in material. so, first batch is 0-15 textures, second is 16-31 and so on.
            // we get the offset from where we start binding the 16 textures to the slots
            // we stick to 16 textures for now. but eventually shift to MAX_TEXTURE_IMAGE_UNITS to jump to 32 when possible
            let texture_offset = 16 * index;
            //bind textures to their respective slots
            for (slot, t) in self.material.textures[texture_offset..texture_offset + 16].iter().enumerate() {
                unsafe {
                    self.gl.active_texture(glow::TEXTURE0 + slot as u32);
                    t.bind();
                }
            }
            // now that all textures for this batch are bound, we can draw the points for this batch using the offset/count of batch;
            self.render( batch.buffer_offset , batch.buffer_count );
            
            

        }
    }
    
}
pub struct Batch {
    pub buffer_offset: u32,
    pub buffer_count: u32,    
}

impl Renderable for MarkerSceneNode {
    fn update_buffers(&self, vb: Option<(&[u8], u32)>, ib: Option<(&[u8], u32)>) {
        if let Some((data, usage)) = vb {
            self.vb.update(data, usage);
        }
        if let Some((_data, _usage)) = ib {
            unimplemented!();
        }
    }
    fn render(&self, count: u32, offset: u32) {
        unsafe {
            self.gl.draw_arrays(glow::POINTS, offset as i32, count as i32);
        }
    }

    fn bind(&self) {
        self.vao.bind();
        self.vb.bind();
        self.material.bind();
    }

    fn update_uniforms(&self, uniform_data: super::scene::SceneNodeUniform) {
        match uniform_data {
            super::scene::SceneNodeUniform::MarkerSceneNodeUniform {
                vp,
                cam_pos,
                player_pos,
                samplers,
            } => unsafe {
                self.gl.uniform_matrix_4_f32_slice(Some(&self.material.uniforms.get(&MaterialUniforms::MarkerVP).unwrap()), false, vp.as_slice());
                self.gl.uniform_3_f32_slice(Some(&self.material.uniforms.get(&MaterialUniforms::MarkerCamPos).unwrap()), cam_pos.as_slice());
                self.gl.uniform_3_f32_slice(Some(&self.material.uniforms.get(&MaterialUniforms::MarkerPlayerPos).unwrap()), player_pos.as_slice());
                self.gl.uniform_4_u32_slice(Some(&self.material.uniforms.get(&MaterialUniforms::MarkerSampler0).unwrap()), &samplers[0..4] );
                self.gl.uniform_4_u32_slice(Some(&self.material.uniforms.get(&MaterialUniforms::MarkerSampler4).unwrap()), &samplers[4..8] );
                self.gl.uniform_4_u32_slice(Some(&self.material.uniforms.get(&MaterialUniforms::MarkerSampler8).unwrap()), &samplers[8..12] );
                self.gl.uniform_4_u32_slice(Some(&self.material.uniforms.get(&MaterialUniforms::MarkerSampler12).unwrap()), &samplers[12..16] );
            },
            super::scene::SceneNodeUniform::EguiSceneNodeUniform {
                screen_size: _,
                u_sampler: _,
            } => unimplemented!(),
        }
    }

    fn unbind(&self) {
        self.vao.unbind();
        self.vb.unbind();
        self.material.unbind();
    }
}

impl Default for MarkerNode {
    fn default() -> Self {
        MarkerNode {
            position: [0.0, 0.0, 0.0],
            scale: 1.0,
            alpha: 1.0,
            height_offset: 1.5,
            fade_near: 0,
            fade_far: u32::MAX,
            min_size: 0,
            max_size: u32::MAX,
            color: [0, 0, 0, 0],
            tex_layer: 0,
            tex_slot: 0,
        }
    }
}

impl From<&Marker> for MarkerNode {
    fn from(m: &Marker) -> Self {
        let mut n = MarkerNode::default();
        n.position = [m.xpos, m.ypos, m.zpos];
        if let Some(p) = m.map_display_size {
            n.scale = p as f32;
        }
        if let Some(p) = m.alpha {
            n.alpha = p;
        }
        if let Some(p) = m.height_offset {
            n.height_offset = p;
        }
        if let Some(p) = m.fade_near {
            n.fade_near = p;
        }
        if let Some(p) = m.fade_far {
            n.fade_far = p;
        }
        if let Some(p) = m.min_size {
            n.min_size = p;
        }
        if let Some(p) = m.max_size {
            n.max_size = p;
        }
        if let Some(p) = m.color {
            n.color = p.to_ne_bytes();
        }

        n
    }
}

impl VertexBufferLayoutTrait for MarkerNode {
    fn get_layout() -> VertexBufferLayout {
        let mut vbl = VertexBufferLayout::default();
        vbl.push_f32(3, false);
        vbl.push_f32(1, false);
        vbl.push_f32(1, false);
        vbl.push_f32(1, false);
        vbl.push_u32(1);
        vbl.push_u32(1);
        vbl.push_u32(1);
        vbl.push_u32(1);
        vbl.push_u32(1); //color as u32
        vbl.push_u32(1);
        vbl.push_u32(1);
        vbl
    }
}

unsafe impl bytemuck::Zeroable for MarkerNode {
    fn zeroed() -> Self {
        unsafe { core::mem::zeroed() }
    }
}
unsafe impl bytemuck::Pod for MarkerNode {}

/*
vertex_position,
tex_id,
tex_coords,

*/
