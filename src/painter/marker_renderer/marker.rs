use glm::{Mat4, Vec3, Vec4, cross, make_vec3, make_vec4, normalize};
use jokolink::mlink::MumbleLink;

use crate::{painter::opengl::buffer::{VertexBufferLayout, VertexBufferLayoutTrait}, tactical::localtypes::{category::IMCategory, marker::POI}, window::glfw_window::GlfwWindow};

/// Marker Vertex contains the vertex position in clip space, tex coords.
/// we send vertices in vec4 (clip space) so that opengl can do perspective division, remove the triangles outside clipped space and finally do perspective interpolation for textures
#[derive(Debug, Clone, Copy, Default)]
pub struct MarkerVertex {
    pub vpos: [f32; 4],
    pub tex_coords: [f32; 3],
    pub alpha: f32,
}

impl VertexBufferLayoutTrait for MarkerVertex {
    fn get_layout() -> crate::painter::opengl::buffer::VertexBufferLayout {
        let mut layout = VertexBufferLayout::default();
        layout.push_f32(4, false);
        layout.push_f32(3, false);
        layout.push_f32(1, false);
        layout
    }
}

unsafe impl bytemuck::Zeroable for MarkerVertex {
    fn zeroed() -> Self {
        unsafe { core::mem::zeroed() }
    }
}
unsafe impl bytemuck::Pod for MarkerVertex {}


#[derive(Debug, Clone, Copy, Default)]
pub struct Triangle {
    pub vertices: [MarkerVertex; 3],
}



unsafe impl bytemuck::Zeroable for Triangle {
    fn zeroed() -> Self {
        unsafe { core::mem::zeroed() }
    }
}
unsafe impl bytemuck::Pod for Triangle {}


#[derive(Debug, Clone, Copy, Default)]
pub struct Quad {
    pub triangles: [Triangle; 2],
}



unsafe impl bytemuck::Zeroable for Quad {
    fn zeroed() -> Self {
        unsafe { core::mem::zeroed() }
    }
}
unsafe impl bytemuck::Pod for Quad {}

// impl Quad {
//     pub fn new(marker: &POI,cat: &IMCategory, camera_position: Vec3, player_position: Vec3, vp: Mat4, tex_coords: [f32; 3]) -> Option<Quad>  {
//         let x = tex_coords[0];
//         let y = tex_coords[1];
//         let z = tex_coords[2];
//         let pos: Vec3 = marker.pos.into();

//         let cdist = glm::distance(&pos, &camera_position);
//         let pdist = glm::distance(&pos, &player_position);
//         let alpha = marker.calculate_alpha(cat, pdist);
//         if alpha < 0.1 {
//             return None;
//         }
//         let mut scale = marker.icon_size.unwrap_or(cat.inherited_template.icon_size.unwrap_or(1.0));
//         let min_size = marker.min_size.unwrap_or_else(|| cat.inherited_template.min_size.unwrap_or(10));
//         // calculate the scaling by deciding how far can a billboard be before it stops shrinking
//         // limiting the depth basically decides its size in a way. 
//         let depth = (vp * pos.push(1.0)).w;
//         // target_depth of 10.0 makes icons 50px sized on my fhd monitor. 
//         let target_depth: f32 = 10.0;
//         // we will calculate how long the depth should be checking how much the min depths is compared 50 px and multiply it with target_depth
//         let target_depth = target_depth * 50.0 / min_size as f32;
//         // we select which ever scale is bigger
//         scale =  scale.max(depth / target_depth) ;
        

// // starting making a billboard from position and texture coordinates

//             // billboard dimensions in meters
//             let billboard_height: f32 =  scale;
//             let billboard_width: f32 = scale;
//             let to_camera = normalize(&(camera_position - pos));
//             let up = make_vec3(&[0.0, 1.0, 0.0]);
//             let right = cross(&to_camera, &up);

//             // change position to left bottom vtx by moving it half width left and half width down.
//             let pos: Vec3 = (pos - (right * billboard_width / 2.0)) - (up * billboard_height / 2.0);
//             let lb: Vec4 = vp * ((pos).push(1.0));
//             // move it one width up to get left top
//             let pos = pos + (up * billboard_height);
//             let lt: Vec4 = vp * (pos).push(1.0);
//             // move it one width right to get right top
//             let pos = pos + (right * billboard_width);
//             let rt: Vec4 = vp * (pos).push(1.0);
//             // move it to one width down to get right bottom
//             let pos = pos - (up * billboard_height);
//             let rb: Vec4 = vp * (pos).push(1.0);

      

//             // first triangle
//             let t1 = Triangle {
//                 vertices: [
//                     MarkerVertex {
//                         vpos: lb.into(),
//                         tex_coords: [0.0, 0.0, z],
//                         alpha,
//                     },
//                     MarkerVertex {
//                         vpos: lt.into(),
//                         tex_coords: [0.0, y, z],
//                         alpha,
//                     },MarkerVertex {
//                         vpos: rt.into(),
//                         tex_coords: [x, y, z],
//                         alpha,
//                     }
//                 ],
//             };
   
