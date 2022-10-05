use bytemuck::cast_slice;
use egui_backend::{GfxBackend, WindowBackend};

use egui_render_wgpu::wgpu::util::{BufferInitDescriptor, DeviceExt};
use egui_render_wgpu::wgpu::{
    self, AddressMode, BindGroupDescriptor, BindGroupEntry, BindGroupLayoutDescriptor,
    BindGroupLayoutEntry, BindingResource, BindingType, BlendComponent, BlendFactor,
    BlendOperation, BlendState, Buffer, BufferBindingType, BufferDescriptor, BufferUsages,
    ColorTargetState, ColorWrites, CommandEncoderDescriptor, Extent3d, FilterMode, FragmentState,
    FrontFace, ImageCopyTexture, ImageDataLayout, LoadOp, MultisampleState, Operations, Origin3d,
    PipelineLayoutDescriptor, PolygonMode, PrimitiveState, PrimitiveTopology,
    RenderPassColorAttachment, RenderPassDescriptor, RenderPipeline, RenderPipelineDescriptor,
    Sampler, SamplerBindingType, SamplerDescriptor, ShaderStages, TextureAspect, TextureDescriptor,
    TextureDimension, TextureFormat, TextureSampleType, TextureUsages, TextureViewDescriptor,
    TextureViewDimension, VertexBufferLayout, VertexState, VertexStepMode,
};
use egui_render_wgpu::{wgpu::BindGroup, WgpuBackend, WgpuSettings};
use glam::{vec2, vec3, vec4, Mat4, Quat, Vec2, Vec3, Vec4};
use intmap::IntMap;
use std::num::{NonZeroU32, NonZeroU64};

pub struct JokoRenderer {
    pub wgpu_backend: egui_render_wgpu::WgpuBackend,
    pub textures: intmap::IntMap<BindGroup>,
    pub markers: Vec<MarkerQuad>,
    pub pipeline: RenderPipeline,
    pub linear_sampler: Sampler,
    pub mvp_bg: BindGroup,
    pub mvp_ub: Buffer,
    pub camera_position: Vec3,
    pub player_position: Vec3,
    pub mvp: Mat4,
    pub vb: Buffer,
    pub vb_len: u64,
    pub blit_pipeline: RenderPipeline,
    pub player_visibility_pipeline: RenderPipeline,
    pub viewport_buffer: Buffer,
}

impl<W: WindowBackend> GfxBackend<W> for JokoRenderer {
    type Configuration = WgpuSettings;

