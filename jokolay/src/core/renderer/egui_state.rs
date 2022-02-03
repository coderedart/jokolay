use std::ffi::CStr;
use std::sync::Arc;
use egui::{ClippedMesh, TextureId};

use erupt::{cstr, vk};
use erupt::vk::{BufferCreateInfo, BufferCreateInfoBuilder, Rect2DBuilder};
use parking_lot::RwLock;
use tracing::info;
use vk_mem_erupt::{Allocation, AllocationCreateFlags, AllocationCreateInfo, AllocationInfo, MemoryUsage};

use crate::core::renderer::{SurfaceCtx, VulkanCtx};

pub const VERT: &[u32] = vk_shader_macros::include_glsl!("shaders/egui.vert");
pub const FRAG: &[u32] = vk_shader_macros::include_glsl!("shaders/egui.frag");

pub struct EguiState {
    pub pipeline: vk::Pipeline,
    pub pipeline_layout: vk::PipelineLayout,
    pub render_pass: vk::RenderPass,
    pub pool: vk::DescriptorPool,
    pub layout: vk::DescriptorSetLayout,
    pub vb: Vec<(vk::Buffer, Allocation, AllocationInfo)>,
    pub ib: Vec<(vk::Buffer, Allocation, AllocationInfo)>,
    pub set: Vec<vk::DescriptorSet>,
}

impl EguiState {
    pub fn destroy(&mut self, vtx: &mut VulkanCtx ) {
        unsafe {
            info!("destroying egui render pass");
            for (b, a, ai) in &self.ib {
                vtx.allocator.destroy_buffer(*b, a);
            }
            for (b, a, ai) in &self.vb {
                vtx.allocator.destroy_buffer(*b, a);
            }
            vtx.device.destroy_descriptor_set_layout(self.layout, None);
            vtx.device.destroy_descriptor_pool(self.pool, None);
            vtx.device.destroy_render_pass(self.render_pass, None);
            vtx.device.destroy_pipeline(self.pipeline, None);
            vtx.device.destroy_pipeline_layout(self.pipeline_layout, None);

        }
    }

