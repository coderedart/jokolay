mod billboard;
use billboard::BillBoardRenderer;
use bytemuck::cast_slice;
use egui_backend::{GfxBackend, WindowBackend};
use egui_render_wgpu::wgpu::*;
use egui_render_wgpu::EguiPainter;
use egui_render_wgpu::SurfaceManager;
use egui_render_wgpu::WgpuConfig;
use joko_core::prelude::*;
use jokolink::MumbleLink;
use std::num::NonZeroU64;

pub struct JokoRenderer {
    mvp_bg: BindGroup,
    mvp_ub: Buffer,
    player_visibility_pipeline: RenderPipeline,
    billboard_renderer: BillBoardRenderer,
    painter: EguiPainter,
    surface_manager: SurfaceManager,
    dev: Arc<Device>,
    queue: Arc<Queue>,
    adapter: Arc<Adapter>,
    instance: Arc<Instance>,
}

impl GfxBackend for JokoRenderer {
    type Configuration = WgpuConfig;

    fn new(window_backend: &mut impl WindowBackend, settings: Self::Configuration) -> Self {
        let WgpuConfig {
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
        #[cfg(not(target_arch = "wasm32"))]
        for adapter in instance.enumerate_adapters(Backends::all()) {
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

        let mvp_bgl = dev.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("marker transform matrix bindgroup layout"),
            entries: &TRANSFORM_MATRIX_UNIFORM_BINDGROUP_ENTRY,
        });
        let mvp_ub = dev.create_buffer(&BufferDescriptor {
            label: Some("mvp buffer"),
            size: 64,
            usage: BufferUsages::COPY_DST | BufferUsages::UNIFORM,
            mapped_at_creation: false,
        });
        let mvp_bg = dev.create_bind_group(&BindGroupDescriptor {
            label: Some("mvp bg"),
            layout: &mvp_bgl,
            entries: &[BindGroupEntry {
                binding: 0,
                resource: BindingResource::Buffer(mvp_ub.as_entire_buffer_binding()),
            }],
        });
        queue.write_buffer(
            &mvp_ub,
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
            BillBoardRenderer::new(&dev, &mvp_bgl, surface_manager.surface_config.format);
        Self {
            player_visibility_pipeline,
            mvp_bg,
            mvp_ub,
            surface_manager,
            dev,
            queue,
            adapter,
            instance,
            billboard_renderer,
            painter,
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
            self.surface_manager
                .surface_config
                .view_formats
                .first()
                .copied()
                .unwrap(),
        );
    }

    fn prepare_frame(&mut self, window_backend: &mut impl WindowBackend) {
        self.surface_manager
            .create_current_surface_texture_view(window_backend, &self.dev);
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
        self.billboard_renderer
            .prepare_render_data(&mut command_encoder, &self.queue, &self.dev);
        {
            let mut render_pass = command_encoder.begin_render_pass(&RenderPassDescriptor {
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
            self.billboard_renderer.render(
                &mut render_pass,
                &self.mvp_bg,
                &self.painter.managed_textures,
            );
            render_pass.set_pipeline(&self.player_visibility_pipeline);
            /*
            let point_on_screen = self.mvp.project_point3(self.player_position);
            let width = self.wgpu_backend.surface_manager.surface_config.width as f32;
            let height = self.wgpu_backend.surface_manager.surface_config.height as f32;
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
            // rpass.set_viewport(0.0, 0.0, 300.0, 300.0, 0.0, 1.0);
            rpass.set_vertex_buffer(0, self.viewport_buffer.slice(..));
            rpass.draw(0..6, 0..1);
             */
            self.painter
                .draw_egui_with_renderpass(&mut render_pass, draw_calls);
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
    pub fn update_from_mumble_link(&mut self, link: &MumbleLink) {
        let viewport_ratio = self.surface_manager.surface_config.width as f32
            / self.surface_manager.surface_config.height as f32;
        let center = link.f_camera_position + link.f_camera_front;
        let view_matrix = Mat4::look_at_lh(link.f_camera_position, center, vec3(0.0, 1.0, 0.0));

        let projection_matrix =
            Mat4::perspective_lh(link.identity.fov, viewport_ratio, 1.0, 10000.0);

        let view_projection_matrix = projection_matrix * view_matrix;
        self.queue.write_buffer(
            &self.mvp_ub,
            0,
            cast_slice(view_projection_matrix.as_ref().as_slice()),
        );
    }
}
pub const TRANSFORM_MATRIX_UNIFORM_BINDGROUP_ENTRY: [BindGroupLayoutEntry; 1] =
    [BindGroupLayoutEntry {
        binding: 0,
        visibility: ShaderStages::VERTEX,
        ty: BindingType::Buffer {
            ty: BufferBindingType::Uniform,
            has_dynamic_offset: false,
            min_binding_size: NonZeroU64::new(64),
        },
        count: None,
    }];
