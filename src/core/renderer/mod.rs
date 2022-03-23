use std::collections::BTreeMap;
use std::path::PathBuf;
use std::sync::atomic::AtomicU64;
use std::sync::Arc;

use color_eyre::eyre::{ContextCompat, WrapErr};
use glm::U32Vec2;
use tokio::io::AsyncReadExt;
use tokio::task;
use tracing::{error, info, warn};

use crate::config::{JokoConfig, VsyncMode};
use wgpu::{
    AddressMode, BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout,
    BindGroupLayoutDescriptor, BindGroupLayoutEntry, BindingResource, BindingType, CommandEncoder,
    CommandEncoderDescriptor, Device, Extent3d, Features, FilterMode, Queue, Sampler,
    SamplerBindingType, SamplerDescriptor, ShaderStages, SurfaceConfiguration, SurfaceTexture,
    Texture, TextureAspect, TextureDescriptor, TextureDimension, TextureFormat, TextureSampleType,
    TextureUsages, TextureView, TextureViewDescriptor, TextureViewDimension,
};
use xxhash_rust::xxh3::xxh3_64;

use crate::core::window::OverlayWindow;
use crate::WgpuContext;

pub struct WgpuContextImpl {
    pub fb: Option<(SurfaceTexture, TextureView)>,
    pub ce: Option<CommandEncoder>,
    loaded_textures: BTreeMap<u64, LoadedTexture>,
    pub linear_bindgroup_layout: BindGroupLayout,
    pub linear_sampler: Sampler,
    pub surface: wgpu::Surface,
    pub config: SurfaceConfiguration,
    pub queue: Queue,
    pub device: Device,
}
impl WgpuContextImpl {
    pub fn init_framebuffer_view(&mut self, framebuffer_size: U32Vec2) {
        if self.config.width != framebuffer_size.x || self.config.height != framebuffer_size.y {
            self.config.width = framebuffer_size.x;
            self.config.height = framebuffer_size.y;
            self.surface.configure(&self.device, &self.config);
        }
        assert!(self.ce.is_none());
        self.ce = Some(
            self.device
                .create_command_encoder(&CommandEncoderDescriptor {
                    label: Some("primary command encoder"),
                }),
        );
        // if we fail to get a framebuffer, we return. so, make sure to do any texture updates before this point
        if let Ok(fb) = self.surface.get_current_texture() {
            if fb.suboptimal {
                warn!("suboptimal");
                self.surface.configure(&self.device, &self.config);
            }

            let fbv = fb.texture.create_view(&TextureViewDescriptor {
                label: Some("frambuffer view"),
                format: Option::from(self.config.format),
                dimension: Some(TextureViewDimension::D2),
                aspect: TextureAspect::All,
                base_mip_level: 0,
                mip_level_count: None,
                base_array_layer: 0,
                array_layer_count: None,
            });

            self.fb = Some((fb, fbv));
        };
    }
    pub fn present_framebuffer_view(&mut self) {
        self.queue.submit(std::iter::once(
            self.ce
                .take()
                .expect("command encoder is missing when trying to present framebuffer_view")
                .finish(),
        ));
        if let Some((fb, _fbv)) = self.fb.take() {
            fb.present();
        }
    }
}
pub fn load_texture_from_path(wtx: WgpuContext, image_path: PathBuf) -> LiveTextureHandle {
    let load_signaller = Arc::new(AtomicU64::new(0));
    let signaller = load_signaller.clone();

    let _ = task::spawn(async move {
        let _ = match tokio::fs::File::open(image_path.as_path()).await {
            Ok(mut f) => {
                let mut image_bytes = vec![];
                f.read_to_end(&mut image_bytes)
                    .await
                    .expect("failed to read image bytes");
                match image::load_from_memory(image_bytes.as_slice()) {
                    Ok(i) => {
                        let rgba8_image = i.to_rgba8();
                        let width = rgba8_image.width();
                        let height = rgba8_image.height();
                        let format = TextureFormat::Rgba8UnormSrgb;
                        let dimension = TextureDimension::D2;
                        let mip_level_count =
                            f32::floor(f32::log2(width.max(height) as f32)) as u32 + 1;
                        let hash_id = xxh3_64(rgba8_image.as_flat_samples().as_slice());
                        let mut wtx = wtx.write();
                        let new_texture = wtx.device.create_texture(&TextureDescriptor {
                            label: Some(&image_path.display().to_string()),
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
                            label: Some(&image_path.display().to_string()),
                            format: Some(format),
                            dimension: Some(TextureViewDimension::D2),
                            aspect: TextureAspect::All,
                            base_mip_level: 0,
                            mip_level_count: Some(
                                mip_level_count
                                    .try_into()
                                    .expect("mip level count in texture view nonzero"),
                            ),
                            base_array_layer: 0,
                            array_layer_count: Some(
                                1.try_into()
                                    .expect("array layer count in texture view nonzero"),
                            ),
                        });
                        let bindgroup = wtx.device.create_bind_group(&BindGroupDescriptor {
                            label: Some(&image_path.display().to_string()),
                            layout: &wtx.linear_bindgroup_layout,
                            entries: &[
                                BindGroupEntry {
                                    binding: 0,
                                    resource: BindingResource::Sampler(&wtx.linear_sampler),
                                },
                                BindGroupEntry {
                                    binding: 1,
                                    resource: BindingResource::TextureView(&view),
                                },
                            ],
                        });
                        let unix_time = time::OffsetDateTime::now_utc().unix_timestamp();
                        wtx.loaded_textures.insert(
                            hash_id,
                            LoadedTexture {
                                id: signaller,
                                still_being_used: AtomicU64::new(
                                    unix_time
                                        .try_into()
                                        .expect("failed to put unix timestamp into u64"),
                                ),
                                native_texture: NativeTexture {
                                    texture: new_texture,
                                    texture_view: view,
                                    bindgroup,
                                },
                            },
                        );
                    }
                    Err(e) => {
                        error!("failed to decode image {image_path:?} due to error {e}");
                    }
                }
            }
            Err(e) => {
                error!("failed to load image {image_path:?} file due to error {e}")
            }
        };
    });
    LiveTextureHandle { id: load_signaller }
}

