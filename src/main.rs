extern crate glfw;

use std::net::UdpSocket;

use egui::{epaint::Vertex, Color32, Pos2};
use glfw::{ffi::glfwGetTime, Context};
use glow::*;
use jokolay::{
    glc::renderer::{node::Node, scene::Scene, texture::Tex2D},
    glfw_window_init,
    gw::{load_markers, marker::Marker},
    mlink::get_ml_udp,
    process_events,
};
use nalgebra_glm::{look_at_lh, perspective_fov_lh, vec3};

fn main() -> anyhow::Result<()> {
    let (mut glfw, gl, mut window, events) = glfw_window_init();
    let (_marker_categories, mut markers, _trails) = load_markers();
    let socket = UdpSocket::bind("127.0.0.1:0").expect("failed to bind to socket");
    socket
        .connect("127.0.0.1:7187")
        .expect("failed to connect to socket");
    let link = get_ml_udp("MumbleLink", &socket)?;
    let current_map_id = link.context.unwrap().map_id;

    let current_map_markers: &mut Vec<Marker> = markers.entry(current_map_id).or_default();
    let mut nodes = Vec::new();
    for m in current_map_markers {
        let m = &*m;
        nodes.push(Node::from(m));
    }

    dbg!(nodes.len(), std::mem::size_of::<Node>());
    let mut scene = Scene::new(&gl);
    let nodes_len = nodes.len();
    scene.update_nodes(&nodes);
    let mut prev_frame_time;
    unsafe {
        prev_frame_time = glfwGetTime();
        if gl.get_error() != glow::NO_ERROR {
            println!("{} {} {}", file!(), line!(), column!());
        }
    }

    let _t = Tex2D::new(&gl, None);

    let v: Vec<Vertex> = vec![
        Vertex {
            //top left
            pos: Pos2::new(-1.0, 0.0),
            uv: Pos2::new(-0.5, -0.5),
            color: Color32::BLACK,
        },
        Vertex {
            pos: Pos2::new(0.0, 0.0),
            uv: Pos2::new(0.5, -0.5),
            color: Color32::BLACK,
        },
        Vertex {
            pos: Pos2::new(-0.5, 1.0),
            uv: Pos2::new(0.0, 0.8),
            color: Color32::BLACK,
        },
        Vertex {
            //top right
            pos: Pos2::new(0.0, 0.0),
            uv: Pos2::new(-0.5, -0.5),
            color: Color32::BLACK,
        },
        Vertex {
            pos: Pos2::new(1.0, 0.0),
            uv: Pos2::new(0.5, -0.5),
            color: Color32::BLACK,
        },
        Vertex {
            pos: Pos2::new(0.5, 1.0),
            uv: Pos2::new(0.0, 0.8),
            color: Color32::BLACK,
        },
        Vertex {
            //bottom left
            pos: Pos2::new(-1.0, -1.0),
            uv: Pos2::new(-0.5, -0.5),
            color: Color32::BLACK,
        },
        Vertex {
            pos: Pos2::new(0.0, -1.0),
            uv: Pos2::new(0.5, -0.5),
            color: Color32::BLACK,
        },
        Vertex {
            pos: Pos2::new(-0.5, 0.0),
            uv: Pos2::new(0.0, 0.8),
            color: Color32::BLACK,
        },
    ];
    let i = vec![0, 1, 2, 3, 4, 5, 6, 7, 8];
    let mut fps = 0_u32;
    while !window.should_close() {
        unsafe {
            let cur_time = glfwGetTime();
            fps += 1;
            if cur_time - prev_frame_time > 1.0 {
                dbg!(fps, cur_time);
                fps = 0;
                prev_frame_time = cur_time;
                let e = gl.get_error();
                if e != glow::NO_ERROR {
                    println!("gl_error: {}", e);
                }
            }
        }
        process_events(&mut window, &events, &gl);
        let ml = get_ml_udp("MumbleLink", &socket)?;

        let camera_position = vec3(
            ml.f_camera_position_x,
            ml.f_camera_position_y,
            ml.f_camera_position_z,
        );
        let center = camera_position
            + vec3(
                ml.f_camera_front_x,
                ml.f_camera_front_y,
                ml.f_camera_front_z,
            );
        let id = ml.identity.unwrap();

        let view = look_at_lh(&camera_position, &center, &vec3(0.0, 1.0, 0.0));
        let projection = perspective_fov_lh(id.fov, 800 as f32, 600 as f32, 0.1, 30000.0);
        let vp = projection * view;

        // std::thread::sleep(std::time::Duration::from_secs(2));
        scene.clear_screen();

        scene.render_nodes(&vp, &camera_position, nodes_len as i32);
        scene.update_egui_buffers(&v, &i);
        scene.render_egui(i.len() as i32);

        window.swap_buffers();
  
        glfw.poll_events();
    }
    Ok(())
}
