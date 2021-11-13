use std::{
    convert::TryInto,
    path::{Path, PathBuf},
    sync::Arc,
};

use ahash::{AHashMap, AHashSet};
use egui::{Color32, Pos2, Rect, TextureId, Vec2};
use flume::Receiver;
use guillotiere::*;
use image::GenericImageView;

use crate::{client::tc::TextureClient, core::painter::{opengl::texture::TextureServer, RenderCommand}};

#[derive(Debug, Clone, Copy)]
pub struct AllocatedTexture {
    pub allocation: Allocation,
    pub layer: usize,
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
        Vec2 {
            x: self.get_width_px() as f32 / TextureClient::WIDTH as f32,
            y: self.get_height_px() as f32 / TextureClient::HEIGHT as f32,
        }
    }
    pub fn get_tex_coords(&self) -> TextureCoordinates {
        let Vec2 { x, y } = self.get_scale();
        let scalex = x;
        let scaley = y;
        let Rect { min, max: _ } = self.get_normalized_rectangle();
        TextureCoordinates {
            startx: min.x,
            starty: min.y,
            scalex,
            scaley,
            layer: self
                .layer
                .try_into()
                .expect("could not fit usize into i32 for texture"),
        }
    }
    pub fn get_normalized_rectangle(&self) -> Rect {
        let startx = self.allocation.rectangle.min.x as f32 / TextureClient::WIDTH as f32;
        let starty = self.allocation.rectangle.min.y as f32 / TextureClient::HEIGHT as f32;
        let endx = self.allocation.rectangle.max.x as f32 / TextureClient::WIDTH as f32;
        let endy = self.allocation.rectangle.max.y as f32 / TextureClient::HEIGHT as f32;
        Rect::from_min_max(
            Pos2 {
                x: startx,
                y: starty,
            },
            Pos2 { x: endx, y: endy },
        )
    }
}
