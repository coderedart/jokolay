use std::{collections::BTreeMap, rc::Rc};

use glow::{Context, HasContext, NativeUniformLocation};

use glm::{Mat4, Vec3};

use crate::{
    gl_error,
    gltypes::{
        buffer::{Buffer, VertexBufferLayout},
        shader::ShaderProgram,
        texture::{Texture, TextureManager},
        vertex_array::VertexArrayObject,
    },
};

use super::xmltypes::xml_marker::Marker;

// use super::xmltypes::xml_marker::Marker;

#[derive(Debug, Clone, Copy)]
pub struct MarkerNode {
    pub position: [f32; 3],
    pub tex_coords: [f32; 2],
    pub tex_layer: u32,
}

impl Default for MarkerNode {
    fn default() -> Self {
        MarkerNode {
            position: [0.0, 0.0, 0.0],
            tex_coords: [0.0, 0.0],
            tex_layer: 0,
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
        vbl.push_f32(2, false); // tex_coords
        vbl.push_u32(1, false); // tex_layer
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
    pub tm: TextureManager,
    pub batches: Vec<Batch>,
    pub u_camera_position: NativeUniformLocation,
    pub u_view_projection: NativeUniformLocation,
    pub u_samplers: NativeUniformLocation,
    pub gl: Rc<glow::Context>,
}
#[derive(Debug, Clone, Copy)]
pub struct Batch {
    pub vb_offset: usize,
    pub vb_count: usize,
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
        let u_camera_position: NativeUniformLocation;
        let u_view_projection: NativeUniformLocation;
        let u_samplers: NativeUniformLocation;
        unsafe {
            u_camera_position = gl.get_uniform_location(sp.id, "camera_position").unwrap();
            u_view_projection = gl.get_uniform_location(sp.id, "view_projection").unwrap();
            u_samplers = gl.get_uniform_location(sp.id, "u_samplers").unwrap();
        }
        let tm = TextureManager::new(gl.clone());
        let scene = MarkerScene {
            vao,
            vb,
            sp,
            tm,
            batches: Vec::new(),
            u_camera_position,
            u_view_projection,
            u_samplers,
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
            self.gl.disable(glow::SCISSOR_TEST);
            self.gl
                .blend_func(glow::SRC_ALPHA, glow::ONE_MINUS_SRC_ALPHA);
            self.gl.active_texture(glow::TEXTURE0);
        }
        self.bind();
        unsafe {
            self.gl.active_texture(glow::TEXTURE0);
        }
        self.update_uniforms(view_projection, camera_position, 0);

        for (slot, batch) in self.batches.iter().enumerate() {
            // batches are organized based on texture index in material. so, first batch is 0-15 textures, second is 16-31 and so on.
            // we get the offset from where we start binding the 16 textures to the slots
            // we stick to 16 textures for now. but eventually shift to MAX_TEXTURE_IMAGE_UNITS to jump to 32 when possible
            //bind textures to their respective slots
            if batch.vb_count == 0 {
                continue;
            }
            gl_error!(self.gl);

            self.tm.array_tex[slot].bind();
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

    fn update_uniforms(&self, view_projection: Mat4, camera_position: Vec3, sampler: i32) {
        unsafe {
            self.gl.uniform_matrix_4_f32_slice(
                Some(&self.u_view_projection),
                false,
                view_projection.as_slice(),
            );
            self.gl
                .uniform_3_f32_slice(Some(&self.u_camera_position), camera_position.as_slice());
            self.gl.uniform_1_i32(Some(&self.u_samplers), sampler);
        }
        gl_error!(self.gl);
    }
    pub fn update_marker_nodes(&mut self, markers: &Vec<Marker>) -> anyhow::Result<()> {
        use image::GenericImageView;

        let mut batched_nodes = vec![vec![]; TextureManager::NUM_OF_ARRAYS];
        for m in markers.iter() {
            let img_path = m.icon_file.clone().unwrap_or("tex.png".to_string());
            let (slot, t_x, t_y, layer) = self.tm.get_image(&img_path);
            log::trace!("texture for current node {} slot {} layer {}", &img_path, slot, layer);

            let mut n = MarkerNode::from(m);
            n.tex_coords = [t_x, t_y];
            n.tex_layer = layer;
            batched_nodes[slot as usize].push(n);
        }
        let mut batches = vec![
            Batch {
                vb_count: 0,
                vb_offset: 0,
            };
            TextureManager::NUM_OF_ARRAYS
        ];
        let mut offset: u32 = 0;
        for i in 0..TextureManager::NUM_OF_ARRAYS {
            batches[i].vb_offset = offset as usize;
            batches[i].vb_count = batched_nodes[i].len();
            offset += batched_nodes[i].len() as u32;
        }
        self.batches = batches;
        let nodes: Vec<MarkerNode> = batched_nodes.into_iter().flatten().collect();
        self.vb
            .update(bytemuck::cast_slice(&nodes), glow::STREAM_DRAW);
        log::trace!("{:#?}",&self.tm.live_images);
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
layout (location = 1) in vec2 tex_coords;
layout (location = 2) in uint tex_layer;

out vec2 v_tex_coords;
flat out uint v_tex_layer;

void main()
{
    gl_Position = vec4(Position , 1.0);
    v_tex_layer = tex_layer;
    v_tex_coords = tex_coords;
}
"#;

const MARKER_FRAGMENT_SHADER_SRC: &str = r#"
#version 330                                                                        
                                                                                    
uniform sampler2DArray u_samplers;                                                        
                                                                       
in vec2 tex_coords; 
flat in uint g_tex_layer;                                                                  
out vec4 frag_color;                                                                 
                                                                                    
void main()                                                                         
{                                                                                   
    frag_color = texture(u_samplers, vec3(tex_coords, g_tex_layer));                                     
        // frag_color = vec4(frag_color.xyz, 1.0);                                                                   
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


in vec2 v_tex_coords[1];
flat in uint v_tex_layer[1];

                                                                       
out vec2 tex_coords; 
flat out uint g_tex_layer;                                                                 
                                                                                    
void main()                                                                         
{              
    g_tex_layer = v_tex_layer[0];

    float billboard_height = 1.0;
    float billboard_width = 1.0;

    vec3 Pos = gl_in[0].gl_Position.xyz;                                            
    vec3 toCamera = normalize(camera_position - Pos);                                    
    vec3 up = vec3(0.0, 1.0, 0.0);                                                  
    vec3 right = cross(toCamera, up);                                               
    //left bottom vertex                                                                                
    Pos -= (right * billboard_width / 2.0);                                                           
    gl_Position = view_projection * vec4(Pos, 1.0);                                             
    tex_coords = vec2(0.0, 0.0);                                                      
    EmitVertex();                                                                   
    // lefttop                                                                   
    Pos.y += billboard_height;                                                                   
    gl_Position = view_projection * vec4(Pos, 1.0);                                             
    tex_coords = vec2(0.0, v_tex_coords[0].y);                                                      
    EmitVertex();                                                                   
    // right bottom                                                                      
    Pos.y -= billboard_height;                                                                   
    Pos += (right * billboard_width);                                                                   
    gl_Position = view_projection * vec4(Pos, 1.0);                                             
    tex_coords = vec2(v_tex_coords[0].x, 0.0);                                                      
    EmitVertex();                                                                   
    // right top                                                                                
    Pos.y += billboard_height;                                                                   
    gl_Position = view_projection * vec4(Pos, 1.0);                                             
    tex_coords = v_tex_coords[0];                                                      
    EmitVertex();                                                                   
                                                                                    
    EndPrimitive();                                                                 
}             
  

"#;