    fn new(window_backend: &mut W, settings: Self::Configuration) -> Self {
        let wgpu_backend = WgpuBackend::new(window_backend, settings);
        let dev = wgpu_backend.device.clone();
        let queue = wgpu_backend.queue.clone();
        let shader_module = egui_render_wgpu::wgpu::include_wgsl!("./marker.wgsl");
        let shader_module = dev.create_shader_module(shader_module);
        let transform_bgl = dev.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("marker transform matrix bindgroup layout"),
            entries: &TRANSFORM_MATRIX_UNIFORM_BINDGROUP_ENTRY,
        });

        let texture_bgl = dev.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("marker texture bindgroup layout"),
            entries: &TEXTURE_BINDGROUP_ENTRIES,
        });
        let pipeline_layout = dev.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("marker pipeline"),
            bind_group_layouts: &[&transform_bgl, &texture_bgl],
            push_constant_ranges: &[],
        });
        let pipeline = dev.create_render_pipeline(&RenderPipelineDescriptor {
            label: Some("marker pipeline"),
            layout: Some(&pipeline_layout),
            vertex: VertexState {
                module: &shader_module,
                entry_point: "vs_main",
                buffers: &[ VertexBufferLayout {
                    array_stride: std::mem::size_of::<MarkerVertex>() as u64,
                    step_mode: VertexStepMode::Vertex,
                    attributes: &egui_render_wgpu::wgpu::vertex_attr_array![0 => Float32x4, 1 => Float32x2],
                } ],
            },
            primitive: PIPELINE_PRIMITIVE_STATE,
            depth_stencil: None,
            multisample: MultisampleState::default(),
            fragment: Some(FragmentState {
                module: &shader_module,
                entry_point: "fs_main",
                targets: &[Some(egui_render_wgpu::wgpu::ColorTargetState {
                    format: wgpu_backend.surface_config.format,
                    blend: Some(PIPELINE_BLEND_STATE),
                    write_mask: ColorWrites::all(),
                })],
            }),
            multiview: None,
        });
        let mvp_ub = dev.create_buffer(&BufferDescriptor {
            label: Some("mvp buffer"),
            size: 64,
            usage: BufferUsages::COPY_DST | BufferUsages::UNIFORM,
            mapped_at_creation: false,
        });
        let mvp_bg = dev.create_bind_group(&BindGroupDescriptor {
            label: Some("mvp bg"),
            layout: &transform_bgl,
            entries: &[BindGroupEntry {
                binding: 0,
                resource: BindingResource::Buffer(mvp_ub.as_entire_buffer_binding()),
            }],
        });
        queue.write_buffer(
            &mvp_ub,
            0,
            bytemuck::cast_slice(glam::Mat4::IDENTITY.as_ref().as_slice()),
        );
        let vb = dev.create_buffer(&BufferDescriptor {
            label: Some("marker vertex buffer"),
            size: std::mem::size_of::<Vec4>() as u64 * 2 * 6,
            usage: BufferUsages::COPY_DST | BufferUsages::VERTEX,
            mapped_at_creation: false,
        });
        let device = dev;
        let blit_shader =
            device.create_shader_module(egui_render_wgpu::wgpu::include_wgsl!("./blit.wgsl"));

        let blit_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("blit"),
            layout: None,
            vertex: wgpu::VertexState {
                module: &blit_shader,
                entry_point: "vs_main",
                buffers: &[],
            },
            fragment: Some(wgpu::FragmentState {
                module: &blit_shader,
                entry_point: "fs_main",
                targets: &[Some(ColorTargetState {
                    format: TextureFormat::Rgba8UnormSrgb,
                    blend: None,
                    write_mask: ColorWrites::default(),
                })],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                ..Default::default()
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
        });
        let player_visibility_pipeline_layout =
            device.create_pipeline_layout(&PipelineLayoutDescriptor {
                label: Some("player visibility layout"),
                bind_group_layouts: &[],
                push_constant_ranges: &[],
            });
        let player_visibility_shader_module = wgpu::include_wgsl!("./player_visibility.wgsl");
        match &player_visibility_shader_module.source {
            wgpu::ShaderSource::Wgsl(src) => {
                dbg!(src)
            }
            _ => todo!(),
        };
        let player_module = device.create_shader_module(player_visibility_shader_module);

        let player_visibility_pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
            label: Some("player visibility pipeline"),
            layout: Some(&player_visibility_pipeline_layout),
            vertex: VertexState {
                module: &player_module,
                entry_point: "vs_main",
                buffers: &[VertexBufferLayout {
                    array_stride: 8,
                    step_mode: VertexStepMode::Vertex,
                    attributes: &[egui_render_wgpu::wgpu::VertexAttribute {
                        format: wgpu::VertexFormat::Float32x2,
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
                    format: wgpu_backend.surface_config.format,
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
        // queue.write_buffer(
        //     &vb,
        //     0,
        //     bytemuck::cast_slice(&[
        //         MarkerVertex {
        //             position: glam::vec4(0.5, 0.5, 0.5, 1.0),
        //             texture_coordinates: vec2(0.0, 0.0),
        //             padding: Vec2::default(),
        //         },
        //         MarkerVertex {
        //             position: glam::vec4(-0.5, 0.5, 0.5, 1.0),
        //             texture_coordinates: vec2(1.0, 0.0),
        //             padding: Vec2::default(),
        //         },
        //         MarkerVertex {
        //             position: glam::vec4(0.0, 0.0, 0.5, 1.0),
        //             texture_coordinates: vec2(1.0, 1.0),
        //             padding: Vec2::default(),
        //         },
        //     ]),
        // );
        let linear_sampler = device.create_sampler(&LINEAR_SAMPLER_DESCRIPTOR);
        let viewport_buffer = device.create_buffer_init(&BufferInitDescriptor {
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
            wgpu_backend,
            textures: IntMap::new(),
            markers: Vec::new(),
            pipeline,
            mvp_ub,
            vb,
            mvp_bg,
            vb_len: 0,
            blit_pipeline,
            camera_position: Vec3::default(),
            linear_sampler,
            player_visibility_pipeline,
            player_position: Vec3::default(),
            mvp: Mat4::default(),
            viewport_buffer,
        }
    }

    fn prepare_frame(&mut self, framebuffer_needs_resize: bool, window_backend: &mut W) {
        self.wgpu_backend
            .prepare_frame(framebuffer_needs_resize, window_backend);
        self.markers.clear();
    }

    fn prepare_render(&mut self, egui_gfx_output: egui_backend::EguiGfxOutput) {
        <WgpuBackend as GfxBackend<W>>::prepare_render(&mut self.wgpu_backend, egui_gfx_output);
    }

    fn render(&mut self) {
        let dev = self.wgpu_backend.device.clone();
        let mut command_encoder = dev.create_command_encoder(&CommandEncoderDescriptor {
            label: Some("marker command encoder"),
        });
        let mut vb = vec![];
        vb.reserve(self.markers.len() * 6 * std::mem::size_of::<MarkerVertex>());
        for verts in self
            .markers
            .iter()
            .map(|mq| mq.get_vertices(self.camera_position))
        {
            vb.extend_from_slice(&verts);
        }
        let required_size_in_bytes = (vb.len() * std::mem::size_of::<MarkerVertex>()) as u64;
        if required_size_in_bytes > self.vb_len {
            self.vb = dev.create_buffer(&BufferDescriptor {
                label: Some("marker vertex buffer"),
                size: required_size_in_bytes,
                usage: BufferUsages::COPY_DST | BufferUsages::VERTEX,
                mapped_at_creation: false,
            });
            self.vb_len = required_size_in_bytes;
            tracing::info!("resizing buffer");
        }
        self.wgpu_backend
            .queue
            .write_buffer(&self.vb, 0, bytemuck::cast_slice(&vb));
        {
            let mut rpass = command_encoder.begin_render_pass(&RenderPassDescriptor {
                label: Some("marker render pass"),
                color_attachments: &[Some(RenderPassColorAttachment {
                    view: self.wgpu_backend.surface_view.as_ref().unwrap(),
                    resolve_target: None,
                    ops: Operations {
                        load: LoadOp::Load,
                        store: true,
                    },
                })],
                depth_stencil_attachment: None,
            });
            rpass.set_pipeline(&self.pipeline);
            rpass.set_bind_group(0, &self.mvp_bg, &[]);

            rpass.set_vertex_buffer(0, self.vb.slice(..));
            for (index, mq) in self.markers.iter().enumerate() {
                let index: u32 = index.try_into().unwrap();
                if let Some(texture) = self.textures.get(mq.texture as u64) {
                    rpass.set_bind_group(1, texture, &[]);
                    rpass.draw((index * 6)..((index + 1) * 6), 0..1);
                }
            }

            rpass.set_pipeline(&self.player_visibility_pipeline);
            let point_on_screen = self.mvp.project_point3(self.player_position);
            let width = self.wgpu_backend.surface_config.width as f32;
            let height = self.wgpu_backend.surface_config.height as f32;
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
        }
        self.wgpu_backend.command_encoders.push(command_encoder);

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
    pub fn set_mvp(&mut self, mat: Mat4) {
        self.wgpu_backend
            .queue
            .write_buffer(&self.mvp_ub, 0, cast_slice(mat.as_ref().as_slice()));
    }
    pub fn upload_texture(&mut self, texture_id: u32, width: u32, height: u32, pixels: Vec<u8>) {
        let dev = self.wgpu_backend.device.clone();
        let queue = self.wgpu_backend.queue.clone();
        let size = Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        };
        // let mip_level_count = numLevels = 1 + floor(log2(max(w, h, d)))
        let mip_level_count = size.max_mips(TextureDimension::D2);
        let new_texture = dev.create_texture(&TextureDescriptor {
            label: None,
            size,
            mip_level_count,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: TextureFormat::Rgba8UnormSrgb,
            usage: TextureUsages::TEXTURE_BINDING
                | TextureUsages::COPY_DST
                | TextureUsages::RENDER_ATTACHMENT,
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
            size,
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
                    resource: BindingResource::Sampler(&self.linear_sampler),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: BindingResource::TextureView(&view),
                },
            ],
        });
        let device = dev.clone();

        let views = (0..mip_level_count)
            .map(|mip| {
                new_texture.create_view(&wgpu::TextureViewDescriptor {
                    label: Some("mip"),
                    format: Some(TextureFormat::Rgba8UnormSrgb),
                    dimension: Some(TextureViewDimension::D2),
                    aspect: wgpu::TextureAspect::All,
                    base_mip_level: mip,
                    mip_level_count: NonZeroU32::new(1),
                    base_array_layer: 0,
                    array_layer_count: None,
                })
            })
            .collect::<Vec<_>>();
        if self.wgpu_backend.command_encoders.is_empty() {
            self.wgpu_backend
                .command_encoders
                .push(dev.create_command_encoder(&CommandEncoderDescriptor {
                    label: Some("blit encoder"),
                }));
        }
        for target_mip in 1..mip_level_count as usize {
            let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
                layout: &self.blit_pipeline.get_bind_group_layout(0),
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::Sampler(
                            &self.wgpu_backend.painter.linear_sampler,
                        ),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::TextureView(&views[target_mip - 1]),
                    },
                ],
                label: None,
            });
            let mut rpass = self
                .wgpu_backend
                .command_encoders
                .last_mut()
                .unwrap()
                .begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("blit render pass"),
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view: &views[target_mip],
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                            store: true,
                        },
                    })],
                    depth_stencil_attachment: None,
                });

            rpass.set_pipeline(&self.blit_pipeline);
            rpass.set_bind_group(0, &bind_group, &[]);
            rpass.draw(0..3, 0..1);
        }

        self.textures.insert(texture_id as u64, bindgroup);
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct MarkerQuad {
    pub position: Vec3,
    pub texture: u32,
    pub width: u16,
    pub height: u16,
}
impl MarkerQuad {
    fn get_vertices(self, camera_position: Vec3) -> [MarkerVertex; 6] {
        let MarkerQuad {
            position,
            texture: _,
            width,
            height,
        } = self;
        let mut billboard_direction = position - camera_position;
        billboard_direction.y = 0.0;
        let rotation = Quat::from_rotation_arc(Vec3::Z, billboard_direction.normalize());
        // let rotation = Quat::IDENTITY;
        let model_matrix = Mat4::from_scale_rotation_translation(
            vec3(width as f32 / 100.0, height as f32 / 100.0, 1.0),
            rotation,
            position,
        );
        let bottom_left = MarkerVertex {
            position: model_matrix * DEFAULT_QUAD[0],
            texture_coordinates: vec2(0.0, 1.0),
            padding: Vec2::default(),
        };

        let top_left = MarkerVertex {
            position: model_matrix * DEFAULT_QUAD[1],
            texture_coordinates: vec2(0.0, 0.0),
            padding: Vec2::default(),
        };
        let top_right = MarkerVertex {
            position: model_matrix * DEFAULT_QUAD[2],
            texture_coordinates: vec2(1.0, 0.0),
            padding: Vec2::default(),
        };
        let bottom_right = MarkerVertex {
            position: model_matrix * DEFAULT_QUAD[3],
            texture_coordinates: vec2(1.0, 1.0),
            padding: Vec2::default(),
        };
        [
            top_left,
            bottom_left,
            bottom_right,
            bottom_right,
            top_right,
            top_left,
        ]
    }
}
#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct MarkerVertex {
    pub position: Vec4,
    pub texture_coordinates: Vec2,
    pub padding: Vec2,
}

