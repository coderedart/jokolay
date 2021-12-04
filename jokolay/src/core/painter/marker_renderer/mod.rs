use std::rc::Rc;

use glow::NativeUniformLocation;

use crate::core::painter::opengl::{shader::ShaderProgram, vertex_array::VertexArrayObject};

pub mod marker;
pub struct MarkerSceneState {}
pub struct MarkerGl {
    pub vao: VertexArrayObject,
    pub sp: ShaderProgram,
    pub u_sampler: NativeUniformLocation,
    pub gl: Rc<glow::Context>,
}
impl MarkerGl {
    // pub fn new(gl: Rc<glow::Context>) -> Self {
    //     let layout = MarkerVertex::get_layout();
    //     let vao = VertexArrayObject::new(gl.clone(), layout);
    //     let sp = ShaderProgram::new(gl.clone(), VERTEX_SHADER_SRC, FRAG_SHADER_SRC, None);
    //     let u_sampler = sp.get_uniform_id("sampler").unwrap();
    //     let marker_gl = Self {
    //         vao,
    //         sp,
    //         u_sampler,
    //         gl: gl.clone(),
    //     };
    //     marker_gl.bind();
    //     marker_gl
    // }
    pub fn draw_markers(&self) {
        // unsafe {
        //     // self.gl.enable(glow::DEPTH_TEST);

        //     self.gl
        //         .blend_func(glow::SRC_ALPHA, glow::ONE_MINUS_SRC_ALPHA);
        // }
        // let mut billboards: Vec<Quad> = Vec::new();
        // let camera_position = glm::Vec3::from(link.f_camera_position);
        // let camera_dvec = camera_position + glm::Vec3::from(link.f_camera_front);
        // let view = glm::look_at_lh(&camera_position, &camera_dvec, &glm::vec3(0.0, 1.0, 0.0));
        // let projection = glm::perspective_fov_lh(link.identity.fov, wc.framebuffer_width as f32, wc.framebuffer_height as f32, self.znear, self.zfar);
        // let vp = projection * view;
    }
    pub fn bind(&self) {
        self.vao.bind();
        self.sp.bind();
    }

    pub fn unbind(&self) {
        self.vao.unbind();
        self.sp.unbind();
    }
}
pub const VERTEX_SHADER_SRC: &str = include_str!("shader.vs");
pub const FRAG_SHADER_SRC: &str = include_str!("shader.fs");
