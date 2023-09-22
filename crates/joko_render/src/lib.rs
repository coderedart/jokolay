pub mod billboard;
use billboard::BillBoardRenderer;
use billboard::MarkerObject;
use billboard::TrailObject;
use egui_backend::{egui, GfxBackend, WindowBackend};
pub use egui_render_wgpu;
use egui_render_wgpu::wgpu::util::BufferInitDescriptor;
use egui_render_wgpu::wgpu::util::DeviceExt;
use egui_render_wgpu::wgpu::*;
use egui_render_wgpu::EguiPainter;
use egui_render_wgpu::SurfaceManager;
use glam::vec2;
use glam::Mat4;
use glam::Vec3;
use jokolink::MumbleLink;
use std::num::NonZeroU64;
use std::sync::Arc;
use tracing::debug;
use tracing::info;
pub struct JokoRenderer {
    marker_bg: BindGroup,
    marker_ub: Buffer,
    view_proj: Mat4,
    player_visibility_pipeline: RenderPipeline,
    viewport_buffer: Buffer,
    pub billboard_renderer: BillBoardRenderer,
    link: Option<Arc<MumbleLink>>,
    painter: EguiPainter,
    surface_manager: SurfaceManager,
    dev: Arc<Device>,
    queue: Arc<Queue>,
    adapter: Arc<Adapter>,
    instance: Arc<Instance>,
}

impl GfxBackend for JokoRenderer {
    type Configuration = egui_render_wgpu::WgpuConfig;