    pub unsafe fn new(vtx: &mut VulkanCtx, stx: &SurfaceCtx) -> anyhow::Result<Self> {
        let format = if stx.swapchain.format().format == vk::Format::UNDEFINED {
            vk::Format::B8G8R8A8_SRGB
        } else {
            stx.swapchain.format().format
        };
        let vs_cinfo = vk::ShaderModuleCreateInfoBuilder::new()
            .flags(vk::ShaderModuleCreateFlags::default())
            .code(VERT);
        let vs = unsafe { vtx.device.create_shader_module(&vs_cinfo, None).result()? };
        let fs_cinfo = vk::ShaderModuleCreateInfoBuilder::new().code(FRAG);
        let fs = unsafe { vtx.device.create_shader_module(&fs_cinfo, None).result()? };
        let pool = vtx.device.create_descriptor_pool(&vk::DescriptorPoolCreateInfoBuilder::new()
            .flags(vk::DescriptorPoolCreateFlags::UPDATE_AFTER_BIND)
            .max_sets(stx.swapchain.frames_in_flight() as u32)
            .pool_sizes(&[
                vk::DescriptorPoolSizeBuilder::new()
                    .descriptor_count(1)
                    ._type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
            ]), None).result()?;
        let layout = [vtx.device.create_descriptor_set_layout(&vk::DescriptorSetLayoutCreateInfoBuilder::new()
            .flags(vk::DescriptorSetLayoutCreateFlags::UPDATE_AFTER_BIND_POOL)
            .bindings(&[
                vk::DescriptorSetLayoutBindingBuilder::new()
                    .binding(0)
                    .descriptor_count(1)
                    .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
                    .stage_flags(vk::ShaderStageFlags::FRAGMENT)

            ]), None).result()?];
        let layout_vec: Vec<_> = (0..stx.swapchain.frames_in_flight()).map(|_| layout[0]).collect();
        let set = vtx.device.allocate_descriptor_sets(&vk::DescriptorSetAllocateInfoBuilder::new()
            .descriptor_pool(pool)
            .set_layouts(&layout_vec)

            ).result()?;
        let attr_desc = [
            vk::VertexInputAttributeDescriptionBuilder::new()
                .location(0)
                .binding(0)
                .offset(0)
                .format(vk::Format::R32G32_SFLOAT),
            vk::VertexInputAttributeDescriptionBuilder::new()
                .location(1)
                .binding(0)
                .offset(8)
                .format(vk::Format::R32G32_SFLOAT),
            vk::VertexInputAttributeDescriptionBuilder::new()
                .location(2)
                .binding(0)
                .offset(16)
                .format(vk::Format::R8G8B8A8_UNORM),
        ];
        let bind_desc = [vk::VertexInputBindingDescriptionBuilder::new()
            .binding(0)
            .input_rate(vk::VertexInputRate::VERTEX)
            .stride(std::mem::size_of::<egui::epaint::Vertex>() as u32)];
        let vertex_input = vk::PipelineVertexInputStateCreateInfoBuilder::new()
            .vertex_attribute_descriptions(&attr_desc)
            .vertex_binding_descriptions(&bind_desc);
        let input_assembly = vk::PipelineInputAssemblyStateCreateInfoBuilder::new()
            .primitive_restart_enable(false)
            .topology(vk::PrimitiveTopology::TRIANGLE_LIST);
        let viewport = [vk::ViewportBuilder::new()
            .height(stx.swapchain.extent().height as std::os::raw::c_float)
            .width(stx.swapchain.extent().width as std::os::raw::c_float)
            .x(0.0)
            .y(0.0)
            .min_depth(0.0)
            .max_depth(1.0)];
        let scissor = [vk::Rect2DBuilder::new()
            .offset(vk::Offset2DBuilder::new().x(0).y(0).build())
            .extent(stx.swapchain.extent())];
        let pipeline_viewport = vk::PipelineViewportStateCreateInfoBuilder::new()
            .scissors(&scissor)
            .viewports(&viewport);
        let rasterizer = vk::PipelineRasterizationStateCreateInfoBuilder::new()
            .depth_clamp_enable(false)
            .rasterizer_discard_enable(false)
            .polygon_mode(vk::PolygonMode::FILL)
            .line_width(1.0)
            .cull_mode(vk::CullModeFlags::NONE)
            .front_face(vk::FrontFace::CLOCKWISE)
            .depth_bias_enable(false);
        let multisample = vk::PipelineMultisampleStateCreateInfoBuilder::new()
            .sample_shading_enable(false)
            .rasterization_samples(vk::SampleCountFlagBits::_1);
        let color_blend_state = [vk::PipelineColorBlendAttachmentStateBuilder::new()
            .color_write_mask(vk::ColorComponentFlags::all())
            .blend_enable(true)
            .src_color_blend_factor(vk::BlendFactor::SRC_ALPHA)
            .dst_color_blend_factor(vk::BlendFactor::ONE_MINUS_SRC_ALPHA)
            .src_alpha_blend_factor(vk::BlendFactor::ONE_MINUS_DST_ALPHA)
            .dst_alpha_blend_factor(vk::BlendFactor::ONE)
            .alpha_blend_op(vk::BlendOp::ADD)
            .color_blend_op(vk::BlendOp::ADD)];
        let blend_state = vk::PipelineColorBlendStateCreateInfoBuilder::new()
            .logic_op_enable(false)
            .logic_op(vk::LogicOp::COPY)
            .attachments(&color_blend_state);
        let dynamic_states = [vk::DynamicState::VIEWPORT, vk::DynamicState::SCISSOR];
        let dynamic_state =
            vk::PipelineDynamicStateCreateInfoBuilder::new().dynamic_states(&dynamic_states);
        let push_const_build = [vk::PushConstantRangeBuilder::new()
            .offset(0)
            .size(8)
            .stage_flags(vk::ShaderStageFlags::VERTEX)];
        let pipeline_layout_create_info = vk::PipelineLayoutCreateInfoBuilder::new()
            .set_layouts(&
                layout
            )
            .push_constant_ranges(&push_const_build);
        let color_attachment = [vk::AttachmentDescriptionBuilder::new()
            .format(format)
            .samples(vk::SampleCountFlagBits::_1)
            .load_op(vk::AttachmentLoadOp::CLEAR)
            .store_op(vk::AttachmentStoreOp::STORE)
            .stencil_load_op(vk::AttachmentLoadOp::DONT_CARE)
            .stencil_store_op(vk::AttachmentStoreOp::DONT_CARE)
            .initial_layout(vk::ImageLayout::UNDEFINED)
            .final_layout(vk::ImageLayout::PRESENT_SRC_KHR)];
        let attachment_ref = [vk::AttachmentReferenceBuilder::new()
            .attachment(0)
            .layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL)];
        let subpass_desc = [vk::SubpassDescriptionBuilder::new()
            .pipeline_bind_point(vk::PipelineBindPoint::GRAPHICS)
            .color_attachments(&attachment_ref)];
        let render_pass_create_info = vk::RenderPassCreateInfoBuilder::new()
            .attachments(&color_attachment)
            .subpasses(&subpass_desc);
        let render_pass = unsafe {
            vtx.device
                .create_render_pass(&render_pass_create_info, None)
                .result()?
        };
        let pipeline_layout = unsafe {
            vtx.device
                .create_pipeline_layout(&pipeline_layout_create_info, None)
                .result()?
        };
        let pipeline = unsafe {
            vtx.device
                .create_graphics_pipelines(
                    Default::default(),
                    &[vk::GraphicsPipelineCreateInfoBuilder::new()
                        .render_pass(render_pass)
                        .layout(pipeline_layout)
                        .color_blend_state(&blend_state)
                        .dynamic_state(&dynamic_state)
                        .input_assembly_state(&input_assembly)
                        .multisample_state(&multisample)
                        .vertex_input_state(&vertex_input)
                        .rasterization_state(&rasterizer)
                        .viewport_state(&pipeline_viewport)
                        .stages(&[
                            vk::PipelineShaderStageCreateInfoBuilder::new()
                                .name(CStr::from_ptr(cstr!("main")))
                                .module(vs)
                                .stage(vk::ShaderStageFlagBits::VERTEX),
                            vk::PipelineShaderStageCreateInfoBuilder::new()
                                .name(CStr::from_ptr(cstr!("main")))
                                .module(fs)
                                .stage(vk::ShaderStageFlagBits::FRAGMENT),
                        ])],
                    None,
                )
                .result()?
        };
            vtx.device.destroy_shader_module(vs, None);
            vtx.device.destroy_shader_module(fs, None);
        let mut vb = vec![];
        let mut ib = vec![];
        for count in 0..vtx.frames_in_flight {
           vb.push(vtx.allocator.create_buffer(&BufferCreateInfoBuilder::new()
                .flags(vk::BufferCreateFlags::empty())
                .size(1024 * 1024 * 5)
                .sharing_mode(vk::SharingMode::EXCLUSIVE)
                .usage(vk::BufferUsageFlags::VERTEX_BUFFER )
                .queue_family_indices(&[vtx.queue_family])
                .build_dangling()
                                        ,
                                        &AllocationCreateInfo {
                                            usage: MemoryUsage::CpuToGpu,
                                            flags: AllocationCreateFlags::MAPPED,
                                            ..Default::default()
                                        }
            )?);
            ib.push(vtx.allocator.create_buffer(&BufferCreateInfoBuilder::new()
                .flags(vk::BufferCreateFlags::empty())
                .size(1024 * 1024 * 5)
                .sharing_mode(vk::SharingMode::EXCLUSIVE)
                .usage(vk::BufferUsageFlags::INDEX_BUFFER )
                .queue_family_indices(&[vtx.queue_family])
                .build_dangling()
                                                 ,
                                                 &AllocationCreateInfo {
                                                     usage: MemoryUsage::CpuToGpu,
                                                     flags: AllocationCreateFlags::MAPPED,
                                                     ..Default::default()
                                                 }
            )?);
        }