#[derive(Clone)]
pub struct LiveTextureHandle {
    id: Arc<AtomicU64>,
}

pub struct NativeTexture {
    texture: Texture,
    texture_view: TextureView,
    bindgroup: BindGroup,
}
pub struct LoadedTexture {
    id: Arc<AtomicU64>,
    still_being_used: AtomicU64,
    native_texture: NativeTexture,
}

pub type TexHandle = Arc<u64>;

impl WgpuContextImpl {
    pub async fn new(window: &OverlayWindow, config: &JokoConfig) -> color_eyre::Result<Self> {
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
            .wrap_err("failed to create adapter")?;
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
            .wrap_err("features not supported by this gpu")?;
        let config = SurfaceConfiguration {
            usage: TextureUsages::RENDER_ATTACHMENT,
            format: surface
                .get_preferred_format(&adapter)
                .wrap_err("surface has no preferred format")?,
            width: window.window_state.read().framebuffer_size.x,
            height: window.window_state.read().framebuffer_size.y,
            present_mode: match config.overlay_window_config.vsync {
                VsyncMode::Immediate => wgpu::PresentMode::Immediate,
                VsyncMode::Fifo => wgpu::PresentMode::Fifo,
            },
        };
        info!("using surface configuration: {:#?}", &config);
        surface.configure(&device, &config);

        let linear_sampler = device.create_sampler(&SamplerDescriptor {
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
        let linear_bindgroup_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
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

        Ok(Self {
            fb: None,
            ce: None,
            loaded_textures: Default::default(),
            linear_bindgroup_layout,
            linear_sampler,
            surface,
            queue,
            device,
            config,
        })
    }
}