    fn new(window_backend: &mut impl WindowBackend, settings: Self::Configuration) -> Self {
        let egui_render_wgpu::WgpuConfig {
            power_preference,
            device_descriptor,
            surface_formats_priority,
            surface_config,
            backends,
        } = settings;
        debug!("using wgpu backends: {:?}", backends);
        let instance = Arc::new(Instance::new(InstanceDescriptor {
            backends,
            dx12_shader_compiler: Default::default(),
        }));
        debug!("iterating over all adapters");
        for adapter in instance.enumerate_adapters(backends) {
            debug!("adapter: {:#?}", adapter.get_info());
        }

        let surface = window_backend.get_window().map(|w| unsafe {
            use egui_backend::raw_window_handle::HasRawWindowHandle;
            debug!("creating a surface with {:?}", w.raw_window_handle());
            instance
                .create_surface(w)
                .expect("failed to create surface")
        });

        info!("is surfaced created at startup?: {}", surface.is_some());

        debug!("using power preference: {:?}", power_preference);
        let adapter = Arc::new(
            pollster::block_on(instance.request_adapter(&RequestAdapterOptions {
                power_preference,
                force_fallback_adapter: false,
                compatible_surface: surface.as_ref(),
            }))
            .expect("failed to get adapter"),
        );

        info!("chosen adapter details: {:?}", adapter.get_info());
        let (dev, queue) =
            pollster::block_on(adapter.request_device(&device_descriptor, Default::default()))
                .expect("failed to create wgpu device");

        let dev = Arc::new(dev);
        let queue = Arc::new(queue);

        let surface_manager = SurfaceManager::new(
            window_backend,
            &instance,
            &adapter,
            &dev,
            surface,
            surface_formats_priority,
            surface_config,
        );

        debug!("device features: {:#?}", dev.features());
        debug!("device limits: {:#?}", dev.limits());

        let painter = EguiPainter::new(&dev, surface_manager.surface_config.format);

        let marker_bgl = dev.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("marker uniform bindgroup layout"),
            entries: &MARKER_UNIFORM_BINDGROUP_ENTRY,
        });
        let marker_ub = dev.create_buffer(&BufferDescriptor {
            label: Some("marker buffer"),
            size: std::mem::size_of::<MarkerUniform>() as _,
            usage: BufferUsages::COPY_DST | BufferUsages::UNIFORM,
            mapped_at_creation: false,
        });
        let marker_bg = dev.create_bind_group(&BindGroupDescriptor {
            label: Some("marker bg"),
            layout: &marker_bgl,
            entries: &[BindGroupEntry {
                binding: 0,
                resource: BindingResource::Buffer(marker_ub.as_entire_buffer_binding()),
            }],
        });
        queue.write_buffer(
            &marker_ub,
            0,
            bytemuck::cast_slice(Mat4::IDENTITY.as_ref().as_slice()),
        );

        let player_visibility_pipeline_layout =
            dev.create_pipeline_layout(&PipelineLayoutDescriptor {
                label: Some("player visibility layout"),
                bind_group_layouts: &[],
                push_constant_ranges: &[],
            });
        let player_visibility_shader_module = include_wgsl!("../shaders/player_visibility.wgsl");

        let player_module = dev.create_shader_module(player_visibility_shader_module);

        let player_visibility_pipeline = dev.create_render_pipeline(&RenderPipelineDescriptor {
            label: Some("player visibility pipeline"),
            layout: Some(&player_visibility_pipeline_layout),
            vertex: VertexState {
                module: &player_module,
                entry_point: "vs_main",
                buffers: &[VertexBufferLayout {
                    array_stride: 8,
                    step_mode: VertexStepMode::Vertex,
                    attributes: &[egui_render_wgpu::wgpu::VertexAttribute {
                        format: VertexFormat::Float32x2,
                        offset: 0,
                        shader_location: 0,
                    }],
                }],
            },
            primitive: PrimitiveState {
                topology: PrimitiveTopology::TriangleList,
                ..Default::default()
            },
            depth_stencil: None,
            multisample: MultisampleState::default(),
            fragment: Some(FragmentState {
                module: &player_module,
                entry_point: "fs_main",
                targets: &[Some(ColorTargetState {
                    format: surface_manager.surface_config.format,
                    // blend: Some(PIPELINE_BLEND_STATE),
                    blend: Some(BlendState {
                        color: BlendComponent {
                            src_factor: BlendFactor::Zero,
                            dst_factor: BlendFactor::SrcAlpha,
                            operation: BlendOperation::Add,
                        },
                        alpha: BlendComponent {
                            src_factor: BlendFactor::One,
                            dst_factor: BlendFactor::One,
                            operation: BlendOperation::Min,
                        },
                    }),
                    write_mask: ColorWrites::default(),
                })],
            }),
            multiview: None,
        });
        let billboard_renderer =
            BillBoardRenderer::new(&dev, &marker_bgl, surface_manager.surface_config.format);
        let viewport_buffer = dev.create_buffer_init(&BufferInitDescriptor {
            label: Some("viewport quad buffer"),
            contents: bytemuck::cast_slice(&[
                vec2(-1.0, -1.0),
                vec2(-1.0, 1.0),
                vec2(1.0, 1.0),
                vec2(1.0, 1.0),
                vec2(1.0, -1.0),
                vec2(-1.0, -1.0),
            ]),
            usage: BufferUsages::VERTEX,
        });
        Self {
            player_visibility_pipeline,
            viewport_buffer,
            marker_bg,
            marker_ub,
            surface_manager,
            dev,
            queue,
            adapter,
            instance,
            billboard_renderer,
            painter,
            link: None,
            view_proj: Default::default(),
        }
    }

    fn resume(&mut self, window_backend: &mut impl WindowBackend) {
        self.surface_manager.reconfigure_surface(
            window_backend,
            &self.instance,
            &self.adapter,
            &self.dev,
        );
        self.painter.on_resume(
            &self.dev,
            self.surface_manager.surface_config.view_formats[0],
        );
    }

    fn prepare_frame(&mut self, window_backend: &mut impl WindowBackend) {
        self.surface_manager
            .create_current_surface_texture_view(window_backend, &self.dev);
        self.billboard_renderer.prepare_frame();
    }

    fn render_egui(
        &mut self,
        meshes: Vec<egui::ClippedPrimitive>,
        textures_delta: egui::TexturesDelta,
        logical_screen_size: [f32; 2],
    ) {
        let mut command_encoder = self.dev.create_command_encoder(&CommandEncoderDescriptor {
            label: Some("joko renderer"),
        });
        let draw_calls = self.painter.upload_egui_data(
            &self.dev,
            &self.queue,
            meshes,
            textures_delta,
            logical_screen_size,
            [
                self.surface_manager.surface_config.width,
                self.surface_manager.surface_config.height,
            ],
            &mut command_encoder,
        );
        if let Some(link) = self.link.as_ref() {
            self.billboard_renderer.prepare_render_data(
                link,
                &mut command_encoder,
                &self.queue,
                &self.dev,
            );
        }
        {
            let mut rpass = command_encoder.begin_render_pass(&RenderPassDescriptor {
                label: Some("joko render pass"),
                color_attachments: &[Some(RenderPassColorAttachment {
                    view: self
                        .surface_manager
                        .surface_view
                        .as_ref()
                        .expect("failed to get surface view for joko render pass creation"),
                    resolve_target: None,
                    ops: Operations {
                        load: LoadOp::Clear(Color::TRANSPARENT),
                        store: true,
                    },
                })],
                depth_stencil_attachment: None,
            });
            if let Some(link) = self.link.as_ref() {
                self.billboard_renderer.render(
                    &mut rpass,
                    &self.marker_bg,
                    &self.painter.managed_textures,
                );
                // clear any pixels that are right over player
                rpass.set_pipeline(&self.player_visibility_pipeline);
                let point_on_screen = self.view_proj.project_point3(link.player_pos);
                let width = self.surface_manager.surface_config.width as f32;
                let height = self.surface_manager.surface_config.height as f32;
                let x = point_on_screen.x * width / 2.0;
                let y = point_on_screen.y * height / 2.0;
                let x = width / 2.0 + x;
                let y = height / 2.0 - y;

                rpass.set_viewport(
                    f32::max(x - width * 0.1 / 2.0, 0.0),
                    f32::max(y - height * 0.2 / 2.0, 0.0),
                    width * 0.1,
                    height * 0.2,
                    0.0,
                    1.0,
                );
                rpass.set_vertex_buffer(0, self.viewport_buffer.slice(..));
                rpass.draw(0..6, 0..1);
                rpass.set_viewport(0.0, 0.0, width, height, 0.0, 1.0);
            }
            self.painter
                .draw_egui_with_renderpass(&mut rpass, draw_calls);
        }
        self.queue.submit(std::iter::once(command_encoder.finish()));
    }

    fn present(&mut self, _window_backend: &mut impl WindowBackend) {
        assert!(self.surface_manager.surface_view.is_some());

        {
            self.surface_manager
                .surface_view
                .take()
                .expect("failed to get surface view to present");
        }
        self.surface_manager
            .surface_current_image
            .take()
            .expect("failed to surface texture to preset")
            .present();
    }

    fn resize_framebuffer(&mut self, window_backend: &mut impl WindowBackend) {
        self.surface_manager
            .resize_framebuffer(&self.dev, window_backend);
    }

    fn suspend(&mut self, _window_backend: &mut impl WindowBackend) {
        self.surface_manager.suspend();
    }
}
/*
prepare the mesh in advance with

intmap of user textures :)
*/

