use joko_core::prelude::*;
use std::collections::BTreeMap;

use egui_render_wgpu::{wgpu::*, EguiTexture};
pub struct BillBoardRenderer {
    markers: Vec<MarkerQuad>,
    pipeline: RenderPipeline,
    camera_position: Vec3,
    // player_position: Vec3,
    vb: Buffer,
    vb_len: u64,
}

impl BillBoardRenderer {
    pub fn new(
        dev: &Device,
        transform_bgl: &BindGroupLayout,
        surface_format: TextureFormat,
    ) -> Self {
        let shader_module = egui_render_wgpu::wgpu::include_wgsl!("../shaders/marker.wgsl");
        let shader_module = dev.create_shader_module(shader_module);

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
                    format: surface_format,
                    blend: Some(PIPELINE_BLEND_STATE),
                    write_mask: ColorWrites::all(),
                })],
            }),
            multiview: None,
        });

        let vb = dev.create_buffer(&BufferDescriptor {
            label: Some("marker vertex buffer"),
            size: std::mem::size_of::<Vec4>() as u64 * 2 * 6,
            usage: BufferUsages::COPY_DST | BufferUsages::VERTEX,
            mapped_at_creation: false,
        });

        Self {
            markers: Vec::new(),
            pipeline,
            vb,
            vb_len: 0,
            camera_position: Default::default(),
            // player_position: Default::default(),
        }
    }
    pub fn prepare_render_data(
        &mut self,
        _encoder: &mut CommandEncoder,
        queue: &Queue,
        dev: &Device,
    ) {
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
        }
        queue.write_buffer(&self.vb, 0, bytemuck::cast_slice(&vb));
    }
    pub fn render<'a: 'b, 'b>(
        &'a self,
        rpass: &mut RenderPass<'b>,
        mvp_bg: &'a BindGroup,
        textures: &'a BTreeMap<u64, EguiTexture>,
    ) {
        rpass.set_pipeline(&self.pipeline);
        rpass.set_bind_group(0, mvp_bg, &[]);

        rpass.set_vertex_buffer(0, self.vb.slice(..));
        for (index, mq) in self.markers.iter().enumerate() {
            let index: u32 = index.try_into().unwrap();
            if let Some(texture) = textures.get(&(mq.texture as _)) {
                rpass.set_bind_group(1, &texture.bindgroup, &[]);
                rpass.draw((index * 6)..((index + 1) * 6), 0..1);
            }
        }
    }
}

pub const _BILLBOARD_MAX_VISIBILITY_DISTANCE: f32 = 10000.0;

#[derive(Debug, Default, Clone, Copy)]
pub struct MarkerQuad {
    pub position: Vec3,
    pub texture: u32,
    pub width: u16,
    pub height: u16,
}
impl MarkerQuad {
    pub fn get_vertices(self, camera_position: Vec3) -> [MarkerVertex; 6] {
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

const DEFAULT_QUAD: [Vec4; 4] = [
    // bottom left
    vec4(-50.0, -50.0, 0.0, 1.0),
    // top left
    vec4(-50.0, 50.0, 0.0, 1.0),
    // top right
    vec4(50.0, 50.0, 0.0, 1.0),
    // bottom right
    vec4(50.0, -50.0, 0.0, 1.0),
];

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
pub const PIPELINE_BLEND_STATE: BlendState = BlendState::ALPHA_BLENDING;

pub const PIPELINE_PRIMITIVE_STATE: PrimitiveState = PrimitiveState {
    topology: PrimitiveTopology::TriangleList,
    strip_index_format: None,
    front_face: FrontFace::Ccw,
    cull_mode: None,
    unclipped_depth: false,
    polygon_mode: PolygonMode::Fill,
    conservative: false,
};
