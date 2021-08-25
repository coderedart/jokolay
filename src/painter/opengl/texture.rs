use std::{collections::HashMap, rc::Rc, sync::Arc};

use crate::{
    fm::{FileManager, VID},
    gl_error,
};
use egui::{Color32, TextureId};
use glow::{Context, HasContext, NativeTexture};

use image::GenericImageView;

#[derive(Debug)]
pub struct Texture {
    gl: Rc<Context>,
    pub id: NativeTexture,
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
            gl.tex_parameter_i32(target, glow::TEXTURE_MIN_FILTER, glow::LINEAR as i32);
            gl.tex_parameter_i32(target, glow::TEXTURE_MAG_FILTER, glow::LINEAR as i32);

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

#[derive(Debug)]
pub struct TextureArray {
    pub id: NativeTexture,
    pub slot: u32,
    pub target: u32,
    pub width: u32,
    pub height: u32,
    pub layers: u32,
    pub length: u32,
    pub bump_size: u32,
    gl: Rc<Context>,
}

impl TextureArray {
    pub fn new(gl: Rc<Context>, slot: u32) -> Self {
        let target = glow::TEXTURE_2D_ARRAY;
        unsafe {
            //create texture buffer id
            let id = gl.create_texture().unwrap();
            gl.active_texture(glow::TEXTURE0 + slot);
            //initialize its state and set its type to target
            gl.bind_texture(target, Some(id));
            //if texture coordinates are outside of range 0.0-1.0, it will just start over from beginning and thus repeat itself
            gl.tex_parameter_i32(target, glow::TEXTURE_WRAP_S, glow::REPEAT as i32);
            gl.tex_parameter_i32(target, glow::TEXTURE_WRAP_T, glow::REPEAT as i32);
            // when the pixel is big matches multiple texels or when pixel small and matches less than one texel.
            gl.tex_parameter_i32(target, glow::TEXTURE_MIN_FILTER, glow::NEAREST as i32);
            gl.tex_parameter_i32(target, glow::TEXTURE_MAG_FILTER, glow::NEAREST as i32);

            TextureArray {
                gl,
                slot,
                id,
                target,
                width: 0,
                height: 0,
                layers: 0,
                length: 0,
                bump_size: 0,
            }
        }
    }
    // pub fn update_pixels(&mut self, data: &[&[u8]]) {
    //     //can't update buffers without binding to the target
    //     self.bind();
    //     gl_error!(self.gl);

    //     unsafe {
    //         self.gl.tex_storage_3d(self.target, f32::floor(f32::log2(self.width as f32)) as i32 + 1, glow::RGBA8, self.width as i32, self.height as i32, data.len() as i32);
    //         gl_error!(self.gl);

    //         for (layer, &pixels) in data.iter().enumerate() {
    //             self.gl.tex_image_3d(
    //                 self.target,
    //                 0, //mipmap level of the image we inserting
    //                 glow::RGBA as i32,
    //                 self.width as i32,
    //                 self.height as i32,
    //                 layer as i32, //texture array layer number
    //                 0,
    //                 glow::RGBA,
    //                 glow::UNSIGNED_BYTE,
    //                 Some(pixels),
    //             );
    //         }
    //     }
    //     gl_error!(self.gl);

    //     unsafe {
    //         self.gl.generate_mipmap(self.target);
    //     }
    //     gl_error!(self.gl);