impl JokoRenderer {
    pub fn tick(&mut self, link: Option<Arc<MumbleLink>>) {
        if let Some(link) = link.as_ref() {
            let viewport_ratio = self.surface_manager.surface_config.width as f32
                / self.surface_manager.surface_config.height as f32;
            let center = link.cam_pos + link.f_camera_front;
            let view_matrix = Mat4::look_at_lh(link.cam_pos, center, Vec3::Y);

            let projection_matrix = Mat4::perspective_lh(link.fov, viewport_ratio, 1.0, 1000.0);

            let view_proj = projection_matrix * view_matrix;
            let uniform_data = MarkerUniform {
                vp: view_proj,
                cam_pos: link.cam_pos,
                padding: 0.0,
            };
            self.queue
                .write_buffer(&self.marker_ub, 0, bytemuck::bytes_of(&uniform_data));
            self.view_proj = view_proj;
        }
        self.link = link;
    }
    pub fn add_billboard(&mut self, marker_object: MarkerObject) {
        self.billboard_renderer.markers.push(marker_object);
    }
    pub fn add_trail(&mut self, trail_object: TrailObject) {
        self.billboard_renderer.trails.push(trail_object);
    }
}
pub const MARKER_UNIFORM_BINDGROUP_ENTRY: [BindGroupLayoutEntry; 1] = [BindGroupLayoutEntry {
    binding: 0,
    visibility: ShaderStages::VERTEX,
    ty: BindingType::Buffer {
        ty: BufferBindingType::Uniform,
        has_dynamic_offset: false,
        min_binding_size: NonZeroU64::new(std::mem::size_of::<MarkerUniform>() as _),
    },
    count: None,
}];

#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
#[repr(C)]
struct MarkerUniform {
    vp: Mat4,
    cam_pos: Vec3,
    padding: f32,
}
