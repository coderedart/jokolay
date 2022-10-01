use egui_backend::{GfxBackend, WindowBackend};
use egui_render_wgpu::wgpu::{
    BindGroupDescriptor, BindGroupEntry, BindingResource, Extent3d, ImageCopyTexture,
    ImageDataLayout, Origin3d, TextureAspect, TextureDescriptor, TextureDimension, TextureFormat,
    TextureUsages, TextureViewDescriptor, TextureViewDimension,
};
use egui_render_wgpu::{wgpu::BindGroup, WgpuBackend, WgpuSettings};
use glam::Vec3;
use intmap::IntMap;
use std::num::NonZeroU32;

pub struct JokoRenderer {
    pub wgpu_backend: egui_render_wgpu::WgpuBackend,
    pub textures: intmap::IntMap<BindGroup>,
    pub markers: Vec<MarkerQuad>,
}

impl<W: WindowBackend> GfxBackend<W> for JokoRenderer {
    type Configuration = WgpuSettings;

    fn new(window_backend: &mut W, settings: Self::Configuration) -> Self {
        let wgpu_backend = WgpuBackend::new(window_backend, settings);
        Self {
            wgpu_backend,
            textures: IntMap::new(),
            markers: Vec::new(),
        }
    }

    fn prepare_frame(&mut self, framebuffer_needs_resize: bool, window_backend: &mut W) {
        self.wgpu_backend
            .prepare_frame(framebuffer_needs_resize, window_backend);
    }

    fn prepare_render(&mut self, egui_gfx_output: egui_backend::EguiGfxOutput) {
        <WgpuBackend as GfxBackend<W>>::prepare_render(&mut self.wgpu_backend, egui_gfx_output);
    }

    fn render(&mut self) {
        <WgpuBackend as GfxBackend<W>>::render(&mut self.wgpu_backend);
    }

    fn present(&mut self, window_backend: &mut W) {
        self.wgpu_backend.present(window_backend);
    }
}
/*
prepare the mesh in advance with

intmap of user textures :)
*/
impl JokoRenderer {
    pub fn draw_marker(&mut self, marker: MarkerQuad) {
        self.markers.push(marker);
    }
    pub fn upload_texture(&mut self, texture_id: u32, width: u32, height: u32, pixels: Vec<u8>) {
        let dev = self.wgpu_backend.device.clone();
        let queue = self.wgpu_backend.queue.clone();
        // let mip_level_count = numLevels = 1 + floor(log2(max(w, h, d)))
        let mip_level_count = 1;
        let new_texture = dev.create_texture(&TextureDescriptor {
            label: None,
            size: Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: TextureFormat::Rgba8UnormSrgb,
            usage: TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST,
        });

        queue.write_texture(
            ImageCopyTexture {
                texture: &new_texture,
                mip_level: 0,
                origin: Origin3d::default(),
                aspect: TextureAspect::All,
            },
            &pixels,
            ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(
                    NonZeroU32::new(width * 4).expect("texture bytes per row is zero"),
                ),
                rows_per_image: Some(
                    NonZeroU32::new(height as u32).expect("texture rows count is zero"),
                ),
            },
            Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
        );
        let view = new_texture.create_view(&TextureViewDescriptor {
            label: None,
            format: Some(TextureFormat::Rgba8UnormSrgb),
            dimension: Some(TextureViewDimension::D2),
            aspect: TextureAspect::All,
            base_mip_level: 0,
            mip_level_count: None,
            base_array_layer: 0,
            array_layer_count: None,
        });
        let bindgroup = dev.create_bind_group(&BindGroupDescriptor {
            label: None,
            layout: &self.wgpu_backend.painter.texture_bindgroup_layout,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: BindingResource::Sampler(&self.wgpu_backend.painter.linear_sampler),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: BindingResource::TextureView(&view),
                },
            ],
        });
        self.textures.insert(texture_id as u64, bindgroup);
    }
}

#[derive(Debug, Default)]
pub struct MarkerQuad {
    pub position: Vec3,
    pub texture: u32,
    pub width: u16,
    pub height: u16,
}