    // }
    pub fn reserve_storage(&mut self, width: u32, height: u32, layers: u32, bump_size: u32) {
        self.width = width;
        self.height = height;
        self.layers = layers;
        self.bump_size = bump_size;
        unsafe {
            self.gl.tex_storage_3d(
                self.target,
                f32::floor(f32::log2(self.width as f32)) as i32 + 1,
                glow::RGBA8,
                self.width as i32,
                self.height as i32,
                self.layers as i32,
            );
        }
        gl_error!(self.gl);
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
            self.gl.tex_sub_image_3d(
                glow::TEXTURE_2D_ARRAY,
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
    pub fn add_image(&mut self, pixels: &[u8], width: u32, height: u32) -> (f32, f32, u32) {
        if self.length == self.layers {
            let new_layers = self.layers + self.bump_size;
            let mut new_atex = TextureArray::new(self.gl.clone(), self.slot);
            log::info!("resizing tex array of width {}, height {} and length {} . srcid: {}, srclayers: {}, dstid: {}, dstlayers: {} ", self.width, self.height, self.length, self.id.0.get(), self.layers, new_atex.id.0.get(), new_layers);
            new_atex.bind();
            new_atex.reserve_storage(self.width, self.height, new_layers, self.bump_size);
            unsafe {
                self.gl.raw.CopyImageSubData(
                    self.id.0.get(),
                    glow::TEXTURE_2D_ARRAY,
                    0,
                    0,
                    0,
                    0,
                    new_atex.id.0.get(),
                    glow::TEXTURE_2D_ARRAY,
                    0,
                    0,
                    0,
                    0,
                    self.width as i32,
                    self.height as i32,
                    self.layers as i32,
                );
            }
            new_atex.length = self.length;
            *self = new_atex;
        }
        self.upload_pixels(pixels, 0, 0, self.length as i32, width, height);
        unsafe {
            self.gl.generate_mipmap(glow::TEXTURE_2D_ARRAY);
        }
        let x = width as f32 / self.width as f32;
        let y = height as f32 / self.height as f32;
        let z = self.length;
        self.length += 1;
        (x, y, z)
    }
    pub fn bind(&self) {
        unsafe {
            self.gl.active_texture(glow::TEXTURE0 + self.slot);
            self.gl.bind_texture(self.target, Some(self.id));
        }
    }

    pub fn unbind(&self) {
        unsafe {
            self.gl.active_texture(glow::TEXTURE0 + self.slot);
            self.gl.bind_texture(self.target, None);
        }
    }
}

impl Drop for TextureArray {
    fn drop(&mut self) {
        unsafe {
            self.gl.delete_texture(self.id);
        }
    }
}

pub struct TextureManager {
    pub array_tex: Vec<TextureArray>,
    pub live_images: HashMap<VID, (u32, f32, f32, u32)>,
}
impl TextureManager {
    pub const SMALLEST_TEXTURE_SIZE: usize = 32;
    pub const NUM_OF_ARRAYS: usize = 7;
    pub const LARGEST_TEXTURE_SIZE: usize =
        Self::SMALLEST_TEXTURE_SIZE * 2usize.pow(Self::NUM_OF_ARRAYS as u32);
    fn get_slot(width: u32, height: u32) -> usize {
        let dimension = u32::max(width, height) as usize;
        assert!(dimension >= Self::SMALLEST_TEXTURE_SIZE);
        assert!(dimension <= Self::LARGEST_TEXTURE_SIZE);
        let dimension = if dimension.is_power_of_two() {
            dimension
        } else {
            dimension.next_power_of_two()
        };
        match dimension {
            32 => 0,
            64 => 1,
            128 => 2,
            256 => 3,
            512 => 4,
            1024 => 5,
            2048 => 6,
            _ => {
                log::error!("texture image size too big or small");
                panic!()
            }
        }
    }
    const trail_texture: &'static [u8] = include_bytes!("../../../assets/trail.png");
    /// creates a new texture manager and automatically uploads egui font texture with the argument and VID(0) which is "egui"
    pub fn new(gl: Rc<Context>, t: Arc<egui::Texture>) -> Self {
        let mut arr = Vec::new();
        for i in 0..Self::NUM_OF_ARRAYS {
            let dim = Self::SMALLEST_TEXTURE_SIZE * 2usize.pow(i as u32);
            let mut at = TextureArray::new(gl.clone(), i as u32);
            at.bind();
            at.reserve_storage(dim as u32, dim as u32, 1 as u32, 1);
            arr.push(at);
        }
        // upload the main egui font texture
        let mut pixels = Vec::new();
        for &alpha in &t.pixels {
            let srgba = Color32::from_white_alpha(alpha);
            pixels.push(srgba.r());
            pixels.push(srgba.g());
            pixels.push(srgba.b());
            pixels.push(srgba.a());
        }
        let slot = Self::get_slot(t.width as u32, t.height as u32);
        let (x, y, z) = arr[slot].add_image(&pixels, t.width as u32, t.height as u32);
        let mut live_images = HashMap::new();
        live_images.insert(VID(0), (slot as u32, x, y, z));

        // upload trail texture
        let tt = image::load_from_memory(Self::trail_texture).unwrap();
        
        let tt = tt.flipv();
        let slot = Self::get_slot(tt.width(), tt.height());
        arr[slot].bind();
        let (x, y, z) = arr[slot].add_image(tt.as_bytes(), tt.width(), tt.height());
        live_images.insert(VID(1), (slot as u32, x, y, z));

        TextureManager {
            array_tex: arr,
            live_images,
        }
    }
    /// uploads image into a texture slots and returns a tuple of (slot, x, y z). does not upload image if it already exists.
    pub fn get_image(&mut self, id: VID, fm: &FileManager) -> (u32, f32, f32, u32) {
        // check if the uploaded textures already contains id
        if !self.live_images.contains_key(&id) {
            // upload the image texture
            self.live_images.insert(id, {
                // use vfspath to open the image file
                let ifile = fm
                    .paths
                    .get(id.0)
                    .unwrap()
                    .open_file()
                    .map_err(|e| {
                        log::error!(
                            "couldn't open image. error: {:?}.\npath: {:?}",
                            &e,
                            fm.paths.get(id.0)
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
                            fm.paths.get(id.0),
                            &e
                        );
                        e
                    })
                    .unwrap();
                // flipv bcoz opengl reads images from bottom
                let img = img.flipv();
                // get rgba bytes because png
                let pixels = img.as_bytes();
                // decide which unit the texture needs to be uploaded to
                let slot = Self::get_slot(img.width(), img.height());
                // texture arrays already have a predefined slot based on the image size, so this might be redundunt
                self.array_tex[slot].bind();
                // upload the image to array and get the layer at which we uploaded the texture. array might resize itself to fit
                let (x, y, z) = self.array_tex[slot].add_image(pixels, img.width(), img.height());
                (slot as u32, x, y, z)
            });
        }
        *self.live_images.get(&id).unwrap()
    }
    pub fn get_etex(&mut self, id: egui::TextureId, fm: &FileManager) -> (u32, f32, f32, u32) {
        match id {
            TextureId::Egui => self.get_image(VID(0), fm),
            TextureId::User(_) => todo!(),
        }
    }
}
