use color_eyre::eyre::ContextCompat;
use std::collections::HashMap;

use egui::epaint::Vertex;
use egui::{ClippedPrimitive, TextureId};

use wgpu::util::{BufferInitDescriptor, DeviceExt};
use wgpu::{
    include_wgsl, BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout,
    BindGroupLayoutDescriptor, BindGroupLayoutEntry, BindingResource, BindingType, BlendComponent,
    BlendFactor, BlendOperation, BlendState, BufferAddress, BufferBinding, BufferBindingType,
    BufferDescriptor, BufferUsages, Color, ColorTargetState, ColorWrites, CommandEncoder,
    FragmentState, FrontFace, IndexFormat, LoadOp, Operations, PipelineLayoutDescriptor,
    PrimitiveState, PrimitiveTopology, RenderPassColorAttachment, RenderPassDescriptor,
    RenderPipelineDescriptor, ShaderModule, ShaderStages, Texture, TextureView, VertexBufferLayout,
    VertexState, VertexStepMode,
};

use crate::core::renderer::WgpuContext;
use crate::core::window::OverlayWindow;

pub struct EguiState {
    pub pipeline: wgpu::RenderPipeline,
    pub pipeline_layout: wgpu::PipelineLayout,
    pub bindgroup_layout: BindGroupLayout,
    pub shader_module: ShaderModule,
}

