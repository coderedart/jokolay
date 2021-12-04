pub mod atlas;

use std::{
    path::{Path, PathBuf},
    sync::Arc,
};

use ahash::{AHashMap, AHashSet};
use egui::{Color32, TextureId};
use flume::Receiver;
use guillotiere::*;
use image::GenericImageView;

use crate::{
    client::tc::atlas::AllocatedTexture,
    core::painter::{opengl::texture::TextureServer, RenderCommand},
};

/// struct to simulate a texture array, and manage it as a dynamic atlas. sending commands to a texture manager in core
#[derive(Clone)]
pub struct TextureClient {
    /// This will track the each layer of texture as a rectangle, and can be used to check which has enough free space to fit in our incoming texture
    layers: Vec<AtlasAllocator>,
    /// This will map textures to a egui texture id after allocating them. the primary atlas map.
    pub eid_tex_map: AHashMap<egui::TextureId, AllocatedTexture>,
    /// This takes the allocated tex AND its egui id to map it to a path (if its from web, it will be a temp file)
    pub fs_eid_map: AHashMap<PathBuf, (egui::TextureId, AllocatedTexture)>,
    /// handle to tokio runtime to spawn async texture loads
    pub handle: tokio::runtime::Handle,
    /// The async texture loading status
    pub async_tex_loads: AHashMap<PathBuf, Receiver<TextureLoadStatus>>,
    /// The Texture Commands buffer to send to server
    pub tex_commands: Option<Vec<RenderCommand>>,
    pub egui_texture_version: Option<u64>,
}

#[derive(Debug, Clone)]
pub enum TextureLoadStatus {
    Success(image::DynamicImage),
    Failed(String),
}

impl TextureClient {
    /// The Width of the texture Array
    pub const WIDTH: u32 = TextureServer::WIDTH;
    /// The height of the texture array
    pub const HEIGHT: u32 = TextureServer::HEIGHT;
    /// Mipmap levels of the texture array based on f32::floor(f32::log2(Self::WIDTH as f32)) as u32 + 1
    pub const MIPMAP_LEVELS: u32 = TextureServer::MIPMAP_LEVELS;

    /// create a new texture manager with empty map. when we start drawing, they will automatically get uploaded.
    pub fn new(handle: tokio::runtime::Handle) -> Self {
        Self {
            layers: vec![AtlasAllocator::new(size2(
                Self::WIDTH as i32,
                Self::HEIGHT as i32,
            ))],
            eid_tex_map: Default::default(),
            fs_eid_map: Default::default(),
            async_tex_loads: Default::default(),
            tex_commands: Default::default(),
            handle,
            egui_texture_version: None,
        }
    }