//             // second triangle
//             let t2 = Triangle {
//                 vertices: [
//                     MarkerVertex {
//                         vpos: lb.into(),
//                         tex_coords: [0.0, 0.0, z],
//                         alpha,
//                     },
//                     MarkerVertex {
//                         vpos: rt.into(),
//                         tex_coords: [x, y, z],
//                         alpha,
//                     },MarkerVertex {
//                         vpos: rb.into(),
//                         tex_coords: [x, 0.0, z],
//                         alpha,
//                     }
//                 ],
//             };

//             Some(Quad {
//                 triangles: [t1, t2],
//             })
          
//     }
// }
impl Quad {
    pub fn new(marker: &POI,cat: &IMCategory, link: &MumbleLink, view: Mat4,proj: Mat4, window: &GlfwWindow,  tex_coords: (u32, f32, f32, u32), znear: f32, zfar: f32) -> Option<Quad>  {
        let x = tex_coords.1;
        let y = tex_coords.2;
        let z = tex_coords.3 as f32;
        // billboard size in pixels
        let size: u32 = match tex_coords.0 {
            0 => 32 ,
            1 => 64,
            2 => 128,
            3 => 256,
            4 => 512,
            5 => 1024,
            6 => 2048,
            _ => {
                log::error!("texture image size too big or small");
                panic!()
            }
        };
        
        let width = window.window_size.0 as f32;
        let height = window.window_size.1 as f32;

        let viewport = make_vec4(&[0.0, 0.0,  width as f32, height as f32]);
        let pos: Vec3 = marker.pos.into();
        let vp = proj * view;
        
        let cdist = glm::distance(&pos, &link.f_camera_position.into());
        let pdist = glm::distance(&pos, &link.f_avatar_position.into());
        let alpha = marker.calculate_alpha(cat, pdist);
        if alpha < 0.1 {
            return None;
        }
        let mut scale = marker.icon_size.unwrap_or(cat.inherited_template.icon_size.unwrap_or(1.0));
        let min_size = marker.min_size.unwrap_or_else(|| cat.inherited_template.min_size.unwrap_or(5));
        let max_size = marker.max_size.unwrap_or_else(|| cat.inherited_template.max_size.unwrap_or(2048));

        // in screen coordinates. so, x/y are in pixels
        let mut screen_pos = glm::project(&pos, &view, &proj, viewport);
        if screen_pos.z < 0.0 || screen_pos.z > 1.0 {
            return None;
        }
        let size = size as f32 * (1.0 - cdist / (zfar - znear));
        let size = f32::max(size, min_size as f32);
        let size = f32::min(size, max_size as f32);

        // now they are in 0.0 - 1.0 range
        //2.0 * pos.x / screen_size.x - 1.0, 1.0 - 2.0 * pos.y / screen_size.y,
        screen_pos.x =2.0 * screen_pos.x / width - 1.0;
        screen_pos.y = 2.0 * screen_pos.y / height - 1.0;
        screen_pos.z = 0.0;

// starting making a billboard from position and texture coordinates
            // let size = 50;
            // we calculate size of billboard between 0.0 and 1.0 by multiplying texture's size
            // then we scale it 
            let billboard_height: f32 =  scale * (1.0/height) * size as f32 * y;
            let billboard_width: f32 = scale * (1.0/width ) * size as f32 * x;
            
            // change position to left bottom vtx by moving it half width left and half width down.
            let lb: Vec4 = [screen_pos.x - billboard_width / 2.0, screen_pos.y - billboard_height / 2.0, 0.0, 1.0].into();
            // move it one width up to get left top
            let lt: Vec4 = [lb.x, lb.y + billboard_height, 0.0, 1.0].into();
            // move it one width right to get right top
            let rt: Vec4 = [lt.x + billboard_width, lt.y, 0.0, 1.0].into();
            // move it to one width down to get right bottom
            let rb: Vec4 = [rt.x, rt.y - billboard_height, 0.0, 1.0].into();

      

            // first triangle
            let t1 = Triangle {
                vertices: [
                    MarkerVertex {
                        vpos: lb.into(),
                        tex_coords: [0.0, 0.0, z],
                        alpha,
                    },
                    MarkerVertex {
                        vpos: lt.into(),
                        tex_coords: [0.0, y, z],
                        alpha,
                    },MarkerVertex {
                        vpos: rt.into(),
                        tex_coords: [x, y, z],
                        alpha,
                    }
                ],
            };
   
            // second triangle
            let t2 = Triangle {
                vertices: [
                    MarkerVertex {
                        vpos: lb.into(),
                        tex_coords: [0.0, 0.0, z],
                        alpha,
                    },
                    MarkerVertex {
                        vpos: rt.into(),
                        tex_coords: [x, y, z],
                        alpha,
                    },MarkerVertex {
                        vpos: rb.into(),
                        tex_coords: [x, 0.0, z],
                        alpha,
                    }
                ],
            };

            Some(Quad {
                triangles: [t1, t2],
            })
          
    }
}