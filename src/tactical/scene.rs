use std::{collections::BTreeMap, rc::Rc};

use glow::{Context, HasContext};

use nalgebra_glm::{Mat4, Vec3};

use crate::{
    gl_error,
    gltypes::{
        buffer::{Buffer, VertexBufferLayout},
        shader::ShaderProgram,
        texture::Texture,
        vertex_array::VertexArrayObject,
    },
};

use super::xmltypes::xml_marker::Marker;

// use super::xmltypes::xml_marker::Marker;

#[derive(Debug, Clone, Copy)]
pub struct MarkerNode {
    pub position: [f32; 3],
    pub tex_layer: u32,
    pub tex_slot: u32,
}

impl Default for MarkerNode {
    fn default() -> Self {
        MarkerNode {
            position: [0.0, 0.0, 0.0],
            tex_layer: 0,
            tex_slot: 0,
        }
    }
}

impl From<&Marker> for MarkerNode {
    fn from(m: &Marker) -> Self {
        let mut n = MarkerNode::default();
        n.position = [m.xpos, m.ypos, m.zpos];
        n
    }
}

unsafe impl bytemuck::Zeroable for MarkerNode {
    fn zeroed() -> Self {
        unsafe { core::mem::zeroed() }
    }
}
unsafe impl bytemuck::Pod for MarkerNode {}

impl MarkerNode {
    fn get_layout() -> VertexBufferLayout {
        let mut vbl = VertexBufferLayout::default();
        vbl.push_f32(3, false); // position
        vbl.push_u32(1, false); // tex_layer
        vbl.push_u32(1, false); // tex_slot
                                //         vbl.push_f32(1, false); // scale
                                //         vbl.push_f32(1, false); // alpha
                                //         vbl.push_u32(1); // fade_near
                                //         vbl.push_u32(1); // fade_far
                                //         vbl.push_u32(1); // min_size
                                //         vbl.push_u32(1); // max_size
                                //         vbl.push_u8(4); // color as u32

        vbl
    }
}
pub struct MarkerScene {
    pub vao: VertexArrayObject,
    pub vb: Buffer,
    pub sp: ShaderProgram,
    pub batches: Vec<Batch>,
    pub u_camera_position: u32,
    pub u_view_projection: u32,
    pub gl: Rc<glow::Context>,
    pub max_texture_units: i32,
}
#[derive(Debug)]
pub struct Batch {
    pub vb_offset: usize,
    pub vb_count: usize,
    pub textures: Vec<Texture>,
}

