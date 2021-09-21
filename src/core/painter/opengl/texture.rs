use std::{collections::HashMap, rc::Rc, sync::Arc};

use crate::{
    core::fm::{FileManager, RID},
    gl_error,
};
use egui::Color32;
use glow::{Context, HasContext, NativeTexture};

use guillotiere::*;
use image::GenericImageView;

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
    /// Mipmap levels of the texture array based on f32::floor(f32::log2(Self::WIDTH as f32)) as u32 + 1
    const MIPMAP_LEVELS: u32 = 11;

    /// The default trail texture
    const TRAIL_TEXTURE: &'static [u8] = include_bytes!("../trail_renderer/trail.png");
    /// The default Marker Texture
    const MARKER_TEXTURE: &'static [u8] = include_bytes!("../marker_renderer/HoT.png");

    /// create a new texture manager with empty map. when we start drawing, they will automatically get uploaded.
    pub fn new(gl: Rc<Context>) -> Self {
        let id = Self::create_tex_array(gl.clone());
        unsafe {
            gl_error!(gl);
        }

        unsafe {
            gl.texture_storage_3d(
                id,
                8, //Self::MIPMAP_LEVELS as i32,
                glow::RGBA8,
                Self::WIDTH as i32,
                Self::HEIGHT as i32,
                1 as i32,
            );
        }
        unsafe {
            gl_error!(gl);
        }

        TextureManager {
            gl,
            id,
            layers: vec![AtlasAllocator::new(size2(
                Self::WIDTH as i32,
                Self::HEIGHT as i32,
            ))],
            tmap: Default::default(),
        }
    }

    fn create_tex_array(gl: Rc<Context>) -> NativeTexture {
        unsafe {
            //create texture buffer id and initialize its state and set its type to target
            let id = gl.create_textures(glow::TEXTURE_2D_ARRAY).unwrap();
            // bind it so that it can initialize its state
            gl.bind_texture(glow::TEXTURE_2D_ARRAY, Some(id));
            //if texture coordinates are outside of range 0.0-1.0, it will just start over from beginning and thus repeat itself
            gl.texture_parameter_i32(id, glow::TEXTURE_WRAP_S, glow::REPEAT as i32);
            gl.texture_parameter_i32(id, glow::TEXTURE_WRAP_T, glow::REPEAT as i32);
            // when the pixel is big matches multiple texels or when pixel small and matches less than one texel.
            gl.texture_parameter_i32(id, glow::TEXTURE_MIN_FILTER, glow::NEAREST as i32);
            gl.texture_parameter_i32(id, glow::TEXTURE_MAG_FILTER, glow::NEAREST as i32);

            gl_error!(gl);
            id
        }
    }

    pub fn get_tc(&mut self, id: RID, fm: &FileManager) -> (usize, Allocation) {
        if let Some(t) = self.tmap.get(&id) {
            return *t;
        }
        let img = match id {
            RID::EguiTexture => {
                log::error!("egui texture not found in tmap.");
                panic!()
            }
            RID::MarkerTexture => {
                let img = image::load_from_memory(Self::MARKER_TEXTURE);
                if let Ok(i) = img {
                    i
                } else {
                    log::error!("could not create image from Self::MARKER_TEXTURE");
                    panic!()
                }
            }
            RID::TrailTexture => {
                let img = image::load_from_memory(Self::TRAIL_TEXTURE);
                if let Ok(i) = img {
                    i
                } else {
                    log::error!("could not create image from Self::TRAIL_TEXTURE");
                    panic!()
                }
            }
            RID::VID(fid) => {
                let ifile = fm
                    .paths
                    .get(fid)
                    .unwrap_or_else(|| {
                        log::error!("could not find fid in paths.");
                        panic!()
                    })
                    .open_file()
                    .map_err(|e| {
                        log::error!(
                            "couldn't open image. error: {:?}.\npath: {:?}",
                            &e,
                            fm.paths.get(fid)
                        );
                        e
                    })
                    .unwrap();
                // create a buf reader
                let ireader = std::io::BufReader::new(ifile);
                // create a image::reader and set its format as it cannot use file path to determine the format due to vfspath (i think)
                let mut imgreader = image::io::Reader::new(ireader);
                imgreader.set_format(image::ImageFormat::Png);
                // get the image
                let img = imgreader
                    .decode()
                    .map_err(|e| {
                        log::error!(
                            "image decode error; image path = {:?}; error: {:?}",
                            fm.paths.get(fid),
                            &e
                        );
                        e
                    })
                    .unwrap();
                img
            }
        };
        // flipv bcoz opengl reads images from bottom
        let img = img.flipv();
        // get rgba bytes because png
        let pixels = img.as_bytes();
        let width = img.width();
        let height = img.height();
        let (layer, allocation) = self.allocate_rectangle(width, height);
        let tbox = allocation.rectangle;
        let [x_offset, y_offset] = tbox.min.to_array();
        self.upload_pixels(pixels, x_offset, y_offset, layer as i32, width, height);
        unsafe {
            gl_error!(self.gl);
        }
        self.tmap.insert(id, (layer, allocation));
        if let Some(t) = self.tmap.get(&id) {
            return *t;
        } else {
            log::error!("couldn't find texture even after uplaoding");
            panic!()
        }
    }
    fn upload_pixels(
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

    fn allocate_rectangle(&mut self, width: u32, height: u32) -> (usize, Allocation) {
        assert!(width <= Self::WIDTH);
        assert!(height <= Self::HEIGHT);
        for (layer, atlas) in self.layers.iter_mut().enumerate() {
            let rect = atlas.allocate(size2(width as i32, height as i32));
            if let Some(a) = rect {
                return (layer, a);
            }
        }
        let mut atlas = AtlasAllocator::new(size2(Self::WIDTH as i32, Self::HEIGHT as i32));
        let rect = atlas.allocate(size2(width as i32, height as i32));
        if let Some(a) = rect {
            let layer = self.layers.len();
            self.bump_tex_array_size();
            unsafe {
                gl_error!(self.gl);
            }
            self.layers.push(atlas);
            return (layer, a);
        }
        panic!("couldn't allocate rectangle");
    }
    pub fn update_etex(&mut self, t: Arc<egui::Texture>) {
        if let Some((layer, a)) = self.tmap.remove(&RID::EguiTexture) {
            self.layers.get_mut(layer).unwrap().deallocate(a.id);
        }
        let mut pixels = Vec::new();
        for &alpha in &t.pixels {
            let rgba = Color32::from_white_alpha(alpha);
            let a = rgba.to_array();
            pixels.extend_from_slice(&a);
            // pixels.push(rgba.r());
            // pixels.push(rgba.g());
            // pixels.push(rgba.b());
            // pixels.push(rgba.a());
        }
        let width = t.width as u32;
        let height = t.height as u32;
        let (layer, allocation) = self.allocate_rectangle(width, height);
        let tbox = allocation.rectangle;
        let [x_offset, y_offset] = tbox.min.to_array();
        unsafe {
            gl_error!(self.gl);
        }

        self.upload_pixels(&pixels, x_offset, y_offset, layer as i32, width, height);
        unsafe {
            gl_error!(self.gl);
        }
        self.tmap.insert(RID::EguiTexture, (layer, allocation));
    }

    fn bump_tex_array_size(&mut self) {
        unsafe {
            gl_error!(self.gl);
        }

        let new_tex = Self::create_tex_array(self.gl.clone());
        unsafe {
            gl_error!(self.gl);
        }

        let old_depth = self.layers.len() as u32;
        let new_depth = old_depth + 1;
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
                new_depth as i32,
            );
            let old_tex = self.id;
            gl_error!(self.gl);

            self.gl.delete_texture(old_tex);
            self.id = new_tex;
        }
    }
    pub fn bind(&self) {
        unsafe {
            self.gl.active_texture(glow::TEXTURE0);
            self.gl.bind_texture(glow::TEXTURE_2D_ARRAY, Some(self.id));
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
