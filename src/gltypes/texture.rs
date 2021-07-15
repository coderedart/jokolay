use std::rc::Rc;

use glow::{Context, HasContext};

// #[derive(Debug)]
// pub struct TextureArray {
//     pub id: u32,
//     pub target: u32,
//     pub width: u32,
//     pub height: u32,
//     pub depth: u32,
//     gl: Rc<Context>,
// }

// impl TextureArray {
//     pub fn new(gl: Rc<Context>, width: u32, height: u32) -> Self {
//         let target = glow::TEXTURE_2D_ARRAY;
//         unsafe {
//             //create texture buffer id
//             let id = gl.create_texture().unwrap();
//             //initialize its state and set its type to target
//             gl.bind_texture(target, Some(id));
//             //if texture coordinates are outside of range 0.0-1.0, it will just start over from beginning and thus repeat itself
//             gl.tex_parameter_i32(target, glow::TEXTURE_WRAP_S, glow::CLAMP_TO_EDGE as i32);
//             gl.tex_parameter_i32(target, glow::TEXTURE_WRAP_T, glow::CLAMP_TO_EDGE as i32);
//             // when the pixel is big matches multiple texels or when pixel small and matches less than one texel.
//             gl.tex_parameter_i32(target, glow::TEXTURE_MIN_FILTER, glow::NEAREST as i32);
//             gl.tex_parameter_i32(target, glow::TEXTURE_MAG_FILTER, glow::NEAREST as i32);

//             TextureArray {
//                 gl,
//                 id,
//                 target,
//                 width,
//                 height,
//                 depth: 0,
//             }
//         }
//     }
//     pub fn update_pixels(&mut self, data: &[&[u8]]) {
//         //can't update buffers without binding to the target
//         self.bind();
//         gl_error!(self.gl);

//         unsafe {
//             self.gl.tex_storage_3d(self.target, f32::floor(f32::log2(self.width as f32)) as i32 + 1, glow::RGBA8, self.width as i32, self.height as i32, data.len() as i32);
//             gl_error!(self.gl);

//             for (layer, &pixels) in data.iter().enumerate() {
//                 self.gl.tex_image_3d(
//                     self.target,
//                     0, //mipmap level of the image we inserting
//                     glow::RGBA as i32,
//                     self.width as i32,
//                     self.height as i32,
//                     layer as i32, //texture array layer number
//                     0,
//                     glow::RGBA,
//                     glow::UNSIGNED_BYTE,
//                     Some(pixels),
//                 );
//             }
//         }
//         gl_error!(self.gl);

//         unsafe {
//             self.gl.generate_mipmap(self.target);
//         }
//         gl_error!(self.gl);

//     }

//     pub fn bind(&self) {
//         unsafe {
//             self.gl.bind_texture(self.target, Some(self.id));
//         }
//     }

//     pub fn unbind(&self) {
//         unsafe {
//             self.gl.bind_texture(self.target, None);
//         }
//     }
// }

// impl Drop for TextureArray {
//     fn drop(&mut self) {
//         unsafe {
//             self.gl.delete_texture(self.id);
//         }
//     }
// }

#[derive(Debug)]
pub struct Texture {
    gl: Rc<Context>,
    pub id: u32,
    pub target: u32,
}

impl Texture {
    pub fn new(gl: Rc<Context>) -> Self {
        let target = glow::TEXTURE_2D;
        unsafe {
            //create texture buffer id
            let id = gl.create_texture().unwrap();
            //initialize its state and set its type to target
            gl.bind_texture(target, Some(id));
            //if texture coordinates are outside of range 0.0-1.0, it will just start over from beginning and thus repeat itself
            gl.tex_parameter_i32(target, glow::TEXTURE_WRAP_S, glow::CLAMP_TO_EDGE as i32);
            gl.tex_parameter_i32(target, glow::TEXTURE_WRAP_T, glow::CLAMP_TO_EDGE as i32);
            // when the pixel is big matches multiple texels or when pixel small and matches less than one texel.
            gl.tex_parameter_i32(target, glow::TEXTURE_MIN_FILTER, glow::NEAREST as i32);
            gl.tex_parameter_i32(target, glow::TEXTURE_MAG_FILTER, glow::NEAREST as i32);

            Texture { gl, id, target }
        }
    }
    pub fn update_pixels(&self, data: &[u8], width: u32, height: u32) {
        //can't update buffers without binding to the target
        self.bind();
        unsafe {
            // load image, create texture and generate mipmaps
            self.gl.tex_image_2d(
                self.target,
                0,
                glow::RGBA as i32,
                width as i32,
                height as i32,
                0,
                glow::RGBA,
                glow::UNSIGNED_BYTE,
                Some(data),
            );
        }

        unsafe {
            self.gl.generate_mipmap(self.target);
        }
    }

    pub fn bind(&self) {
        unsafe {
            self.gl.bind_texture(self.target, Some(self.id));
        }
    }

    pub fn unbind(&self) {
        unsafe {
            self.gl.bind_texture(self.target, None);
        }
    }
}

impl Drop for Texture {
    fn drop(&mut self) {
        unsafe {
            self.gl.delete_texture(self.id);
        }
    }
}
