use std::collections::HashMap;

use anyhow::Context;
use egui::{Color32, ImageData, TextureId};
use tracing::info;

use wgpu::{
    AddressMode, BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout,
    BindGroupLayoutDescriptor, BindGroupLayoutEntry, BindingResource, BindingType,
    CommandEncoderDescriptor, Device, Extent3d, Features, FilterMode, ImageCopyTexture,
    ImageDataLayout, Origin3d, PresentMode, Queue, Sampler, SamplerBindingType, SamplerDescriptor,
    ShaderStages, SurfaceConfiguration, SurfaceError, Texture, TextureAspect, TextureDescriptor,
    TextureDimension, TextureFormat, TextureSampleType, TextureUsages, TextureView,
    TextureViewDescriptor, TextureViewDimension,
};

use crate::core::renderer::egui_state::EguiState;
use crate::core::window::OverlayWindow;

mod egui_state;

pub struct Renderer {
    pub egui_state: EguiState,
    pub textures: HashMap<TextureId, (Texture, TextureView, BindGroup)>,
    pub egui_linear_bindgroup_layout: BindGroupLayout,
    pub egui_linear_sampler: Sampler,
    pub wtx: WgpuContext,
}

impl Renderer {
    pub async fn new(window: &OverlayWindow, _validation: bool) -> anyhow::Result<Self> {
        let mut wtx = WgpuContext::new(window).await?;
        let egui_linear_sampler = wtx.device.create_sampler(&SamplerDescriptor {
            label: Some("egui linear sampler"),
            address_mode_u: AddressMode::ClampToEdge,
            address_mode_v: AddressMode::ClampToEdge,
            address_mode_w: Default::default(),
            mag_filter: FilterMode::Linear,
            min_filter: FilterMode::Linear,
            mipmap_filter: FilterMode::Linear,
            lod_min_clamp: 0.0,
            lod_max_clamp: 0.0,
            compare: None,
            anisotropy_clamp: None,
            border_color: None,
        });
        let egui_linear_bindgroup_layout =
            wtx.device
                .create_bind_group_layout(&BindGroupLayoutDescriptor {
                    label: Some("egui linear bindgroup layout"),
                    entries: &[
                        BindGroupLayoutEntry {
                            binding: 0,
                            visibility: ShaderStages::FRAGMENT,
                            ty: BindingType::Sampler(SamplerBindingType::Filtering),
                            count: None,
                        },
                        BindGroupLayoutEntry {
                            binding: 1,
                            visibility: ShaderStages::FRAGMENT,
                            ty: BindingType::Texture {
                                sample_type: TextureSampleType::Float { filterable: true },
                                view_dimension: TextureViewDimension::D2,
                                multisampled: false,
                            },
                            count: None,
                        },
                    ],
                });
        let egui_state = EguiState::new(&mut wtx, &egui_linear_bindgroup_layout)?;

        Ok(Self {
            wtx,
            egui_state,
            egui_linear_sampler,
            egui_linear_bindgroup_layout,
            textures: HashMap::new(),
        })
    }

