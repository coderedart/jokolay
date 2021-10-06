use std::{collections::HashMap, convert::TryInto, sync::Arc};

use egui::{Color32, Pos2, Rect, TextureId, Vec2};
use guillotiere::*;
use image::GenericImageView;

use crate::{core::painter::opengl::texture::TextureManager};
/// struct to simulate a texture array, and manage it as a dynamic atlas. sending commands to a texture manager in core
#[derive(Default, Clone)]
pub struct AtlasManager {
    /// This will track the each layer of texture as a rectangle, and can be used to check which has enough free space to fit in our incoming texture
    layers: Vec<AtlasAllocator>,
    /// the map will contain the RID and where that texture is allocated. if its not, it will get allocated and uploaded.
    atlas_map: AtlasMap,
}
#[derive(Debug, Default, Clone)]
pub struct AtlasMap {
    pub amap: HashMap<egui::TextureId, AllocatedTexture>,
}

impl AtlasMap {
    pub fn get_alloc_tex(&self, id: egui::TextureId) -> Option<AllocatedTexture> {
        self.amap.get(&id).copied()
    }
    pub fn upload_alloc_tex(&mut self, id: TextureId, at: AllocatedTexture) -> Option<AllocatedTexture> {
        self.amap.insert(id, at)
    }
    pub fn delete_alloc_tex(&mut self, id: TextureId) -> Option<AllocatedTexture> {
        self.amap.remove(&id)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct AllocatedTexture {
    allocation: Allocation,
    layer: usize,
}
pub struct TextureCoordinates {
    pub startx: f32,
    pub starty: f32,
    pub scalex: f32,
    pub scaley: f32,
    pub layer: i32,
}
impl AllocatedTexture {
    pub fn new(allocation: Allocation, layer: usize) -> Self {
        Self { allocation, layer }
    }
    pub fn get_width_px(&self) -> u32 {
        self.allocation.rectangle.width() as u32
    }
    pub fn get_height_px(&self) -> u32 {
        self.allocation.rectangle.height() as u32
    }
    pub fn get_scale(&self) -> Vec2 {
        Vec2 { x: self.get_width_px() as f32 / AtlasManager::WIDTH as f32, y: self.get_height_px() as f32 / AtlasManager::HEIGHT as f32}
    }
    pub fn get_tex_coords(&self) -> TextureCoordinates {
        let Vec2 { x, y} = self.get_scale();
        let scalex = x;
        let scaley = y;
        let Rect { min, max: _ } = self.get_normalized_rectangle();
        TextureCoordinates {
            startx: min.x,
            starty: min.y,
            scalex,
            scaley,
            layer: self.layer.try_into().expect("could not fit usize into i32 for texture")
        }
    }
    pub fn get_normalized_rectangle(&self) -> Rect {
        let startx = self.allocation.rectangle.min.x as f32 / AtlasManager::WIDTH as f32;
        let starty = self.allocation.rectangle.min.y as f32 / AtlasManager::HEIGHT as f32;
        let endx = self.allocation.rectangle.max.x as f32 / AtlasManager::WIDTH as f32;
        let endy = self.allocation.rectangle.max.y as f32 / AtlasManager::HEIGHT as f32;
        Rect::from_min_max(
            Pos2 {
                x: startx,
                y: starty,
            },
            Pos2 { x: endx, y: endy },
        )
    }
}

impl AtlasManager {
    /// The Width of the texture Array
    pub const WIDTH: u32 = TextureManager::WIDTH;
    /// The height of the texture array
    pub const HEIGHT: u32 = TextureManager::HEIGHT;
    /// Mipmap levels of the texture array based on f32::floor(f32::log2(Self::WIDTH as f32)) as u32 + 1
    pub const MIPMAP_LEVELS: u32 = TextureManager::MIPMAP_LEVELS;

    /// create a new texture manager with empty map. when we start drawing, they will automatically get uploaded.
    pub fn new() -> Self {
        Self {
            layers: vec![AtlasAllocator::new(size2(
                Self::WIDTH as i32,
                Self::HEIGHT as i32,
            ))],
            atlas_map: Default::default(),
        }
    }

    pub fn get_alloc_tex(&self) -> Option<AllocatedTexture> {
        todo!()
    }
  
    
}
pub fn update_etex(t: Arc<egui::Texture>) {
        
    let mut pixels = Vec::new();
    for &alpha in &t.pixels {
        let rgba = Color32::from_white_alpha(alpha);
        let a = rgba.to_array();
        pixels.extend_from_slice(&a);
    }
}