    pub fn get_alloc_tex(&self, eid: TextureId) -> Option<AllocatedTexture> {
        self.eid_tex_map.get(&eid).cloned()
    }
    pub fn is_being_allocated(&self, path: &Path) -> bool {
        self.async_tex_loads.contains_key(path)
    }
    pub fn allocate_if_needed(&mut self, path: &Path) {
        if !self.fs_eid_map.contains_key(path) && !self.async_tex_loads.contains_key(path) {
            self.allocate_image(path.to_path_buf());
        }
    }
    fn allocate_image(&mut self, path: PathBuf) {
        let copy_path = path.clone();
        let (s, r) = flume::bounded::<TextureLoadStatus>(1);
        self.handle.spawn(async move {
            match tokio::fs::read(path).await {
                Ok(buffer) => {
                    match image::load_from_memory_with_format(&buffer, image::ImageFormat::Png) {
                        Ok(i) => s.send_async(TextureLoadStatus::Success(i)).await.unwrap(),
                        Err(e) => s
                            .send_async(TextureLoadStatus::Failed(format!(
                                "failed to get png image from the file. {:?}",
                                &e
                            )))
                            .await
                            .unwrap(),
                    }
                }
                Err(e) => s
                    .send_async(TextureLoadStatus::Failed(format!(
                        "failed to read image file. {:?}",
                        &e
                    )))
                    .await
                    .unwrap(),
            }
        });
        self.async_tex_loads.insert(copy_path, r);
    }
    pub fn update_egui(&mut self, t: Arc<egui::Texture>) {
        if Some(t.version) != self.egui_texture_version {
            self.egui_texture_version = Some(t.version);
            self.deallocate_tex(TextureId::Egui);
            let pixels = Self::get_pixels(t.clone());
            let width = t.width as u32;
            let height = t.height as u32;
            let at = self.allocate_upload_pixels(width, height, pixels);
            self.eid_tex_map.insert(TextureId::Egui, at);
        }
    }
    fn allocate_upload_pixels(
        &mut self,
        width: u32,
        height: u32,
        pixels: Vec<u8>,
    ) -> AllocatedTexture {
        for (layer, atlas) in self.layers.iter_mut().enumerate() {
            if let Some(allocation) = atlas.allocate(size2(width as i32, height as i32)) {
                let allocated_texture = AllocatedTexture::new(allocation, layer);
                if self.tex_commands.is_none() {
                    self.tex_commands = Some(vec![]);
                }
                if let Some(ref mut tm) = self.tex_commands {
                    tm.push(RenderCommand::TextureUpload {
                        pixels,
                        x_offset: allocated_texture.allocation.rectangle.min.x,
                        y_offset: allocated_texture.allocation.rectangle.min.y,
                        z_offset: layer as i32,
                        width: allocated_texture.allocation.rectangle.width(),
                        height: allocated_texture.allocation.rectangle.height(),
                    });
                }
                return allocated_texture;
            }
        }
        let mut a = AtlasAllocator::new(size2(Self::WIDTH as i32, Self::HEIGHT as i32));
        if self.tex_commands.is_none() {
            self.tex_commands = Some(vec![]);
        }
        if let Some(ref mut tm) = self.tex_commands {
            tm.push(RenderCommand::BumpTextureArraySize);
        }
        if let Some(at) = a.allocate(size2(width as i32, height as i32)) {
            let layer = self.layers.len();
            let allocated_texture = AllocatedTexture::new(at, layer);
            if self.tex_commands.is_none() {
                self.tex_commands = Some(vec![]);
            }
            if let Some(ref mut tm) = self.tex_commands {
                tm.push(RenderCommand::TextureUpload {
                    pixels,
                    x_offset: allocated_texture.allocation.rectangle.min.x,
                    y_offset: allocated_texture.allocation.rectangle.min.y,
                    z_offset: layer as i32,
                    width: allocated_texture.allocation.rectangle.width(),
                    height: allocated_texture.allocation.rectangle.height(),
                });
            }
            self.layers.push(a);
            allocated_texture
        } else {
            panic!("could not allocate texture.")
        }
    }

    fn deallocate_tex(&mut self, eid: TextureId) {
        if let Some(t) = self.eid_tex_map.remove(&eid) {
            if let Some(a) = self.layers.get_mut(t.layer) {
                a.deallocate(t.allocation.id);
            }
        }
    }
    fn get_pixels(t: Arc<egui::Texture>) -> Vec<u8> {
        let mut pixels = Vec::new();
        for &alpha in &t.pixels {
            let rgba = Color32::from_white_alpha(alpha);
            let a = rgba.to_array();
            pixels.extend_from_slice(&a);
        }
        pixels
    }
    pub fn tick(&mut self, t: Arc<egui::Texture>) {
        self.update_egui(t);
        let mut delete_set = AHashSet::default();
        let mut upload_jobs = vec![];
        for (p, ts) in self.async_tex_loads.iter_mut() {
            match ts.try_recv() {
                Ok(status) => match status {
                    TextureLoadStatus::Success(i) => {
                        upload_jobs.push((p.clone(), i));
                    }
                    TextureLoadStatus::Failed(i) => {
                        log::error!("Texture load failed due to error: {}", &i);
                    }
                },
                Err(e) => match e {
                    flume::TryRecvError::Empty => continue,
                    flume::TryRecvError::Disconnected => log::error!(
                        "disconnected async load job.\n path: {:?}\n error: {:?}",
                        &p,
                        &e
                    ),
                },
            }
            delete_set.insert(p.clone());
        }
        for (p, i) in upload_jobs {
            let width = i.width();
            let height = i.height();
            let pixels = i.to_bytes();
            let at = self.allocate_upload_pixels(width, height, pixels);
            let id = at.allocation.id.serialize() as u64;
            if let Some(previous) = self.eid_tex_map.insert(TextureId::User(id), at) {
                log::error!("had a previous allocation. something very wrong. previous id: {:?}, present id: {:?} and map: {:?} ", previous, at, &self.eid_tex_map);
            }
            if let Some(previous) = self.fs_eid_map.insert(p, (TextureId::User(id), at)) {
                log::error!("had a previous allocation. something very wrong. previous id: {:?}, present id: {:?} and map: {:?} ", previous, at, &self.fs_eid_map);
            }
        }
        for e in delete_set {
            self.async_tex_loads.remove(&e);
        }
    }
}
