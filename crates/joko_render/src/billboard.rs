use std::collections::BTreeMap;

use egui_render_wgpu::{wgpu::*, EguiTexture};
use glam::{Vec2, Vec3, Vec4};
pub struct BillBoardRenderer {
    pub markers: Vec<MarkerObject>,
    pipeline: RenderPipeline,
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
                    attributes: &egui_render_wgpu::wgpu::vertex_attr_array![0 => Float32x3, 1 => Float32x2],
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
                    blend: Some(BlendState::ALPHA_BLENDING),
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
        }
    }
    pub fn prepare_render_data(
        &mut self,
        _link: &jokolink::MumbleLink,
        _encoder: &mut CommandEncoder,
        queue: &Queue,
        dev: &Device,
    ) {
        let mut vb = vec![];

        vb.reserve(self.markers.len() * 6 * std::mem::size_of::<MarkerVertex>());
        // sort by depth
        self.markers.sort_unstable_by(|first, second| {
            first.distance.total_cmp(&second.distance).reverse() // we need the farther markers (more distance from camera) to be rendered first, for correct alpha blending
        });
        for marker_object in self.markers.iter() {
            vb.extend_from_slice(&marker_object.vertices);
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
        for (index, mo) in self.markers.iter().enumerate() {
            let index: u32 = index.try_into().unwrap();
            if let Some(texture) = textures.get(&mo.texture) {
                rpass.set_bind_group(1, &texture.bindgroup, &[]);
                rpass.draw((index * 6)..((index + 1) * 6), 0..1);
            }
        }
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct MarkerVertex {
    pub position: Vec3,
    pub texture_coordinates: Vec2,
    pub padding: Vec2,
}

pub const TEXTURE_BINDGROUP_ENTRIES: [BindGroupLayoutEntry; 2] = [
    BindGroupLayoutEntry {
        binding: 0,
        visibility: ShaderStages::FRAGMENT,
        ty: BindingType::Texture {
            sample_type: TextureSampleType::Float { filterable: true },
            view_dimension: TextureViewDimension::D2,
            multisampled: false,
        },
        count: None,
    },
    BindGroupLayoutEntry {
        binding: 1,
        visibility: ShaderStages::FRAGMENT,
        ty: BindingType::Sampler(SamplerBindingType::Filtering),
        count: None,
    },
];

pub const PIPELINE_PRIMITIVE_STATE: PrimitiveState = PrimitiveState {
    topology: PrimitiveTopology::TriangleList,
    strip_index_format: None,
    front_face: FrontFace::Ccw,
    cull_mode: None,
    unclipped_depth: false,
    polygon_mode: PolygonMode::Fill,
    conservative: false,
};

pub struct MarkerObject {
    /// The six vertices that make up the marker quad
    pub vertices: [MarkerVertex; 6],
    /// The (managed) texture id from egui data
    pub texture: u64,
    /// The distance from camera
    /// As markers have transparency, we need to render them from far -> near order
    /// So, we will sort them using this distance just before rendering
    pub distance: f32,
}