// const SAMPLERS_ARRAY: [i32; 16] = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15];
impl MarkerScene {
    pub fn new(gl: Rc<Context>) -> MarkerScene {
        let vb = Buffer::new(gl.clone(), glow::ARRAY_BUFFER);
        let vao = VertexArrayObject::new(gl.clone());

        let sp = ShaderProgram::new(
            gl.clone(),
            MARKER_VERTEX_SHADER_SRC,
            MARKER_FRAGMENT_SHADER_SRC,
            Some(MARKER_GEOMETRY_SHADER_SRC),
        );
        let u_camera_position: u32;
        let u_view_projection: u32;
        let max_texture_units: i32;
        unsafe {
            u_camera_position = gl.get_uniform_location(sp.id, "camera_position").unwrap();
            u_view_projection = gl.get_uniform_location(sp.id, "view_projection").unwrap();
            max_texture_units = gl.get_parameter_i32(glow::MAX_TEXTURE_IMAGE_UNITS);
        }
        let scene = MarkerScene {
            vao,
            vb,
            sp,
            batches: Vec::new(),
            u_camera_position,
            u_view_projection,
            max_texture_units,
            gl: gl.clone(),
        };
        scene.bind();
        let vblayout = MarkerNode::get_layout();
        vblayout.set_layout(gl);
        return scene;
    }
    pub fn draw(&self, view_projection: Mat4, camera_position: Vec3) {
        unsafe {
            self.gl.disable(glow::FRAMEBUFFER_SRGB);
            self.gl.enable(glow::DEPTH_TEST);
            self.gl
                .blend_func(glow::SRC_ALPHA, glow::ONE_MINUS_SRC_ALPHA);
        }
        self.bind();
        self.update_uniforms(view_projection, camera_position);
        for batch in self.batches.iter() {
            // batches are organized based on texture index in material. so, first batch is 0-15 textures, second is 16-31 and so on.
            // we get the offset from where we start binding the 16 textures to the slots
            // we stick to 16 textures for now. but eventually shift to MAX_TEXTURE_IMAGE_UNITS to jump to 32 when possible
            //bind textures to their respective slots
            for (slot, texture) in batch.textures.iter().enumerate() {
                unsafe {
                    self.gl.active_texture(glow::TEXTURE0 + slot as u32);
                    texture.bind();
                }
            }
            gl_error!(self.gl);

            // now that all textures for this batch are bound, we can draw the points for this batch using the offset/count of batch;
            self.render(batch.vb_offset as u32, batch.vb_count as u32);
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
        self.sp.bind();
    }

    fn update_uniforms(&self, view_projection: Mat4, camera_position: Vec3) {
        unsafe {
            self.gl.uniform_matrix_4_f32_slice(
                Some(&self.u_view_projection),
                false,
                view_projection.as_slice(),
            );
            self.gl
                .uniform_3_f32_slice(Some(&self.u_camera_position), camera_position.as_slice());
        }
        gl_error!(self.gl);
    }
    pub fn update_marker_nodes(&mut self, markers: &Vec<Marker>) -> anyhow::Result<()> {
        use image::GenericImageView;
        let max_texture_slots: usize = self.max_texture_units as usize;

        let mut textures: Vec<Texture> = Vec::new();
        let mut images: BTreeMap<String, usize> = BTreeMap::new();
        let mut nodes = Vec::new();
        for m in markers.iter() {
            let img_path = m
                .icon_file
                .as_ref()
                .ok_or_else(|| {
                    log::error!("marker has no icon file");
                    anyhow::Error::msg("None")
                })
                .unwrap();
            let index = images.entry(img_path.to_string()).or_insert_with(|| {
                let img = image::open(format!("./res/tw/{}", img_path))
                    .map_err(|e| {
                        log::error!("couldn't open image: {}", &e);
                        e
                    })
                    .unwrap();
                let img = img.flipv();
                let pixels = img.as_bytes();
                let t = Texture::new(self.gl.clone());
                t.bind();
                t.update_pixels(pixels, img.width(), img.height());
                let index = textures.len();
                textures.push(t);
                index
            });

            let mut n = MarkerNode::from(m);
            n.tex_slot = (*index % max_texture_slots) as u32;
            nodes.push(n);
        }
        nodes.sort_unstable_by_key(|n| n.tex_slot);
        let mut batches: Vec<Batch> = Vec::new();
        let mut start_node_index = 0;
        let mut current_batch_num = 0;
        for (node_index, n) in nodes.iter().enumerate() {
            if n.tex_slot / 16 > current_batch_num {
                batches.push(Batch {
                    vb_offset: start_node_index,
                    vb_count: node_index - start_node_index,
                    textures: textures.drain(0..max_texture_slots).collect(),
                });
                current_batch_num = n.tex_slot / 16;
                start_node_index = node_index;
            }
        }
        batches.push(Batch {
            vb_offset: start_node_index,
            vb_count: nodes.len() - start_node_index,
            textures: textures.drain(0..).collect(),
        });
        self.batches = batches;
        self.vb
            .update(bytemuck::cast_slice(&nodes), glow::STREAM_DRAW);
        Ok(())
    }

    fn _unbind(&self) {
        self.vao.unbind();
        self.vb.unbind();
        self.sp.unbind();
    }
}

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
