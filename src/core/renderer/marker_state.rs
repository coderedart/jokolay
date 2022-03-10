use crate::core::renderer::WgpuContext;
use crate::core::window::OverlayWindow;


use egui::{TextureId};
use std::collections::HashMap;

use wgpu::{
    include_wgsl, BindGroup, BindGroupLayout, BlendComponent,
    BlendFactor, BlendOperation, BlendState, Color, ColorTargetState, ColorWrites, CommandEncoder,
    FragmentState, FrontFace, LoadOp, Operations, PipelineLayout,
    PipelineLayoutDescriptor, PrimitiveState, PrimitiveTopology, RenderPassColorAttachment,
    RenderPassDescriptor, RenderPipeline, RenderPipelineDescriptor, ShaderModule,
    Texture, TextureView, VertexBufferLayout, VertexState, VertexStepMode,
};

pub struct MarkerState {
    pub pipeline: RenderPipeline,
    pub pipeline_layout: PipelineLayout,
    pub shader_module: ShaderModule,
}

impl MarkerState {
    pub fn new(
        wtx: &mut WgpuContext,
        texture_bindgroup_layout: &BindGroupLayout,
    ) -> color_eyre::Result<Self> {
        let pipeline_layout = wtx
            .device
            .create_pipeline_layout(&PipelineLayoutDescriptor {
                label: Some("egui pipeline layout"),
                bind_group_layouts: &[texture_bindgroup_layout],
                push_constant_ranges: &[],
            });
        let shader_module = wtx
            .device
            .create_shader_module(&include_wgsl!("shaders/marker.wgsl"));
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
            shader_module,
        })
    }

    #[allow(clippy::too_many_arguments)]
    pub fn tick(
        &mut self,
        fbv: &TextureView,
        encoder: &mut CommandEncoder,
        _window: &OverlayWindow,
        _wtx: &WgpuContext,
        _textures: &HashMap<TextureId, (Texture, TextureView, BindGroup)>,
    ) -> color_eyre::Result<()> {
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
        }
        Ok(())
    }
}