    pub fn tick(
        &mut self,
        textures_delta: egui::TexturesDelta,
        shapes: Vec<egui::ClippedMesh>,
        window: &OverlayWindow,
    ) -> anyhow::Result<()> {
        let tex_update = !textures_delta.set.is_empty() || !textures_delta.free.is_empty();

        for (id, delta) in textures_delta.set {
            let whole = delta.is_whole();
            let width = delta.image.width() as u32;
            let height = delta.image.height() as u32;
            let pixels: Vec<u8> = match delta.image {
                ImageData::Color(c) => c
                    .pixels
                    .into_iter()
                    .map(|c32| c32.to_array())
                    .flatten()
                    .collect(),
                ImageData::Alpha(a) => a
                    .pixels
                    .into_iter()
                    .map(|a8| Color32::from_white_alpha(a8).to_array())
                    .flatten()
                    .collect(),
            };
            let size = pixels.len() as u32;
            let position = [
                delta.pos.unwrap_or([0, 0])[0] as u32,
                delta.pos.unwrap_or([0, 0])[1] as u32,
            ];
            assert_eq!(size, width * height * 4);
            if whole {
                let format = TextureFormat::Rgba8UnormSrgb;
                let dimension = TextureDimension::D2;
                let mip_level_count = if id != TextureId::Managed(0) {
                    f32::floor(f32::log2(width.max(height) as f32)) as u32 + 1
                } else {
                    1
                };
                let new_texture = self.wtx.device.create_texture(&TextureDescriptor {
                    label: Some(&format!("{:#?}", id)),
                    size: Extent3d {
                        width: width as u32,
                        height: height as u32,
                        depth_or_array_layers: 1,
                    },
                    mip_level_count,
                    sample_count: 1,
                    dimension,
                    format,
                    usage: TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST,
                });
                let view = new_texture.create_view(&TextureViewDescriptor {
                    label: Some(&format!("view {:#?}", id)),
                    format: Some(format),
                    dimension: Some(TextureViewDimension::D2),
                    aspect: TextureAspect::All,
                    base_mip_level: 0,
                    mip_level_count: Some(mip_level_count.try_into()?),
                    base_array_layer: 0,
                    array_layer_count: Some(1.try_into()?),
                });
                let bindgroup = self.wtx.device.create_bind_group(&BindGroupDescriptor {
                    label: Some(&format!("bindgroup {:#?}", id)),
                    layout: &self.egui_linear_bindgroup_layout,
                    entries: &[
                        BindGroupEntry {
                            binding: 0,
                            resource: BindingResource::Sampler(&self.egui_linear_sampler),
                        },
                        BindGroupEntry {
                            binding: 1,
                            resource: BindingResource::TextureView(&view),
                        },
                    ],
                });
                self.textures.insert(id, (new_texture, view, bindgroup));
            }
            if let Some((tex, _view, _bindgroup)) = self.textures.get(&id) {
                self.wtx.queue.write_texture(
                    ImageCopyTexture {
                        texture: tex,
                        mip_level: 0,
                        origin: Origin3d {
                            x: position[0],
                            y: position[1],
                            z: 0,
                        },
                        aspect: TextureAspect::All,
                    },
                    pixels.as_slice(),
                    ImageDataLayout {
                        offset: 0,
                        bytes_per_row: Some((width as u32 * 4).try_into()?),
                        rows_per_image: None,
                    },
                    Extent3d {
                        width,
                        height,
                        depth_or_array_layers: 1,
                    },
                )
            }
        }
        // if we fail to get a framebuffer, we return. so, make sure to do any texture updates before this point
        match self.wtx.surface.get_current_texture() {
            Ok(fb) => {
                if fb.suboptimal {
                    dbg!("suboptimal");
                }
                if self.wtx.config.width != window.window_state.framebuffer_size.x
                    || self.wtx.config.height != window.window_state.framebuffer_size.y
                {
                    dbg!(&self.wtx.config, window.window_state.framebuffer_size);
                }
                let mut encoder =
                    self.wtx
                        .device
                        .create_command_encoder(&CommandEncoderDescriptor {
                            label: Some("Render encoder"),
                        });
                {
                    let fbv = fb.texture.create_view(&TextureViewDescriptor {
                        label: Some("frambuffer view"),
                        format: Option::from(self.wtx.config.format),
                        dimension: Some(TextureViewDimension::D2),
                        aspect: TextureAspect::All,
                        base_mip_level: 0,
                        mip_level_count: None,
                        base_array_layer: 0,
                        array_layer_count: None,
                    });

                    self.egui_state.tick(
                        &fbv,
                        &mut encoder,
                        window,
                        &self.wtx,
                        shapes,
                        tex_update,
                        &self.textures,
                    )?;
                }

                self.wtx.queue.submit(std::iter::once(encoder.finish()));
                fb.present();
            }
            Err(e) => match e {
                SurfaceError::Outdated => {
                    self.wtx
                        .surface
                        .configure(&self.wtx.device, &self.wtx.config);
                }
                rest => {
                    anyhow::bail!("surface error: {:#?}", rest);
                }
            },
        };
        for id in textures_delta.free {
            self.textures.remove(&id);
        }
        Ok(())
    }
}

pub struct WgpuContext {
    pub surface: wgpu::Surface,
    pub config: SurfaceConfiguration,
    pub queue: Queue,
    pub device: Device,
}

impl WgpuContext {
    pub async fn new(window: &OverlayWindow) -> anyhow::Result<Self> {
        let instance = wgpu::Instance::new(wgpu::Backends::PRIMARY);
        let surface = unsafe { instance.create_surface(&window.window) };
        info!("{:#?}", &surface);
        info!("list of GPUs and their features: ");
        for gpu in instance.enumerate_adapters(wgpu::Backends::all()) {
            info!(
                "adapter details:\n{:#?}\nadapter features:\n{:#?}",
                gpu.get_info(),
                gpu.features()
            );
        }
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                force_fallback_adapter: false,
                compatible_surface: Some(&surface),
            })
            .await
            .context("failed to create adapter")?;
        info!("chose adapter: {}", &adapter.get_info().name);
        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: "jokolay device".into(),
                    features: Features::MULTI_DRAW_INDIRECT
                        | Features::MULTIVIEW
                        | Features::MULTI_DRAW_INDIRECT_COUNT
                        | Features::TEXTURE_COMPRESSION_BC
                        | Features::INDIRECT_FIRST_INSTANCE
                        | Features::MULTIVIEW,
                    limits: Default::default(),
                },
                None,
            )
            .await
            .context("features not supported by this gpu")?;
        let config = SurfaceConfiguration {
            usage: TextureUsages::RENDER_ATTACHMENT,
            format: surface
                .get_preferred_format(&adapter)
                .context("surface has no preferred format")?,
            width: window.window_state.framebuffer_size.x,
            height: window.window_state.framebuffer_size.y,
            present_mode: PresentMode::Fifo,
        };
        info!("using surface configuration: {:#?}", &config);
        surface.configure(&device, &config);
        Ok(Self {
            surface,
            queue,
            device,
            config,
        })
    }
}