impl EguiState {
    pub fn new(
        wtx: &mut WgpuContext,
        texture_bindgroup_layout: &BindGroupLayout,
    ) -> color_eyre::Result<Self> {
        let bindgroup_layout = wtx
            .device
            .create_bind_group_layout(&BindGroupLayoutDescriptor {
                label: Some("egui bind group layout"),
                entries: &[BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::VERTEX,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
            });
        let pipeline_layout = wtx
            .device
            .create_pipeline_layout(&PipelineLayoutDescriptor {
                label: Some("egui pipeline layout"),
                bind_group_layouts: &[&bindgroup_layout, texture_bindgroup_layout],
                push_constant_ranges: &[],
            });
        let shader_module = wtx
            .device
            .create_shader_module(&include_wgsl!("shaders/egui.wgsl"));
        let attributes = wgpu::vertex_attr_array![
            0 => Float32x2,
            1 => Float32x2,
            2 => Unorm8x4,
        ];

        let pipeline = wtx
            .device
            .create_render_pipeline(&RenderPipelineDescriptor {
                label: Some("egui render pipeline"),
                layout: Some(&pipeline_layout),
                vertex: VertexState {
                    module: &shader_module,
                    entry_point: "vs_main",
                    buffers: &[VertexBufferLayout {
                        array_stride: 20,
                        step_mode: VertexStepMode::Vertex,
                        attributes: &attributes,
                    }],
                },
                primitive: PrimitiveState {
                    topology: PrimitiveTopology::TriangleList,
                    strip_index_format: None,
                    front_face: FrontFace::Ccw,
                    cull_mode: None,
                    unclipped_depth: false,
                    polygon_mode: Default::default(),
                    conservative: false,
                },
                depth_stencil: None,
                multisample: Default::default(),
                fragment: Some(FragmentState {
                    module: &shader_module,
                    entry_point: "fs_main",
                    targets: &[ColorTargetState {
                        format: wtx.config.format,
                        blend: Some(BlendState {
                            color: BlendComponent {
                                src_factor: BlendFactor::SrcAlpha,
                                dst_factor: BlendFactor::OneMinusSrcAlpha,
                                operation: BlendOperation::Add,
                            },
                            alpha: BlendComponent {
                                src_factor: BlendFactor::OneMinusDstAlpha,
                                dst_factor: BlendFactor::One,
                                operation: BlendOperation::Add,
                            },
                        }),
                        write_mask: ColorWrites::ALL,
                    }],
                }),
                multiview: None,
            });
        Ok(Self {
            pipeline,
            pipeline_layout,
            bindgroup_layout,
            shader_module,
        })
    }

    #[allow(clippy::too_many_arguments)]
    pub fn tick(
        &mut self,
        fbv: &TextureView,
        encoder: &mut CommandEncoder,
        window: &OverlayWindow,
        wtx: &WgpuContext,
        shapes: Vec<ClippedPrimitive>,
        _tex_update: bool,
        textures: &HashMap<TextureId, (Texture, TextureView, BindGroup)>,
    ) -> color_eyre::Result<()> {
        let size_in_points: [f32; 2] = [
            wtx.config.width as f32 / window.window_state.scale.x,
            wtx.config.height as f32 / window.window_state.scale.y,
        ];

        let ub = wtx.device.create_buffer_init(&BufferInitDescriptor {
            label: Some("egui uniform buffer"),
            contents: bytemuck::cast_slice(size_in_points.as_slice()),
            usage: BufferUsages::UNIFORM,
        });
        let ub_bindgroup = wtx.device.create_bind_group(&BindGroupDescriptor {
            label: Some("egui uniform bindgroup"),
            layout: &self.bindgroup_layout,
            entries: &[BindGroupEntry {
                binding: 0,
                resource: BindingResource::Buffer(BufferBinding {
                    buffer: &ub,
                    offset: 0,
                    size: None,
                }),
            }],
        });

        let vb_size: usize = shapes
            .iter()
            .map(|cm| {
                if let egui::epaint::Primitive::Mesh(m) = &cm.primitive {
                    m.vertices.len()
                } else {
                    0
                }
            })
            .sum::<usize>()
            * std::mem::size_of::<egui::epaint::Vertex>();
        let ib_size: usize = shapes
            .iter()
            .map(|cm| {
                if let egui::epaint::Primitive::Mesh(m) = &cm.primitive {
                    m.indices.len()
                } else {
                    0
                }
            })
            .sum::<usize>()
            * 4;
        let vb = wtx.device.create_buffer(&BufferDescriptor {
            label: Some("egui vertex buffer"),
            size: vb_size as BufferAddress,
            usage: BufferUsages::VERTEX,
            mapped_at_creation: true,
        });
        let ib = wtx.device.create_buffer(&BufferDescriptor {
            label: Some("egui index buffer"),
            size: (ib_size) as BufferAddress,
            usage: BufferUsages::INDEX,
            mapped_at_creation: true,
        });

        {
            let mut render_pass = encoder.begin_render_pass(&RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[RenderPassColorAttachment {
                    view: fbv,
                    resolve_target: None,
                    ops: Operations {
                        load: LoadOp::Clear(Color {
                            r: 0.0,
                            g: 0.0,
                            b: 0.0,
                            a: 0.0,
                        }),
                        store: true,
                    },
                }],
                depth_stencil_attachment: None,
            });
            render_pass.set_pipeline(&self.pipeline);
            render_pass.set_bind_group(0, &ub_bindgroup, &[]);
            render_pass.set_vertex_buffer(0, vb.slice(..));
            render_pass.set_index_buffer(ib.slice(..), IndexFormat::Uint32);
            {
                let mut vb_view = vb.slice(..).get_mapped_range_mut();
                let mut ib_view = ib.slice(..).get_mapped_range_mut();
                let mut vb_offset: usize = 0;
                let mut ib_offset: usize = 0;

                for primitive in shapes {
                    let ClippedPrimitive {
                        clip_rect,
                        primitive,
                    } = primitive;
                    if let egui::epaint::Primitive::Mesh(mesh) = primitive {
                        let vb_len = mesh.vertices.len() * 20;
                        let ib_len = mesh.indices.len() * std::mem::size_of::<u32>();
                        vb_view[vb_offset..(vb_offset + vb_len)].copy_from_slice(
                            bytemuck::cast_slice::<Vertex, u8>(mesh.vertices.as_slice()),
                        );
                        ib_view[ib_offset..(ib_offset + ib_len)]
                            .copy_from_slice(bytemuck::cast_slice(mesh.indices.as_slice()));
                        render_pass.set_bind_group(
                            1,
                            &textures
                                .get(&mesh.texture_id)
                                .wrap_err("texture not found")?
                                .2,
                            &[],
                        );

                        // Transform clip rect to physical pixels:
                        let pixels_per_point = window.window_state.scale.x;
                        let clip_min_x = pixels_per_point * clip_rect.min.x;
                        let clip_min_y = pixels_per_point * clip_rect.min.y;
                        let clip_max_x = pixels_per_point * clip_rect.max.x;
                        let clip_max_y = pixels_per_point * clip_rect.max.y;

                        // // Make sure clip rect can fit within a `u32`:
                        // let clip_min_x = clip_min_x.clamp(0.0, wtx.config.width as f32);
                        // let clip_min_y = clip_min_y.clamp(0.0, wtx.config.height as f32);
                        // let clip_max_x = clip_max_x.clamp(clip_min_x, wtx.config.width as f32);
                        // let clip_max_y = clip_max_y.clamp(clip_min_y, wtx.config.height as f32);

                        // let clip_min_x = clip_min_x.round() as i32;
                        // let clip_min_y = clip_min_y.round() as i32;
                        // let clip_max_x = clip_max_x.round() as i32;
                        // let clip_max_y = clip_max_y.round() as i32;
                        // wgpu cannot handle zero sized scissor rectangles, so this workaround is necessary
                        // https://github.com/gfx-rs/wgpu/issues/1750
                        if (clip_max_y - clip_min_y) >= 1.0 && (clip_max_x - clip_min_x) >= 1.0 {
                            render_pass.set_scissor_rect(
                                clip_min_x as u32,
                                (clip_min_y) as u32,
                                (clip_max_x - clip_min_x) as u32,
                                (clip_max_y - clip_min_y) as u32,
                            );

                            render_pass.draw_indexed(
                                ((ib_offset / 4) as u32)
                                    ..(((ib_offset / 4) + mesh.indices.len()) as u32),
                                (vb_offset / 20).try_into()?,
                                0..1,
                            );
                        }

                        vb_offset += vb_len;
                        ib_offset += ib_len;
                    }
                }
            }
            vb.unmap();
            ib.unmap();
        }
        Ok(())
    }
}
