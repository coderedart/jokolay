extern crate glfw;

use std::{fs::File, io::Read, mem, path::Path, sync::mpsc::Receiver};

use cgmath::SquareMatrix;
use glfw::{ffi::glfwGetTime, Action, Context, Key};
use glow::*;
use jokolay::{glc::renderer::shader::ShaderProgram, gw::mlink::get_ml};


const SCR_HEIGHT: u32 = 1080;
const SCR_WIDTH: u32 = 960;
fn main() {
    let vspath: &Path = Path::new("res/shader.vs");
    let fspath: &Path = Path::new("res/shader.fs");

    let mut glfw = glfw::init(glfw::FAIL_ON_ERRORS).unwrap();
    glfw.window_hint(glfw::WindowHint::ContextVersion(4, 6));
    glfw.window_hint(glfw::WindowHint::OpenGlProfile(
        glfw::OpenGlProfileHint::Core,
    ));
    glfw.window_hint(glfw::WindowHint::TransparentFramebuffer(true));
    glfw.window_hint(glfw::WindowHint::Floating(true));
    //glfw.window_hint(glfw::WindowHint::MousePassthrough(true));
    //glfw.window_hint(glfw::WindowHint::DoubleBuffer(false));
    let (mut window, events) = glfw
        .create_window(
            SCR_WIDTH,
            SCR_HEIGHT,
            "LearnOpenGL",
            glfw::WindowMode::Windowed,
        )
        .expect("Failed to create GLFW window");

    window.set_key_polling(true);
    window.make_current();
    window.set_framebuffer_size_polling(true);
    let gl =
        unsafe { glow::Context::from_loader_function(|s| window.get_proc_address(s) as *const _) };
    let shader_program = ShaderProgram::new(&gl, vspath, fspath);

    let world_matrix = cgmath::Matrix4::<f32>::from_translation(cgmath::vec3(0.7, 0.7, 0.2));

    let vao = setup_buffers(&gl);
    let uni;
    let mut start;
    unsafe {
        uni = gl
            .get_uniform_location(shader_program.id, "transform")
            .unwrap();
        start = glfwGetTime();
    }
    let sec = 1.0;
    let mut fps = 0;

    while !window.should_close() {
        process_events(&mut window, &events, &gl);
        fps += 1;

        unsafe {
            gl.clear_color(0.0, 0.0, 0.0, 0.0);
            gl.clear(glow::COLOR_BUFFER_BIT);
            gl.use_program(Some(shader_program.id));
            gl.bind_vertex_array(Some(vao));
            let tf: &[f32; 16] = world_matrix.as_ref();
            gl.uniform_matrix_4_f32_slice(Some(&uni), false, tf);
            gl.draw_arrays(glow::TRIANGLES, 0, 3);
            let link = get_ml("MumbleLink").unwrap();
            if glfwGetTime() - start > sec {
                dbg!(fps, start, link.ui_tick);
                fps = 0;
                start = glfwGetTime();
            }
        }
        glfw.poll_events();
        window.swap_buffers();
    }
}
fn process_events(
    window: &mut glfw::Window,
    events: &Receiver<(f64, glfw::WindowEvent)>,
    gl: &glow::Context,
) {
    for (_, event) in glfw::flush_messages(events) {
        match event {
            glfw::WindowEvent::FramebufferSize(width, height) => {
                // make sure the viewport matches the new window dimensions; note that width and
                // height will be significantly larger than specified on retina displays.
                unsafe {
                    gl.viewport(0, 0, width, height);
                }
                eprintln!("resizing viewport");
            }
            glfw::WindowEvent::Key(Key::Escape, _, Action::Press, _) => {
                window.set_should_close(true)
            }
            _ => {}
        }
    }
}
fn setup_buffers(gl: &glow::Context) -> u32 {
    unsafe {
        let vertices: Vec<f32> = vec![
            -0.3, -0.3, 0.0, // left
            0.3, -0.3, 0.0, // right
            0.0, 0.3, 0.0, // top
            -0.3, 0.3, 0.0, //leftop
            0.0, -0.3, 0.0, //bottom
            0.3, 0.3, 0.0, //rightop
        ];
        let vao = gl.create_vertex_array().unwrap();
        let vbo = gl.create_buffer().unwrap();
        gl.bind_vertex_array(Some(vao));
        gl.bind_buffer(glow::ARRAY_BUFFER, Some(vbo));
        gl.buffer_data_u8_slice(
            glow::ARRAY_BUFFER,
            bytemuck::cast_slice(&vertices),
            glow::STATIC_DRAW,
        );
        gl.vertex_attrib_pointer_f32(
            0,
            3,
            glow::FLOAT,
            false,
            3 * mem::size_of::<f32>() as i32,
            0,
        );
        gl.enable_vertex_attrib_array(0);
        gl.bind_vertex_array(Some(vao));
        vao
    }
}
