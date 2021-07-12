use std::{collections::BTreeMap, rc::Rc};

use glow::{Context, HasContext};
use image::DynamicImage;
use nalgebra_glm::{Mat4, Vec3};

use crate::gltypes::{
    buffer::{Buffer, VertexBufferLayout},
    shader::ShaderProgram,
    texture::TextureArray,
    vertex_array::VertexArrayObject,
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
        vbl.push_u32(1); // tex_layer
        vbl.push_u32(1); // tex_slot
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
}
pub struct Batch {
    pub vb_offset: u32,
    pub vb_count: u32,
    pub textures: Vec<TextureArray>,
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
        unsafe {
            u_camera_position = gl.get_uniform_location(sp.id, "camera_position").unwrap();
            u_view_projection = gl.get_uniform_location(sp.id, "view_projection").unwrap();
        }

        let scene = MarkerScene {
            vao,
            vb,
            sp,
            batches: Vec::new(),
            u_camera_position,
            u_view_projection,
            gl: gl.clone(),
        };
        scene.bind();
        let vblayout = MarkerNode::get_layout();
        vblayout.set_layout(gl);
        return scene;
    }
    pub fn draw(&self, view_projection: Mat4, camera_position: Vec3) {
        unsafe {
            // self.gl.disable(glow::FRAMEBUFFER_SRGB);
            self.gl.disable(glow::SCISSOR_TEST);
            self.gl.enable(glow::DEPTH_TEST);
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
            // now that all textures for this batch are bound, we can draw the points for this batch using the offset/count of batch;
            self.render(batch.vb_offset, batch.vb_count);
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
    }
    pub fn update_marker_nodes(&mut self, markers: &Vec<Marker>) -> anyhow::Result<()> {
        use image::GenericImageView;
        let images: BTreeMap<String, DynamicImage> = markers
            .iter()
            .filter_map(|m| {
                let i = m
                    .icon_file
                    .as_ref()
                    .ok_or_else(|| {
                        log::error!("marker has no icon file");
                        anyhow::Error::msg("None")
                    })
                    .ok()?;
                let img = image::open(i)
                    .map_err(|e| {
                        log::error!("couldn't open image: {}", &e);
                        e
                    })
                    .ok()?;
                Some((i.clone(), img))
            })
            .collect();
        let mut sizes_to_slot_map: BTreeMap<(u32, u32), (u32, u32)> = BTreeMap::new();
        let mut textures = Vec::new();
        let mut nodes = Vec::new();
        let mut pixels: Vec<Vec<&[u8]>> = Vec::new();
        for m in markers {
            if m.icon_file.is_none() {
                let n = MarkerNode::from(m);
                nodes.push(n);
                continue;
            }
            let name = m.icon_file.as_ref().unwrap();
            let i = images.get(name).unwrap();
            let (slot, layer) = sizes_to_slot_map.entry(i.dimensions()).or_insert_with(|| {
                let t = TextureArray::new(self.gl.clone(), i.width(), i.height());
                textures.push(t);
                pixels.push(Vec::new());
                ((textures.len() - 1) as u32, 0)
            });
            pixels[*slot as usize].push(i.as_bytes());

            let mut n = MarkerNode::from(m);
            n.tex_slot = *slot;
            n.tex_layer = *layer;
            *layer += 1;
            nodes.push(n);
        }
        for (t, p) in textures.iter_mut().zip(pixels.iter()) {
            t.bind();
            t.update_pixels(&p);
        }
        nodes.sort_unstable_by_key(|n| n.tex_slot);
        self.batches.clear();
        let num_of_batches = textures.len() / 16;
        let mut current_batch = 0;
        let mut previous_offset = 0;
        while current_batch < num_of_batches {
            let start_of_batch = previous_offset;
            let end_of_batch = nodes[previous_offset..]
                .iter()
                .position(|n| n.tex_slot % 16 > current_batch as u32);
            match end_of_batch {
                Some(count) => {
                    previous_offset += count;
                    self.batches.push(Batch {
                        vb_offset: start_of_batch as u32,
                        vb_count: count as u32,
                        textures: textures.drain(0..16).collect(),
                    });
                }
                None => {}
            }
            current_batch += 1;
        }
        self.batches.push(Batch {
            vb_offset: previous_offset as u32,
            vb_count: (previous_offset + nodes[previous_offset..].len()) as u32,
            textures,
        });
        self.vb.update(bytemuck::cast_slice(&nodes), glow::STREAM_DRAW);
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
