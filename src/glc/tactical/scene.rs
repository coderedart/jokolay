use std::collections::BTreeMap;
use std::rc::Rc;

use glow::{Context, HasContext};
use nalgebra_glm::{Mat4, Vec3};

use crate::glc::gltypes::shader::ShaderProgram;
use crate::{glc::gltypes::material::MaterialUniforms};

use crate::glc::gltypes::{
    buffer::{Buffer, VertexBufferLayout, VertexBufferLayoutTrait},
    material::Material,
    scene::{Renderable, SceneNodeUniform},
    texture::Texture,
    vertex_array::VertexArrayObject,
};

use super::xmltypes::xml_marker::Marker;

#[derive(Debug, Clone, Copy)]
pub struct MarkerNode {
    pub position: [f32; 3],
    pub scale: f32,
    pub alpha: f32,
    pub fade_near: u32,
    pub fade_far: u32,
    pub min_size: u32,
    pub max_size: u32,
    pub color: [u8; 4],
    pub tex_layer: u32,
    pub tex_slot: u32,
}
impl VertexBufferLayoutTrait for MarkerNode {
    fn get_layout() -> VertexBufferLayout {
        let mut vbl = VertexBufferLayout::default();
        vbl.push_f32(3, false); // position
        vbl.push_f32(1, false); // scale
        vbl.push_f32(1, false); // alpha
        vbl.push_u32(1); // fade_near
        vbl.push_u32(1); // fade_far
        vbl.push_u32(1); // min_size
        vbl.push_u32(1); // max_size
        vbl.push_u8(4);  // color as u32
        vbl.push_u32(1); // tex_layer
        vbl.push_u32(1); // tex_slot
        vbl
    }
}
pub struct SceneNode {
    pub vao: VertexArrayObject,
    pub vb: Buffer,
    pub ib: Buffer,
    pub material: Material,
    pub batches: Vec<Batch>,
    pub gl: Rc<glow::Context>,
}
const SAMPLERS_ARRAY: [i32; 16] = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15];
impl SceneNode {
    pub fn new(gl: Rc<Context>) -> SceneNode {
    let vb = Buffer::new(gl.clone(), glow::ARRAY_BUFFER);
    let ib = Buffer::new(gl.clone(), glow::ELEMENT_ARRAY_BUFFER);
    let vao = VertexArrayObject::new(gl.clone());

    let program = ShaderProgram::new(
        gl.clone(),
        MARKER_VERTEX_SHADER_SRC,
        MARKER_FRAGMENT_SHADER_SRC,
        Some(MARKER_GEOMETRY_SHADER_SRC),
        
    );
    let mut uniforms = BTreeMap::new();
    unsafe {
        let u_camera_position = gl.get_uniform_location(program.id, "camera_position").unwrap();
        // let u_player_position = gl.get_uniform_location(program.id, "player_position").unwrap();
        let u_view_projection = gl.get_uniform_location(program.id, "view_projection").unwrap();
        uniforms.insert(MaterialUniforms::MarkerCamPos, u_camera_position);
        // uniforms.insert(MaterialUniforms::MarkerPlayerPos, u_player_position);
        uniforms.insert(MaterialUniforms::MarkerVP, u_view_projection);
    }
    let material = Material {
        program,
        uniforms,
        gl: gl.clone(),
    };

    let scene = SceneNode {
        vao,
        vb,
        ib,
        material,
        batches: Vec::new(),
        gl: gl.clone(),
    };
    scene.bind();
    let vblayout = MarkerNode::get_layout();
    vblayout.set_layout(gl);
    return scene;

    }
    pub fn draw(
        &self,
        markers: Option<&Vec<MarkerNode>>,
        vp: Mat4,
        cam_pos: Vec3,
        player_pos: Vec3,
    ) {
        self.bind();
        unsafe {
            // self.gl.disable(glow::FRAMEBUFFER_SRGB);
            self.gl.enable(glow::DEPTH_TEST);
            self.gl.enable(glow::BLEND);
            self.gl.blend_func(glow::ONE, glow::ONE_MINUS_SRC_ALPHA);
            self.gl.disable(glow::SCISSOR_TEST);
        }
        if let Some(m) = markers {
            self.update_buffers(Some((bytemuck::cast_slice(m), glow::DYNAMIC_DRAW)), None);
        }

        self.update_uniforms(SceneNodeUniform::MarkerSceneNodeUniform {
            vp,
            cam_pos,
            player_pos,
            samplers: &SAMPLERS_ARRAY,
        });
     

        // for each batch in batches
        // for (_index, batch) in self.batches.iter().enumerate() {
        //     // batches are organized based on texture index in material. so, first batch is 0-15 textures, second is 16-31 and so on.
        //     // we get the offset from where we start binding the 16 textures to the slots
        //     // we stick to 16 textures for now. but eventually shift to MAX_TEXTURE_IMAGE_UNITS to jump to 32 when possible
        //     assert!(batch.textures.len() < 16);
        //     //bind textures to their respective slots
        //     for (slot, texture) in batch.textures.iter().enumerate() {
        //         unsafe {
        //             self.gl.active_texture(glow::TEXTURE0 + slot as u32);
        //             texture.bind();
        //         }
        //     }
        //     // now that all textures for this batch are bound, we can draw the points for this batch using the offset/count of batch;
        //     self.render(batch.buffer_offset, batch.buffer_count);
        // }
        self.render(0, 200);
    }
}
pub struct Batch {
    pub buffer_offset: u32,
    pub buffer_count: u32,
    pub textures: Vec<Texture>,
}

