use std::sync::Arc;

use erupt::vk;
use tracing::info;

use crate::core::renderer::{SurfaceCtx, VulkanCtx};

pub struct EguiState {
    // pub pipeline: vk::Pipeline,
    pub vtx: Arc<VulkanCtx>,
    pub render_pass: vk::RenderPass,
}

impl Drop for EguiState {
    fn drop(&mut self) {
        unsafe {
            info!("destroying egui render pass");
            self.vtx.device.destroy_render_pass(self.render_pass, None);
        }
    }
}

impl EguiState {
    pub fn new(vtx: Arc<VulkanCtx>, stx: &SurfaceCtx) -> anyhow::Result<Self> {
        dbg!(stx.swapchain.format().format == vk::Format::B8G8R8A8_SRGB);
        let format = vk::Format::B8G8R8A8_SRGB;
        let _vertex_input = vk::PipelineVertexInputStateCreateInfoBuilder::new()
            .vertex_attribute_descriptions(&[])
            .vertex_binding_descriptions(&[]);
        let _input_assembly = vk::PipelineInputAssemblyStateCreateInfoBuilder::new()
            .primitive_restart_enable(false)
            .topology(vk::PrimitiveTopology::TRIANGLE_LIST);
        let viewport = vk::ViewportBuilder::new()
            .height(stx.swapchain.extent().height as std::os::raw::c_float)
            .width(stx.swapchain.extent().width as std::os::raw::c_float)
            .x(0.0)
            .y(0.0)
            .min_depth(0.0)
            .max_depth(1.0);
        let scissor = vk::Rect2DBuilder::new().extent(stx.swapchain.extent());
        let _pipeline_viewport = vk::PipelineViewportStateCreateInfoBuilder::new()
            .scissors(&[scissor])
            .viewports(&[viewport]);
        let _rasterizer = vk::PipelineRasterizationStateCreateInfoBuilder::new()
            .depth_clamp_enable(false)
            .rasterizer_discard_enable(false)
            .polygon_mode(vk::PolygonMode::FILL)
            .line_width(1.0)
            .cull_mode(vk::CullModeFlags::NONE)
            .front_face(vk::FrontFace::CLOCKWISE)
            .depth_bias_enable(false);
        let _multisample = vk::PipelineMultisampleStateCreateInfoBuilder::new()
            .sample_shading_enable(false)
            .rasterization_samples(vk::SampleCountFlagBits::_1);
        let color_blend_state = vk::PipelineColorBlendAttachmentStateBuilder::new()
            .color_write_mask(vk::ColorComponentFlags::all())
            .blend_enable(true)
            .src_alpha_blend_factor(vk::BlendFactor::ONE)
            .dst_alpha_blend_factor(vk::BlendFactor::ZERO)
            .src_color_blend_factor(vk::BlendFactor::SRC_ALPHA)
            .dst_color_blend_factor(vk::BlendFactor::ONE_MINUS_SRC_ALPHA)
            .alpha_blend_op(vk::BlendOp::ADD)
            .color_blend_op(vk::BlendOp::ADD);
        let _blend_state = vk::PipelineColorBlendStateCreateInfoBuilder::new()
            .logic_op_enable(true)
            .logic_op(vk::LogicOp::COPY)
            .attachments(&[color_blend_state]);
        let dynamic_states = [vk::DynamicState::VIEWPORT, vk::DynamicState::SCISSOR];
        let _dynamic_state =
            vk::PipelineDynamicStateCreateInfoBuilder::new().dynamic_states(&dynamic_states);
        let _pipeline_layout_create_info = vk::PipelineLayoutCreateInfoBuilder::new()
            .set_layouts(&[])
            .push_constant_ranges(&[]);
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

        Ok(Self { vtx, render_pass })
    }
    pub fn tick(
        &mut self,
        cb: vk::CommandBuffer,
        fb: vk::Framebuffer,
        render_area: vk::Rect2D,
    ) -> anyhow::Result<()> {
        unsafe {
            let mut clear_values = [vk::ClearValue::default()];
            clear_values[0].color = vk::ClearColorValue {
                float32: [0.0, 0.0, 0.0, 0.0],
            };
            let render_pass_begin_info = vk::RenderPassBeginInfoBuilder::new()
                .framebuffer(fb)
                .render_pass(self.render_pass)
                .clear_values(&clear_values)
                .render_area(render_area);
            self.vtx.device.cmd_begin_render_pass(
                cb,
                &render_pass_begin_info,
                vk::SubpassContents::INLINE,
            );
            self.vtx.device.cmd_end_render_pass(cb);
        }
        Ok(())
    }
}
