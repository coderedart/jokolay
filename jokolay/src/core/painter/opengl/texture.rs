use std::rc::Rc;

use crate::gl_error;
use glow::{Context, HasContext, NativeTexture};

/// A texture manager that manages a texture array, and uses texture atlassing to make sure that we can do a single render pass for vertices using any texture
/// We will have 3 lvls of indirection. first, a texture array on gpu. we will keep track of the textures on a array layer using the atlas allocators.
/// resizing the array to get a new layer, copying old contents, when we can't fit a texture into the existing layers.
/// then we take the RID of a target texture, check if its in array using tmap, uploading the texture if necessary. We only need to think of a way to deallocate textures.
#[derive(Debug)]
pub struct TextureServer {
    /// The texture array on gpu. everytime we resize, we will create new one, and replace this after copying the pixels and dropping the old one.
    id: NativeTexture,
    /// The depth of the array
    depth: u32,
    /// The gl clone so that we can use it to create/drop texturearrays or stuff like that.
    gl: Rc<Context>,
}

impl TextureServer {
    /// The Width of the texture Array
    pub const WIDTH: u32 = 2048;
    /// The height of the texture array
    pub const HEIGHT: u32 = 2048;
    /// Mipmap levels of the texture array based on f32::floor(f32::log2(Self::WIDTH as f32)) as u32 + 1
    pub const MIPMAP_LEVELS: u32 = 11;

    /// create a new texture manager with empty map. when we start drawing, they will automatically get uploaded.
    pub fn new(gl: Rc<Context>) -> Self {
        let id = Self::create_tex_array(gl.clone());
        unsafe {
            gl_error!(gl);
        }
        let depth = 1_u32;
        unsafe {
            gl.texture_storage_3d(
                id,
                Self::MIPMAP_LEVELS as i32,
                glow::RGBA8,
                Self::WIDTH as i32,
                Self::HEIGHT as i32,
                depth as i32,
            );
        }
        unsafe {
            gl_error!(gl);
        }
        log::trace!("created new texture server. {:?}", id);
        Self { gl, depth, id }
    }

    fn create_tex_array(gl: Rc<Context>) -> NativeTexture {
        unsafe {
            //create texture buffer id and initialize its state and set its type to target
            let id = gl.create_textures(glow::TEXTURE_2D_ARRAY).unwrap();
            // no need to bind it for initialization, but still need to bind so that shaders can access it without bindless
            gl.bind_texture(glow::TEXTURE_2D_ARRAY, Some(id));
            //if texture coordinates are outside of range 0.0-1.0, it will just start over from beginning and thus repeat itself
            gl.texture_parameter_i32(id, glow::TEXTURE_WRAP_S, glow::REPEAT as i32);
            gl.texture_parameter_i32(id, glow::TEXTURE_WRAP_T, glow::REPEAT as i32);
            // when the pixel is big matches multiple texels or when pixel small and matches less than one texel.
            gl.texture_parameter_i32(id, glow::TEXTURE_MIN_FILTER, glow::NEAREST as i32);
            gl.texture_parameter_i32(id, glow::TEXTURE_MAG_FILTER, glow::NEAREST as i32);

            gl_error!(gl);
            log::debug!("created texture array. id: {:?}", id);
            id
        }
    }

    pub fn upload_pixels(
        &self,
        pixels: &[u8],
        x_offset: i32,
        y_offset: i32,
        z_offset: i32,
        width: u32,
        height: u32,
    ) {
        unsafe {
            gl_error!(self.gl);
        }
        log::debug!(
            "uploading texture. width: {}, height: {}, z_offset: {}, y_offset: {}, x_offset: {}",
            width,
            height,
            z_offset,
            y_offset,
            x_offset
        );
        unsafe {
            self.gl.texture_sub_image_3d(
                self.id,
                0,
                x_offset,
                y_offset,
                z_offset,
                width as i32,
                height as i32,
                1,
                glow::RGBA,
                glow::UNSIGNED_BYTE,
                glow::PixelUnpackData::Slice(pixels),
            );
            gl_error!(self.gl);

            self.gl.generate_texture_mipmap(self.id);
        }
    }

    pub fn bump_tex_array_size(&mut self, new_len: Option<u32>) {
        unsafe {
            gl_error!(self.gl);
        }

        let new_tex = Self::create_tex_array(self.gl.clone());
        unsafe {
            gl_error!(self.gl);
        }

        let old_depth = self.depth;
        let new_depth = new_len.unwrap_or(old_depth + 1);
        log::debug!(
            "bumping texture array from {} to {} layers",
            old_depth,
            new_depth
        );
        unsafe {
            gl_error!(self.gl);

            self.gl.texture_storage_3d(
                new_tex,
                Self::MIPMAP_LEVELS as i32,
                glow::RGBA8,
                Self::WIDTH as i32,
                Self::HEIGHT as i32,
                new_depth as i32,
            );
            gl_error!(self.gl);

            self.gl.copy_image_sub_data(
                self.id,
                glow::TEXTURE_2D_ARRAY,
                0,
                0,
                0,
                0,
                new_tex,
                glow::TEXTURE_2D_ARRAY,
                0,
                0,
                0,
                0,
                Self::WIDTH as i32,
                Self::HEIGHT as i32,
                old_depth as i32,
            );
            let old_tex = self.id;
            gl_error!(self.gl);

            self.gl.delete_texture(old_tex);
        }
        self.id = new_tex;
        self.depth = new_depth;
    }
    pub fn bind(&self) {
        unsafe {
            self.gl.active_texture(glow::TEXTURE0);
            self.gl.bind_texture(glow::TEXTURE_2D_ARRAY, Some(self.id));
        }
    }
}
impl Drop for TextureServer {
    fn drop(&mut self) {
        unsafe {
            log::debug!("deleting texture array. {:?}", self.id);
            self.gl.delete_texture(self.id);
        }
    }
}