impl Renderable for SceneNode {
    fn update_buffers(&self, vb: Option<(&[u8], u32)>, ib: Option<(&[u8], u32)>) {
        if let Some((data, usage)) = vb {
            self.vb.update(data, usage);
        }
        if let Some((_data, _usage)) = ib {
            unimplemented!();
        }
    }
    fn render(&self, offset: u32, count: u32) {
        unsafe {
            self.gl
                .draw_arrays(glow::POINTS, offset as i32, count as i32);
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
                self.gl.uniform_matrix_4_f32_slice(
                    Some(
                        &self
                            .material
                            .uniforms
                            .get(&MaterialUniforms::MarkerVP)
                            .unwrap(),
                    ),
                    false,
                    vp.as_slice(),
                );
                self.gl.uniform_3_f32_slice(
                    Some(
                        &self
                            .material
                            .uniforms
                            .get(&MaterialUniforms::MarkerCamPos)
                            .unwrap(),
                    ),
                    cam_pos.as_slice(),
                );
                // self.gl.uniform_3_f32_slice(
                //     Some(
                //         &self
                //             .material
                //             .uniforms
                //             .get(&MaterialUniforms::MarkerPlayerPos)
                //             .unwrap(),
                //     ),
                //     player_pos.as_slice(),
                // );
                // self.gl.uniform_4_i32_slice(
                //     Some(
                //         &self
                //             .material
                //             .uniforms
                //             .get(&MaterialUniforms::MarkerSampler0)
                //             .unwrap(),
                //     ),
                //     &samplers[0..4] ,
                // );
                // self.gl.uniform_4_i32_slice(
                //     Some(
                //         &self
                //             .material
                //             .uniforms
                //             .get(&MaterialUniforms::MarkerSampler4)
                //             .unwrap(),
                //     ),
                //     &samplers[4..8],
                // );
                // self.gl.uniform_4_i32_slice(
                //     Some(
                //         &self
                //             .material
                //             .uniforms
                //             .get(&MaterialUniforms::MarkerSampler8)
                //             .unwrap(),
                //     ),
                //     &samplers[8..12],
                // );
                // self.gl.uniform_4_i32_slice(
                //     Some(
                //         &self
                //             .material
                //             .uniforms
                //             .get(&MaterialUniforms::MarkerSampler12)
                //             .unwrap(),
                //     ),
                //     &samplers[12..16],
                // );
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
            n.position[1] = n.position[1] + p;
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

const MARKER_VERTEX_SHADER_SRC: &str = r#"
#version 330

layout (location = 0) in vec3 Position;
layout (location = 1) in float scale;
layout (location = 2) in float alpha;
layout (location = 3) in uint fade_near;
layout (location = 4) in uint fade_far;
layout (location = 5) in uint min_size;
layout (location = 6) in uint max_size;
layout (location = 7) in vec4 color;
layout (location = 8) in uint tex_layer;
layout (location = 9) in uint tex_slot;

out float v_scale;
out float v_alpha;


uniform vec3 camera_position;
// uniform vec3 player_position;

void main()
{
    // v_scale = scale;
    // if fade_near != 0 && fade_far != 0 {
    // float dist = distance(player_position, Position);
    // fade = (dist - fade_near)

    // }
    // dist = dist / (dist * 0.99);
    gl_Position = vec4(Position , 1.0);


}
"#;

const MARKER_FRAGMENT_SHADER_SRC: &str = r#"

#version 330                                                                        
                                                                                    
uniform sampler2D u_samplers;                                                        
                                                                                    
in vec2 tex_coords;                                                                   
out vec4 frag_color;                                                                 
                                                                                    
void main()                                                                         
{                                                                                   
    frag_color = texture2D(u_samplers, tex_coords);                                     
                                                                                    
    // if (frag_color.a == 0) {
    //     discard;                                                                    
    // }     
}
"#;

const MARKER_GEOMETRY_SHADER_SRC: &str = r#"
#version 330                                                                        
                                                                                    
layout(points) in;                                                                  
layout(triangle_strip) out;                                                         
layout(max_vertices = 4) out;                                                       


uniform mat4 view_projection;                                                                   
uniform vec3 camera_position;                                                            

                                                                           
out vec2 tex_coords;                                                                  
                                                                                    
void main()                                                                         
{             
    float billboard_height = 5.0;
    float billboard_width = 5.0;

    vec3 Pos = gl_in[0].gl_Position.xyz;                                            
    vec3 toCamera = normalize(camera_position - Pos);                                    
    vec3 up = vec3(0.0, 1.0, 0.0);                                                  
    vec3 right = cross(toCamera, up);                                               
                                                                                    
    Pos -= (right * billboard_width / 2.0);                                                           
    gl_Position = view_projection * vec4(Pos, 1.0);                                             
    tex_coords = vec2(0.0, 0.0);                                                      
    EmitVertex();                                                                   
                                                                                    
    Pos.y += billboard_height;                                                                   
    gl_Position = view_projection * vec4(Pos, 1.0);                                             
    tex_coords = vec2(0.0, 1.0);                                                      
    EmitVertex();                                                                   
                                                                                    
    Pos.y -= billboard_height;                                                                   
    Pos += (right * billboard_width);                                                                   
    gl_Position = view_projection * vec4(Pos, 1.0);                                             
    tex_coords = vec2(1.0, 0.0);                                                      
    EmitVertex();                                                                   
                                                                                    
    Pos.y += billboard_height;                                                                   
    gl_Position = view_projection * vec4(Pos, 1.0);                                             
    tex_coords = vec2(1.0, 1.0);                                                      
    EmitVertex();                                                                   
                                                                                    
    EndPrimitive();                                                                 
}             

"#;