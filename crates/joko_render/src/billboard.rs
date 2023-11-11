use std::sync::Arc;

use egui::ahash::HashMap;
use egui_render_three_d::{
    three_d::{context::*, Context, HasContext},
    GpuTexture,
};
use glam::{Vec2, Vec3};
use tracing::{error, info, warn};

use crate::gl_error;

const MARKER_VERTEX_STRIDE: i32 = std::mem::size_of::<MarkerVertex>() as _;
pub struct BillBoardRenderer {
    pub markers: Vec<MarkerObject>,
    pub trails: Vec<TrailObject>,
    marker_program: NativeProgram,
    vao: NativeVertexArray,
    vb: NativeBuffer,
    trail_buffers: Vec<NativeBuffer>,
}
pub struct TrailObject {
    pub vertices: Arc<[MarkerVertex]>,
    pub texture: u64,
}
const MARKER_VS: &str = include_str!("../shaders/marker.vs");

const MARKER_FS: &str = include_str!("../shaders/marker.fs");
impl BillBoardRenderer {
    pub fn new(gl: &Context) -> Self {
        unsafe {
            let marker_program = new_program(gl, MARKER_VS, MARKER_FS, None);
            let vb = create_marker_buffer(gl);
            let vao = gl.create_vertex_array().expect("failed to create egui vao");
            gl.bind_vertex_array(Some(vao));
            gl.bind_vertex_buffer(0, Some(vb), 0, MARKER_VERTEX_STRIDE);
            gl_error!(gl);

            gl.enable_vertex_array_attrib(vao, 0);
            gl.vertex_array_attrib_format_f32(vao, 0, 3, FLOAT, false, 0);
            gl.vertex_array_attrib_binding_f32(vao, 0, 0);
            gl_error!(gl);

            gl.enable_vertex_array_attrib(vao, 1);
            gl.vertex_array_attrib_format_f32(vao, 1, 1, FLOAT, false, 12);
            gl.vertex_array_attrib_binding_f32(vao, 1, 0);
            gl_error!(gl);

            gl.enable_vertex_array_attrib(vao, 2);
            gl.vertex_array_attrib_format_f32(vao, 2, 2, FLOAT, false, 16);
            gl.vertex_array_attrib_binding_f32(vao, 2, 0);
            gl_error!(gl);

            gl.enable_vertex_array_attrib(vao, 3);
            gl.vertex_array_attrib_format_f32(vao, 3, 2, FLOAT, false, 24);
            gl.vertex_array_attrib_binding_f32(vao, 3, 0);
            gl_error!(gl);

            gl.enable_vertex_array_attrib(vao, 4);
            gl.vertex_array_attrib_format_f32(vao, 4, 4, UNSIGNED_BYTE, true, 32);
            gl.vertex_array_attrib_binding_f32(vao, 4, 0);
            gl_error!(gl);

            Self {
                markers: Vec::new(),
                marker_program,
                vb,
                trails: Vec::new(),
                trail_buffers: Default::default(),
                vao,
            }
        }
    }
    pub fn prepare_frame(&mut self) {
        self.markers.clear();
        self.trails.clear();
    }
    pub fn prepare_render_data(&mut self, _link: &jokolink::MumbleLink, gl: &Context) {
        unsafe {
            gl_error!(gl);
        }
        // sort by depth
        self.markers.sort_unstable_by(|first, second| {
            first.distance.total_cmp(&second.distance).reverse() // we need the farther markers (more distance from camera) to be rendered first, for correct alpha blending
        });

        let mut required_size_in_bytes =
            (self.markers.len() * 6 * std::mem::size_of::<MarkerVertex>()) as u64;
        for trail in self.trails.iter() {
            let len = (trail.vertices.len() * std::mem::size_of::<MarkerVertex>()) as u64;
            required_size_in_bytes = required_size_in_bytes.max(len);
        }
        let mut vb = vec![];
        vb.reserve(self.markers.len() * 6 * std::mem::size_of::<MarkerVertex>());

        for marker_object in self.markers.iter() {
            vb.extend_from_slice(&marker_object.vertices);
        }
        unsafe {
            gl_error!(gl);
            gl.bind_buffer(ARRAY_BUFFER, Some(self.vb));
            gl.buffer_data_u8_slice(ARRAY_BUFFER, bytemuck::cast_slice(&vb), DYNAMIC_DRAW);
            gl_error!(gl);
        }
        if self.trails.len() > self.trail_buffers.len() {
            let needs = self.trails.len() - self.trail_buffers.len();
            for _ in 0..needs {
                self.trail_buffers.push(unsafe { create_marker_buffer(gl) });
            }
        }
        for (trail, trail_buffer) in self.trails.iter().zip(self.trail_buffers.iter()) {
            unsafe {
                gl.bind_buffer(ARRAY_BUFFER, Some(*trail_buffer));
                gl.buffer_data_u8_slice(
                    ARRAY_BUFFER,
                    bytemuck::cast_slice(trail.vertices.as_ref()),
                    DYNAMIC_DRAW,
                );
            }
        }
        unsafe {
            gl_error!(gl);
        }
    }
    pub fn render(
        &self,
        gl: &Context,
        cam_pos: glam::Vec3,
        view_proj: &glam::Mat4,
        textures: &HashMap<u64, GpuTexture>,
    ) {
        unsafe {
            gl_error!(gl);
            gl.disable(SCISSOR_TEST);

            gl.use_program(Some(self.marker_program));
            gl.bind_vertex_array(Some(self.vao));
            gl.active_texture(TEXTURE0);

            gl.uniform_3_f32_slice(Some(&NativeUniformLocation(0)), cam_pos.as_ref());
            gl.uniform_matrix_4_f32_slice(
                Some(&NativeUniformLocation(2)),
                false,
                view_proj.to_cols_array().as_ref(),
            );
            for (trail, trail_buffer) in self.trails.iter().zip(self.trail_buffers.iter()) {
                if let Some(texture) = textures.get(&trail.texture) {
                    gl.bind_vertex_buffer(0, Some(*trail_buffer), 0, MARKER_VERTEX_STRIDE);
                    gl.bind_buffer(ARRAY_BUFFER, Some(*trail_buffer));
                    gl.bind_texture(TEXTURE_2D, Some(texture.handle));
                    gl.bind_sampler(0, Some(texture.sampler));
                    gl.draw_arrays(TRIANGLES, 0, trail.vertices.len() as _);
                }
            }
            gl.bind_vertex_buffer(0, Some(self.vb), 0, MARKER_VERTEX_STRIDE);

            gl.bind_buffer(ARRAY_BUFFER, Some(self.vb));
            for (index, mo) in self.markers.iter().enumerate() {
                let index: u32 = index.try_into().unwrap();
                if let Some(texture) = textures.get(&mo.texture) {
                    gl.bind_texture(TEXTURE_2D, Some(texture.handle));
                    gl.bind_sampler(0, Some(texture.sampler));
                    gl.draw_arrays(TRIANGLES, index as i32 * 6, 6);
                }
            }
            gl_error!(gl);
            gl.bind_vertex_array(None);
        }
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct MarkerVertex {
    pub position: Vec3,
    pub alpha: f32,
    pub texture_coordinates: Vec2,
    pub fade_near_far: Vec2,
    pub color: [u8; 4],
}

pub struct MarkerObject {
    /// The six vertices that make up the marker quad
    pub vertices: [MarkerVertex; 6],
    /// The (managed) texture id from egui data
    pub texture: u64,
    /// The distance from camera
    /// As markers have transparency, we need to render them from far -> near order
    /// So, we will sort them using this distance just before rendering
    pub distance: f32,
}

/// takes in strings containing vertex/fragment shaders and returns a Shaderprogram with them attached
#[tracing::instrument(skip(gl))]
pub fn new_program(
    gl: &Context,
    vertex_shader_source: &str,
    fragment_shader_source: &str,
    _geometry_shader_source: Option<&str>,
) -> NativeProgram {
    unsafe {
        gl_error!(gl);

        let program = gl.create_program().unwrap();
        let vertex_shader = gl.create_shader(VERTEX_SHADER).unwrap();
        gl.shader_source(vertex_shader, vertex_shader_source);
        gl.compile_shader(vertex_shader);
        if !gl.get_shader_compile_status(vertex_shader) {
            let e = gl.get_shader_info_log(vertex_shader);
            error!("{}", &e);
            panic!("vertex shader compilation error: {}", &e);
        }
        let frag_shader = gl.create_shader(FRAGMENT_SHADER).unwrap();
        gl.shader_source(frag_shader, fragment_shader_source);
        gl.compile_shader(frag_shader);
        if !gl.get_shader_compile_status(frag_shader) {
            let e = gl.get_shader_info_log(frag_shader);
            error!("frag shader compilation error:{}", &e);
            panic!("frag shader compilation error: {}", &e);
        }
        gl.attach_shader(program, vertex_shader);
        gl.attach_shader(program, frag_shader);
        // let geometry_shader;
        // geometry_shader = gl.create_shader(glow::GEOMETRY_SHADER).unwrap();
        // if let Some(gss) = geometry_shader_source {
        //     gl.shader_source(geometry_shader, gss);
        //     gl.compile_shader(geometry_shader);
        //     if !gl.get_shader_compile_status(geometry_shader) {
        //         let e = gl.get_shader_info_log(geometry_shader);
        //         error!("frag shader compilation error:{}", &e);
        //         panic!("geometry shader compilation error: {}", &e);
        //     }
        //     gl.attach_shader(shader_program, geometry_shader);
        // }
        gl.link_program(program);
        if !gl.get_program_link_status(program) {
            let e = gl.get_program_info_log(program);
            error!("shader program link error: {}", &e);
            panic!("shader program link error: {}", &e);
        }
        gl.delete_shader(vertex_shader);
        // if geometry_shader_source.is_some() {
        //     gl.delete_shader(geometry_shader);
        // }
        gl.delete_shader(frag_shader);
        gl_error!(gl);
        let active_attribute_count = gl.get_active_attributes(program);
        let mut shader_info = format!("Shader Info:\nvertex attributes: {active_attribute_count}");
        for index in 0..active_attribute_count {
            if let Some(attr) = gl.get_active_attribute(program, index) {
                let location = gl.get_attrib_location(program, &attr.name);
                shader_info = format!("{shader_info}\n{} @ {}", attr.name, location.unwrap());
            } else {
                warn!("attribute with index {index} doesn't exist");
            }
        }
        let active_uniform_count = gl.get_active_uniforms(program);
        shader_info = format!("{shader_info}\nuniform locations:{active_uniform_count}");
        for index in 0..active_uniform_count {
            if let Some(attr) = gl.get_active_uniform(program, index) {
                let location = gl.get_uniform_location(program, &attr.name);
                shader_info = format!("{shader_info}\n{} @ {}", attr.name, location.unwrap().0);
            } else {
                warn!("uniform with index {index} doesn't exist");
            }
        }
        info!("{shader_info}");
        program
    }
}
unsafe fn create_marker_buffer(gl: &Context) -> NativeBuffer {
    gl_error!(gl);
    let vb = gl.create_buffer().expect("failed to create vb for markers");
    gl_error!(gl);

    gl.bind_vertex_array(None);
    gl.bind_buffer(ARRAY_BUFFER, Some(vb));
    gl_error!(gl);

    gl.bind_buffer(ARRAY_BUFFER, None);
    gl_error!(gl);
    vb
}