pub const DEFAULT_QUAD: [Vec4; 4] = [
    // bottom left
    vec4(-50.0, -50.0, 0.0, 1.0),
    // top left
    vec4(-50.0, 50.0, 0.0, 1.0),
    // top right
    vec4(50.0, 50.0, 0.0, 1.0),
    // bottom right
    vec4(50.0, -50.0, 0.0, 1.0),
];

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

pub const TEXTURE_BINDGROUP_ENTRIES: [BindGroupLayoutEntry; 2] = [
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
];
pub const PIPELINE_BLEND_STATE: BlendState = BlendState {
    color: BlendComponent {
        src_factor: BlendFactor::One,
        dst_factor: BlendFactor::OneMinusSrcAlpha,
        operation: BlendOperation::Add,
    },
    alpha: BlendComponent {
        src_factor: BlendFactor::OneMinusDstAlpha,
        dst_factor: BlendFactor::One,
        operation: BlendOperation::Add,
    },
};

pub const PIPELINE_PRIMITIVE_STATE: PrimitiveState = PrimitiveState {
    topology: PrimitiveTopology::TriangleList,
    strip_index_format: None,
    front_face: FrontFace::Ccw,
    cull_mode: None,
    unclipped_depth: false,
    polygon_mode: PolygonMode::Fill,
    conservative: false,
};
pub const LINEAR_SAMPLER_DESCRIPTOR: SamplerDescriptor = SamplerDescriptor {
    label: Some("linear sampler"),
    mag_filter: FilterMode::Linear,
    min_filter: FilterMode::Linear,
    mipmap_filter: FilterMode::Linear,
    address_mode_u: AddressMode::Repeat,
    address_mode_v: AddressMode::Repeat,
    address_mode_w: AddressMode::Repeat,
    lod_min_clamp: 0.0,
    lod_max_clamp: f32::MAX,
    compare: None,
    anisotropy_clamp: None,
    border_color: None,
};