        Ok(Self {
            pipeline: pipeline[0],
            pipeline_layout,
            render_pass,
            layout: layout[0],
            pool,
            vb,
            ib,
            set: set.to_vec()
        })
    }
    pub fn tick(
        &mut self,
        vtx: &mut VulkanCtx,
        cb: vk::CommandBuffer,
        fb: vk::Framebuffer,
        render_area: vk::Rect2D,
        shapes: Vec<ClippedMesh>,
        tex_update: bool,
        pixels_per_point: f32
    ) -> anyhow::Result<()> {
        unsafe {

            let mut clear_values = [vk::ClearValue::default()];
            clear_values[0].color = vk::ClearColorValue {
                float32: [0.0, 0.0, 0.0, 0.0],
            };
            let (vb, vba, vbai) = &self.vb[vtx.frame_resource_index];
            let vb = *vb;
            let vb_ptr = vbai.get_mapped_data();
            let (ib, iba, ibai) = &self.ib[vtx.frame_resource_index];
            let ib = *ib;
            let ib_ptr = ibai.get_mapped_data();
            let set = self.set[vtx.frame_resource_index];
            let render_pass_begin_info = vk::RenderPassBeginInfoBuilder::new()
                .framebuffer(fb)
                .render_pass(self.render_pass)
                .clear_values(&clear_values)
                .render_area(render_area);
            let mut vb_offset = 0;
            let mut ib_offset = 0;
            vtx.device.cmd_begin_render_pass(
                cb,
                &render_pass_begin_info,
                vk::SubpassContents::INLINE,
            );
            vtx.device.cmd_bind_pipeline(cb, vk::PipelineBindPoint::GRAPHICS, self.pipeline);
            vtx.device.cmd_bind_vertex_buffers(cb, 0, &[vb], &[0]);
            vtx.device.cmd_bind_index_buffer(cb,  ib, 0, vk::IndexType::UINT32);
            let screen_size = [render_area.extent.width as f32 / pixels_per_point, render_area.extent.height as f32 / pixels_per_point];
            vtx.device.cmd_set_viewport(cb, 0, &[vk::ViewportBuilder::new()
                .x(render_area.offset.x as f32)
                .y(render_area.offset.y as f32)
                .width(render_area.extent.width as f32)
                .height(render_area.extent.height as f32)]);
            vtx.device.cmd_push_constants(cb, self.pipeline_layout, vk::ShaderStageFlags::VERTEX, 0, 8, screen_size.as_ptr() as * const std::ffi::c_void);

            if true {
                vtx.device.update_descriptor_sets(
                    &[
                        vk::WriteDescriptorSetBuilder::new()
                            .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
                            .dst_binding(0)
                            .dst_set(set)
                            .image_info(&[
                                vk::DescriptorImageInfoBuilder::new()
                                    .image_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
                                    .image_view(vtx.textures.get(&TextureId::Managed(0)).unwrap().0)
                                    .sampler(vtx.linear_sampler)
                            ])
                    ],
                    &[]
                );
            }
            vtx.device.cmd_bind_descriptor_sets(cb, vk::PipelineBindPoint::GRAPHICS
            , self.pipeline_layout, 0, &[set], &[]);
            for cmesh in &shapes {
                let vb_bytes_len = cmesh.1.vertices.len() * 20;
                let ib_bytes_len = cmesh.1.indices.len() * 4;
                std::ptr::copy_nonoverlapping(cmesh.1.vertices.as_ptr() as * const u8, (vb_ptr.add( vb_offset)), vb_bytes_len );
                std::ptr::copy_nonoverlapping(cmesh.1.indices.as_ptr() as * const u8, (ib_ptr.add( ib_offset)), ib_bytes_len );
                let clip_rect = cmesh.0;
                // Transform clip rect to physical pixels:
                let clip_min_x = pixels_per_point * clip_rect.min.x;
                let clip_min_y = pixels_per_point * clip_rect.min.y;
                let clip_max_x = pixels_per_point * clip_rect.max.x;
                let clip_max_y = pixels_per_point * clip_rect.max.y;
                let size_in_pixels = (render_area.extent.width, render_area.extent.height);


                    vtx.device.cmd_set_scissor(cb, 0, &[
                        vk::Rect2DBuilder::new()
                            .offset(vk::Offset2DBuilder::new().x(clip_min_x as i32).y((clip_min_y) as i32).build())
                            .extent(vk::Extent2DBuilder::new().width((clip_max_x - clip_min_x) as u32).height((clip_max_y - clip_min_y) as u32).build())
                    ]);
                vtx.device.cmd_draw_indexed(cb, cmesh.1.indices.len() as u32,
                                            1,
                                            ib_offset as u32 / 4,
                                            vb_offset as i32 / 20, 0);
                vb_offset += vb_bytes_len;
                ib_offset += ib_bytes_len;
            }
            vtx.device.cmd_end_render_pass(cb);
        }
        Ok(())
    }
}
