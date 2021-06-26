use std::path::Path;

use glow::{Context, HasContext};
use image::GenericImageView;

pub struct Tex2D<'a> {
    gl: &'a Context,
    id: u32,
}

impl<'a> Tex2D<'a> {
    pub fn new(gl: &'a Context, img_path: Option<&Path>) -> Self {
        let img_path = img_path.unwrap_or(Path::new("./res/tex.png"));
        unsafe {
            let id = gl.create_texture().unwrap();
            gl.bind_texture(glow::TEXTURE_2D, Some(id)); // all upcoming GL_TEXTURE_2D operations now have effect on this texture object
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
            let img = image::open(&img_path).expect("Failed to load texture");
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
            Tex2D { gl, id }
        }
    }
}

impl Drop for Tex2D<'_> {
    fn drop(&mut self) {
        unsafe {
            self.gl.delete_texture(self.id);
        }
    }
}
