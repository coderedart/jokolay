extern crate glfw;

use std::{convert::TryInto, path::Path, usize, vec};

use glfw::Context;
use glow::*;
use image::GenericImageView;
use jokolay::{
    glc::renderer::{
        node::Node,
        scene::{self, Scene},
    },
    gw::{
        load_markers,
        marker::Marker,
        mlink::{get_ml, get_win_pos_dim},
    },
    process_events,
};
use nalgebra_glm::{
    look_at_lh, look_at_rh, make_vec3, ortho_lh, perspective_fov_lh, perspective_fov_rh, Mat4, Vec3,
};

fn main() {
    let scr_height: u32;
    let scr_width: u32;
    let mut glfw = glfw::init(glfw::FAIL_ON_ERRORS).unwrap();
    glfw.window_hint(glfw::WindowHint::ContextVersion(4, 6));
    glfw.window_hint(glfw::WindowHint::OpenGlProfile(
        glfw::OpenGlProfileHint::Core,
    ));
    glfw.window_hint(glfw::WindowHint::TransparentFramebuffer(true));
    glfw.window_hint(glfw::WindowHint::Floating(true));
    //glfw.window_hint(glfw::WindowHint::MousePassthrough(true));
    //glfw.window_hint(glfw::WindowHint::DoubleBuffer(false));
    let win_pos_dim = get_win_pos_dim("MumbleLink");
    match win_pos_dim {
        Some(w) => {
            scr_height = w.height;
            scr_width = w.width;
        }
        None => todo!(),
    }
    let (mut window, events) = glfw
        .create_window(
            scr_width,
            scr_height,
            "LearnOpenGL",
            glfw::WindowMode::Windowed,
        )
        .expect("Failed to create GLFW window");

    window.set_key_polling(true);
    window.make_current();
    window.set_framebuffer_size_polling(true);
    let gl =
        unsafe { glow::Context::from_loader_function(|s| window.get_proc_address(s) as *const _) };
    let (_marker_categories, mut markers, _trails) = load_markers();
    let link = get_ml("MumbleLink");
    let mut current_map_id: u32 = 15;
    if let Some(ml) = link {
        current_map_id = ml.get_identity().map_id;
    }

    let current_map_markers: &mut Vec<Marker> = markers.entry(current_map_id).or_default();
    let mut nodes = Vec::new();
    for m in current_map_markers {
        // for m in marker_vec {
        nodes.push(Node {
            xpos: m.xpos,
            ypos: m.ypos,
            zpos: m.zpos,
        });
        // }
    }
    // nodes.push(Node {xpos: 20.0, ypos: 16.0,zpos: 0.0});
    // nodes.push(Node {xpos: 0.0, ypos: 16.0,zpos: 0.0});
    // nodes.push(Node {xpos: -20.0, ypos: 16.0,zpos: 0.0});

    dbg!(nodes.len());
    let mut scene = Scene::new(&gl, nodes);

    unsafe {
        gl.enable(glow::DEPTH_TEST);
        let texture = gl.create_texture().unwrap();
        gl.bind_texture(glow::TEXTURE_2D, Some(texture)); // all upcoming GL_TEXTURE_2D operations now have effect on this texture object
                                                          // set the texture wrapping parameters
        gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_WRAP_S, glow::REPEAT as i32); // set texture wrapping to gl::REPEAT (default wrapping method)
        gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_WRAP_T, glow::REPEAT as i32);
        // set texture filtering parameters
        gl.tex_parameter_i32(
            glow::TEXTURE_2D,
            glow::TEXTURE_MIN_FILTER,
            glow::LINEAR as i32,
        );
        gl.tex_parameter_i32(
            glow::TEXTURE_2D,
            glow::TEXTURE_MAG_FILTER,
            glow::LINEAR as i32,
        );
        // load image, create texture and generate mipmaps
        let img = image::open(&Path::new("./res/tex.png")).expect("Failed to load texture");
        let img = img.flipv();
        let data = img.as_bytes();
        gl.tex_image_2d(
            glow::TEXTURE_2D,
            0,
            glow::RGBA as i32,
            img.width() as i32,
            img.height() as i32,
            0,
            glow::RGBA,
            glow::UNSIGNED_BYTE,
            Some(&data),
        );
        gl.generate_mipmap(glow::TEXTURE_2D);
    }

    while !window.should_close() {
        process_events(&mut window, &events, &gl);
        let link = get_ml("MumbleLink");
        let model = Mat4::new_translation(&make_vec3(&[0.0, 100.0, 0.0]));
        if let Some(ml) = link {
            let center = make_vec3(&ml.f_camera_position) + make_vec3(&ml.f_camera_front);
            let view = look_at_lh(
                &make_vec3(&ml.f_camera_position),
                &center,
                &make_vec3(&[0.0, 1.0, 0.0]),
            );
            let id = ml.get_identity();
            dbg!(id.fov, &ml.f_avatar_position);
            let projection =
                perspective_fov_lh(id.fov, scr_width as f32, scr_height as f32, 0.1, 30000.0);
            //let projection  = ortho_lh(21000.0, 21000.0, 0.0, 200.0, 1.0, 10000.0);
            scene.view_projection = projection * view;
            scene.cam_pos = make_vec3(&ml.f_camera_position); // make_vec3(&ml.f_camera_position);
        }
        // std::thread::sleep(std::time::Duration::from_secs(2));
        scene.render();
        window.swap_buffers();
        glfw.poll_events();
    }
}
