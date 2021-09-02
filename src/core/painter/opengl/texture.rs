use std::{collections::HashMap, rc::Rc, sync::Arc};

use crate::{
    core::fm::{FileManager, RID},
    gl_error,
};
use glow::{Context, HasContext, NativeTexture};

use guillotiere::*;

/// A texture manager that manages a texture array, and uses texture atlassing to make sure that we can do a single render pass for vertices using any texture
/// We will have 3 lvls of indirection. first, a texture array on gpu. we will keep track of the textures on a array layer using the atlas allocators.
/// resizing the array to get a new layer, copying old contents, when we can't fit a texture into the existing layers. 
/// then we take the RID of a target texture, check if its in array using tmap, uploading the texture if necessary. We only need to think of a way to deallocate textures.
pub struct TextureManager {
    /// The texture array on gpu. everytime we resize, we will create new one, and replace this after copying the pixels and dropping the old one.
    id: NativeTexture,
    /// This will track the each layer of texture as a rectangle, and can be used to check which has enough free space to fit in our incoming texture
    layers: Vec<AtlasAllocator>,
    /// the map will contain the RID and where that texture is allocated. if its not, it will get allocated and uploaded. 
    tmap: HashMap<RID, (usize, Allocation)>,
    /// The gl clone so that we can use it to create/drop texturearrays or stuff like that. 
    gl: Rc<Context>,
}

impl TextureManager {
    /// The Width of the texture Array
    pub const WIDTH: u32 = 2048;
    /// The height of the texture array
    pub const HEIGHT: u32 = 2048;
    /// The target which is TEXTURE_2D_Array
    pub const  TARGET: u32 = glow::TEXTURE_2D_ARRAY;
    /// The default trail texture
    pub const TRAIL_TEXTURE: &'static [u8] = include_bytes!("../trail_renderer/trail.png");
    /// The default Marker Texture
    pub const MARKER_TEXTURE: &'static [u8] = include_bytes!("../marker_renderer/HoT.png");

    /// create a new texture manager with empty map. when we start drawing, they will automatically get uploaded. 
    pub fn new(gl: Rc<Context>) -> Self {
        unsafe {
            //create texture buffer id and initialize its state and set its type to target
            let id = gl.create_textures(Self::TARGET).unwrap();

            //if texture coordinates are outside of range 0.0-1.0, it will just start over from beginning and thus repeat itself
            gl.texture_parameter_i32(id, glow::TEXTURE_WRAP_S, glow::REPEAT as i32);
            gl.texture_parameter_i32(id, glow::TEXTURE_WRAP_T, glow::REPEAT as i32);
            // when the pixel is big matches multiple texels or when pixel small and matches less than one texel.
            gl.texture_parameter_i32(id, glow::TEXTURE_MIN_FILTER, glow::LINEAR as i32);
            gl.texture_parameter_i32(id, glow::TEXTURE_MAG_FILTER, glow::LINEAR as i32);
            gl_error!(gl);
            TextureManager {
                gl,
                id,
                layers: vec![],
                tmap: Default::default(),
            }
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
            )
        }
    }
    
    pub fn bind(&self) {
        unsafe {
            self.gl.active_texture(glow::TEXTURE0 );
            self.gl.bind_texture(Self::TARGET, Some(self.id));
        }
    }

  
}
impl Drop for TextureManager {
    fn drop(&mut self) {
        unsafe {
            self.gl.delete_texture(self.id);
        }
    }
}