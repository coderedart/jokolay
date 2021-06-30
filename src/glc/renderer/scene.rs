use std::rc::Rc;

use super::{buffer::Buffer,  material::Material, vertex_array::VertexArrayObject};

use nalgebra_glm::{Mat4, Vec2, Vec3};

pub struct SceneNode {
    pub vao: VertexArrayObject,
    pub vb: Buffer,
    pub ib: Buffer,
    pub material: Material,
    pub gl: Rc<glow::Context>,

}

pub trait Renderable {
    fn bind(&self);
    fn update_buffers(&self, vb: Option<(&[u8], u32)>, ib: Option<(&[u8], u32)>);
    fn update_uniforms(&self, uniform_data: SceneNodeUniform);
    fn render(&self, count: u32, offset: u32);
    fn unbind(&self);


}
pub enum SceneNodeUniform<'b> {
    MarkerSceneNodeUniform {
        vp: Mat4,
        cam_pos: Vec3,
        player_pos: Vec3,
        samplers: &'b [u32],
    },
    EguiSceneNodeUniform {
        screen_size: Vec2,
        u_sampler: u32,
    }
}

// fn setup_vao_node<'a>(gl: &'a glow::Context) -> VertexArrayObject<'a> {
//     let vb = Buffer::new(gl);
//     let vblayout = MarkerNode::get_buffer_layout();
//     let mut ib_id = None;
//     unsafe {
//         ib_id = Some(gl.create_buffer().unwrap());
//     }
//     let program_node = ShaderProgram::new(
//         &gl,
//         Path::new("./res/node.vs"),
//         Some(Path::new("./res/node.gs")),
//         Path::new("./res/node.fs"),
//     );
//     let vao = VertexArrayObject::new(gl, vb, vblayout, ib_id, program_node);
//     vao
// }

// fn setup_vao_egui<'a>(gl: &'a glow::Context) -> VertexArrayObject<'a> {
//     let vb = Buffer::new(gl);
//     let vblayout = eglfw::get_egui_vertex_buffer_layout();
//     let mut ib_id = None;
//     unsafe {
//         ib_id = Some(gl.create_buffer().unwrap());
//     }
//     let program_egui = ShaderProgram::new(
//         &gl,
//         Path::new("./res/egui.vs"),
//         None,
//         Path::new("./res/egui.fs"),
//     );
//     let vao = VertexArrayObject::new(gl, vb, vblayout, ib_id, program_egui);
//     vao
// }
