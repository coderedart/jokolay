use std::collections::{HashMap, VecDeque};
use std::ffi::CStr;
use std::sync::Arc;

use anyhow::Context;
use egui::{AlphaImage, ColorImage, ImageData, TextureId, TexturesDelta};
use erupt::{DeviceLoader, EntryLoader, InstanceLoader, vk};
use erupt::vk::{DebugUtilsMessageSeverityFlagBitsEXT, QueueFlags};
use erupt::vk::DebugUtilsMessengerEXT;
use erupt_bootstrap::{DebugMessenger, DeviceBuilder, InstanceBuilder, ValidationLayers};
use erupt_bootstrap::{DeviceMetadata, InstanceMetadata};
use parking_lot::RwLock;
use tracing::{debug, error, info, warn};
use vk_mem_erupt::{Allocation, AllocationCreateFlags, AllocationCreateInfo, AllocatorCreateFlags, MemoryUsage};

use crate::core::renderer::egui_state::EguiState;
use crate::core::window::OverlayWindow;

pub mod egui_state;
mod texture;

pub struct Renderer {
    pub cmd_pool: vk::CommandPool,
    pub cmd_buffers: erupt::SmallVec<vk::CommandBuffer>,
    pub synctx: Vec<vk::Semaphore>,
    pub egui_state: egui_state::EguiState,
    pub stx: SurfaceCtx,
    pub vtx: VulkanCtx,
}

pub struct VulkanCtx {
    pub textures: HashMap<TextureId, (vk::ImageView, vk::Image, Allocation)>,
    pub delete_queue: VecDeque<(
        Vec<(vk::ImageView, vk::Image, Allocation)>,
        Vec<(vk::Buffer, Allocation)>,
    )>,
    pub linear_sampler: vk::Sampler,
    pub allocator: vk_mem_erupt::Allocator,
    pub frame_resource_index: usize,
    pub present_frame: usize,
    pub frames_in_flight: usize,
    pub queue_family: u32,
    pub queue: Arc<vk::Queue>,
    pub device: Arc<DeviceLoader>,
    debug_messenger: Arc<Option<DebugUtilsMessengerEXT>>,
    pub instance: Arc<InstanceLoader>,
    _entry: Arc<EntryLoader>,
    _instance_metadata: Arc<InstanceMetadata>,
    pub device_metadata: Arc<DeviceMetadata>,
}

impl VulkanCtx {
    pub fn destroy(&mut self) {
        unsafe {
            for (v, i, a) in self.textures.values() {
                self.device.destroy_image_view(*v, None);
                self.allocator.destroy_image(*i, a);
            }
            for (iv, bv) in &self.delete_queue {
                for (v, i, a) in iv {
                    self.device.destroy_image_view(*v, None);
                    self.allocator.destroy_image(*i, a);
                }
                for (b, a) in bv {
                    self.allocator.destroy_buffer(*b, a);
                }
            }
            warn!("{:#?}", self.allocator.build_stats_string(false));
            self.allocator.destroy();
            self.device.destroy_sampler(self.linear_sampler, None);
            warn!("destroying device");
            self.device.destroy_device(None);
            if let Some(messenger) = *self.debug_messenger {
                self.instance
                    .destroy_debug_utils_messenger_ext(messenger, None);
            }
            warn!("destroying instance");
            self.instance.destroy_instance(None);
        }
    }
}
impl VulkanCtx {
    pub fn tick(&mut self) -> anyhow::Result<()> {
        unsafe {
            self.present_frame += 1;
            while self.delete_queue.len() > self.frames_in_flight {
                let (imgs, bufs) = self.delete_queue.pop_front().expect("could not pop front delete queue, despite length being greater than frames in flight");

                {
                    for (iv, i, a) in imgs {
                        self.device.destroy_image_view(iv, None);
                        self.allocator.destroy_image(i, &a);
                    }
                    for (b, a) in bufs {
                        // self.vtx.device.destroy_buffer(iv, None);
                        self.allocator.destroy_buffer(b, &a);
                    }
                }
            }
        }
            Ok(())
    }
}
pub struct SurfaceCtx {
    pub invalidate_count_frames_reset: usize,
    pub frame_buffers: Vec<(vk::ImageView, vk::Framebuffer)>,
    pub old_frame_buffers: Vec<(vk::ImageView, vk::Framebuffer)>,
    pub swapchain: erupt_bootstrap::Swapchain,
    pub surface: vk::SurfaceKHR,
}

impl  SurfaceCtx {
    pub fn destroy(&mut self, vtx: &mut VulkanCtx) {
        unsafe {
            for &(iv, fb) in &self.frame_buffers {
                vtx.device.destroy_image_view(iv, None);
                vtx.device.destroy_framebuffer(fb, None);
            }
            for &(iv, fb) in &self.old_frame_buffers {
                vtx.device.destroy_image_view(iv, None);
                vtx.device.destroy_framebuffer(fb, None);
            }
            self.swapchain.destroy(&vtx.device);
            vtx.instance.destroy_surface_khr(self.surface, None);
        }
    }
}

impl SurfaceCtx {
    pub fn tick(
        &mut self,
        vtx: &mut VulkanCtx,
        render_pass: vk::RenderPass,
    ) -> anyhow::Result<erupt_bootstrap::AcquiredFrame> {
        unsafe {
            let current_frame = self
                .swapchain
                .acquire(&vtx.instance, &vtx.device, u64::MAX)
                .result()?;
            if current_frame.invalidate_images {
                self.invalidate_count_frames_reset = 0;
                for &element in &self.frame_buffers {
                    self.old_frame_buffers.push(element);
                }
                self.frame_buffers.clear();
            }
            self.invalidate_count_frames_reset += 1;

            if !self.old_frame_buffers.is_empty()
                && self.invalidate_count_frames_reset > self.swapchain.frames_in_flight()
            {
                for &(iv, fb) in &self.old_frame_buffers {
                    vtx.device.destroy_image_view(iv, None);
                    vtx.device.destroy_framebuffer(fb, None);
                }
                self.old_frame_buffers.clear();
            }
            assert!(!self.swapchain.images().is_empty());
            if self.frame_buffers.is_empty() {
                for &scimage in self.swapchain.images() {
                    let iv = vtx

                        .device
                        .create_image_view(
                            &vk::ImageViewCreateInfoBuilder::new()
                                .format(self.swapchain.format().format)
                                .components(
                                    vk::ComponentMappingBuilder::new()
                                        .a(vk::ComponentSwizzle::IDENTITY)
                                        .r(vk::ComponentSwizzle::IDENTITY)
                                        .g(vk::ComponentSwizzle::IDENTITY)
                                        .b(vk::ComponentSwizzle::IDENTITY)
                                        .build(),
                                )
                                .image(scimage)
                                .view_type(vk::ImageViewType::_2D)
                                .subresource_range(
                                    vk::ImageSubresourceRangeBuilder::new()
                                        .aspect_mask(vk::ImageAspectFlags::COLOR)
                                        .base_mip_level(0)
                                        .base_array_layer(0)
                                        .layer_count(1)
                                        .level_count(1)
                                        .build(),
                                ),
                            None,
                        )
                        .result()?;
                    let fb = vtx
                        .device
                        .create_framebuffer(
                            &vk::FramebufferCreateInfoBuilder::new()
                                .render_pass(render_pass)
                                .attachments(&[iv])
                                .width(self.swapchain.extent().width)
                                .height(self.swapchain.extent().height)
                                .layers(1),
                            None,
                        )
                        .result()?;
                    self.frame_buffers.push((iv, fb));
                }
                dbg!(
                    self.swapchain.format(),
                    self.swapchain.images().len(),
                    self.swapchain.frames_in_flight()
                );
            }
            Ok(current_frame)
        }
    }
}

impl Drop for Renderer {
    fn drop(&mut self) {
        unsafe {
            self.vtx.device.device_wait_idle().unwrap();
            
            self.vtx
                .device
                .free_command_buffers(self.cmd_pool, &self.cmd_buffers);
            info!("freed cmd buffers");
            self.vtx.device.destroy_command_pool(self.cmd_pool, None);
            info!("deleted cmd pool");
            for &s in &self.synctx {
                self.vtx.device.destroy_semaphore(s, None);
            }
            self.egui_state.destroy(&mut self.vtx);
            self.stx.destroy(&mut self.vtx);
            self.vtx.destroy();
        }
    }
}

impl Renderer {
    pub fn initialize_vulkan(window: &OverlayWindow, validation: bool) -> anyhow::Result<Self> {
        unsafe {
            info!("vulkan supported: {}", window.glfw.vulkan_supported());
            // loader to get vulkan instance pointers
            let entry =
                Arc::new(EntryLoader::new().context("failed to create EntryLoader for vulkan")?);

            // app info. because why not
            let instance_builder = InstanceBuilder::new()
                .app_name(super::window::OverlayWindow::WINDOW_TITLE)?
                .app_version(0, 1)
                .engine_name("Jokolay Engine")?
                .engine_version(0, 1)
                .require_api_version(1, 2)
                .require_extension(erupt::extensions::khr_get_surface_capabilities2::KHR_GET_SURFACE_CAPABILITIES_2_EXTENSION_NAME)
                .require_extension(erupt::extensions::khr_get_physical_device_properties2::KHR_GET_PHYSICAL_DEVICE_PROPERTIES_2_EXTENSION_NAME);
            // if validation enabled, extra stuff
            let instance_builder = if validation {
                instance_builder
                    .validation_layers(if validation {
                        ValidationLayers::Require
                    } else {
                        ValidationLayers::Disable
                    })
                    .debug_message_severity(vk::DebugUtilsMessageSeverityFlagsEXT::all())
                    .debug_message_type(vk::DebugUtilsMessageTypeFlagsEXT::all())
                    .request_debug_messenger(DebugMessenger::Custom {
                        callback: debug_callback,
                        user_data_pointer: std::ptr::null_mut(),
                    })
            } else {
                instance_builder
            };

            let instance_builder = instance_builder
                .require_surface_extensions(&window.window)
                .context("failed to build instance_builder")?;

            // gets instance function pointers
            let (instance, debug_messenger, instance_metadata) = instance_builder
                .build(&entry)
                .context("failed to create instance")?;
            let instance = Arc::new(instance);
            info!("validation layers enabled? : {}", validation);
            info!("instance created: {:#?}", &instance_metadata);
            // create surface that needs to be presentable to by the device/queue and create swapchain for
            let surface =
                erupt::utils::surface::create_surface(&instance, &window.window, None).unwrap();

            let device_features = vk::PhysicalDeviceFeatures2Builder::new().features(
                vk::PhysicalDeviceFeaturesBuilder::new()
                    .multi_draw_indirect(true)
                    .logic_op(true)
                    .build(),
            );
            let queue_requirements = erupt_bootstrap::QueueFamilyCriteria::graphics_present()
                .must_support(QueueFlags::GRAPHICS | QueueFlags::TRANSFER);
            let device_builder = DeviceBuilder::new()
                .require_version(1, 2)
                .queue_family(queue_requirements)
                .require_features(&device_features)
                // .require_extension(erupt::extensions::khr_synchronization2::KHR_SYNCHRONIZATION_2_EXTENSION_NAME)
                // .require_extension(erupt::extensions::khr_timeline_semaphore::KHR_TIMELINE_SEMAPHORE_EXTENSION_NAME)
                .require_extension(erupt::extensions::khr_swapchain::KHR_SWAPCHAIN_EXTENSION_NAME)
                .for_surface(surface);
            // acquire device function pointers
            let (device, device_metadata) = device_builder
                .build(&instance, &instance_metadata)
                .context("failed to build device")?;
            let device = Arc::new(device);
            info!("created device: {:#?}", &device_metadata);
            // get queue
            let (queue, queue_family) = device_metadata
                .device_queue(&instance, &device, queue_requirements, 0)
                .context("failed to create graphics queue")?
                .context("failed ot unwrap option of graphics queue")?;
            // https://vulkan-tutorial.com/Drawing_a_triangle/Presentation/Swap_chain
            let surface_caps = instance
                .get_physical_device_surface_capabilities_khr(
                    device_metadata.physical_device(),
                    surface,
                )
                .unwrap();
            info!("surface capabilities: {:#?}", surface_caps);

            // https://www.khronos.org/registry/vulkan/specs/1.2-extensions/man/html/VkSurfaceCapabilitiesKHR.html#_description
            let swapchain_image_extent = match surface_caps.current_extent {
                vk::Extent2D {
                    width: u32::MAX,
                    height: u32::MAX,
                } => {
                    let (width, height) = window.window.get_framebuffer_size();
                    vk::Extent2D {
                        width: width as u32,
                        height: height as u32,
                    }
                }
                normal => normal,
            };

            let mut options = erupt_bootstrap::swapchain::SwapchainOptions::new();
            options
                // .composite_alpha(vk::CompositeAlphaFlagBitsKHR::PRE_MULTIPLIED_KHR)
                .format_preference(&[vk::SurfaceFormatKHRBuilder::new()
                    .format(vk::Format::B8G8R8A8_SRGB)
                    .color_space(vk::ColorSpaceKHR::SRGB_NONLINEAR_KHR)
                    .build()])
                // .present_mode_preference(&[vk::PresentModeKHR::IMMEDIATE_KHR])
            ;
            let swapchain = erupt_bootstrap::Swapchain::new(
                options,
                surface,
                device_metadata.physical_device(),
                &device,
                swapchain_image_extent,
            );

            let linear_sampler = device
                .create_sampler(&vk::SamplerCreateInfoBuilder::new()
                    .address_mode_u(vk::SamplerAddressMode::CLAMP_TO_EDGE)
                    .address_mode_v(vk::SamplerAddressMode::CLAMP_TO_EDGE)
                                , None)
                .result()?;
            let allocator = vk_mem_erupt::Allocator::new(&vk_mem_erupt::AllocatorCreateInfo {
                physical_device: device_metadata.physical_device(),
                device: device.clone(),
                instance: instance.clone(),
                flags: AllocatorCreateFlags::NONE,
                preferred_large_heap_block_size: 0,
                frame_in_use_count: swapchain.frames_in_flight() as u32,
                heap_size_limits: None,
            })?;
            let mut vtx =VulkanCtx {
                textures: HashMap::new(),
                delete_queue: VecDeque::new(),
                allocator,
                linear_sampler,
                frames_in_flight: swapchain.frames_in_flight(),
                present_frame: 0,
                frame_resource_index: 0,
                queue_family,
                queue: Arc::new(queue),
                _entry: entry,
                instance,
                _instance_metadata: Arc::new(instance_metadata),
                device,
                device_metadata: Arc::new(device_metadata),
                debug_messenger: Arc::new(debug_messenger),
            };

            let cmd_pool_create_info = vk::CommandPoolCreateInfoBuilder::new()
                .queue_family_index(queue_family)
                .flags(vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER);
            let cmd_pool = vtx
                .device
                .create_command_pool(&cmd_pool_create_info, None)
                .result()?;

            let alloc_info = vk::CommandBufferAllocateInfoBuilder::new()
                .command_pool(cmd_pool)
                .command_buffer_count(swapchain.frames_in_flight() as u32);
            let cmd_buffers = vtx.device.allocate_command_buffers(&alloc_info).result()?;
            let mut synctx = vec![];
            for _ in 0..swapchain.frames_in_flight() {
                synctx.push(
                    vtx.device
                        .create_semaphore(&vk::SemaphoreCreateInfo::default(), None)
                        .result()?,
                );
            }
            let stx = SurfaceCtx {
                invalidate_count_frames_reset: 0,

                frame_buffers: vec![],
                old_frame_buffers: vec![],
                swapchain,
                surface,
            };
            let egui_state = EguiState::new( &mut vtx, &stx)?;

            let renderer = Self {

                stx,
                egui_state,
                vtx,
                cmd_pool,
                cmd_buffers,
                synctx,
            };
            Ok(renderer)
        }
    }
    pub fn tick(&mut self, delta: TexturesDelta, shapes: Vec<egui::ClippedMesh>, window: &OverlayWindow) -> anyhow::Result<()> {
        unsafe {
            let frame = self.stx.tick(&mut self.vtx, self.egui_state.render_pass)?;
            let dev = self.vtx.device.clone();
            let cb = self.cmd_buffers[frame.frame_index];
            let fb = self.stx.frame_buffers[frame.image_index].1;
            self.vtx.delete_queue.push_back((vec![], vec![]));
            self.vtx.frame_resource_index = frame.frame_index;
            self.vtx.tick();
            dev.begin_command_buffer(cb, &vk::CommandBufferBeginInfoBuilder::new())
                .result()?;
            let mut tex_update = !delta.set.is_empty() || !delta.free.is_empty();
            for (id, dt) in delta.set {
                info!("{:#?}, {:#?}", &id, &dt.pos);
                if dt.pos.is_none() {
                    dbg!("resizing", &dt.image.width(), &dt.image.height());
                }
                let offset_x = dt.pos.unwrap_or_default()[0] as i32;
                let offset_y = dt.pos.unwrap_or_default()[1] as i32;
                let width = dt.image.width() as u32;
                let height = dt.image.height() as u32;
                let format = vk::Format::R8G8B8A8_SRGB;
                let whole = dt.is_whole();
                let pixels: Vec<u8> = match dt.image {
                    egui::ImageData::Color(c) => {
                        c.pixels.into_iter().map(|c32| {
                            c32.to_srgba_unmultiplied()
                        }).flatten().collect()
                    },
                    egui::ImageData::Alpha(a) => {
                        a.srgba_pixels(1.0).map(|c32| {
                            c32.to_srgba_unmultiplied()
                        }).flatten().collect()
                        // a.pixels.into_iter().map(|a8| {
                        //     egui::Color32::from_white_alpha(a8).to_array()
                        // }).flatten().collect()
                    }
                };
                // let cimg = egui::ColorImage::example();
                // // let cimg = egui::ColorImage::new([128, 128], egui::Color32::BLUE);
                // let pixels: Vec<u8> = cimg.pixels.iter().map(|&c32| {
                //     c32.to_srgba_unmultiplied()
                //     // [255, 234, 123, 255]
                // }).flatten().collect();
                // let width = cimg.width() as u32;
                // let height = cimg.height() as u32;
                assert_eq!(pixels.len(), (width * height * 4) as usize);
                let size = pixels.len();
                if whole {
                    // let img_create_info = ;
                    let alloc_info = AllocationCreateInfo {
                        usage: MemoryUsage::GpuOnly,
                        flags: AllocationCreateFlags::empty(),
                        required_flags: Default::default(),
                        preferred_flags: Default::default(),
                        memory_type_bits: 0,
                        pool: None,
                        user_data: None,
                    };
                    let (img, img_allocation, img_allocation_info) = self
                    .vtx
                        .allocator
                        .create_image(&*vk::ImageCreateInfoBuilder::new()
                            .format(format)
                            .extent(
                                vk::Extent3DBuilder::new()
                                    .width(width)
                                    .height(height)
                                    .depth(1)
                                    .build(),
                            )
                            .sharing_mode(vk::SharingMode::EXCLUSIVE)
                            .usage(vk::ImageUsageFlags::TRANSFER_DST | vk::ImageUsageFlags::SAMPLED)
                            .initial_layout(vk::ImageLayout::PREINITIALIZED)
                            .samples(vk::SampleCountFlagBits::_1)
                            .tiling(vk::ImageTiling::OPTIMAL)
                            .mip_levels(1)
                            .array_layers(1)
                            .queue_family_indices(&[self.vtx.queue_family])
                            .image_type(vk::ImageType::_2D), &alloc_info)?;

                    let view = self
                        .vtx
                        .device
                        .create_image_view(
                            &vk::ImageViewCreateInfoBuilder::new()
                                .format(format)
                                .image(img)
                                .subresource_range(
                                    vk::ImageSubresourceRangeBuilder::new()
                                        .level_count(1)
                                        .base_mip_level(0)
                                        .layer_count(1)
                                        .base_array_layer(0)
                                        .aspect_mask(vk::ImageAspectFlags::COLOR)
                                        .build(),
                                )
                                .view_type(vk::ImageViewType::_2D)
                                .components(vk::ComponentMappingBuilder::default().build()),
                            None,
                        )
                        .result()?;
                    if let Some(previous_texture) =
                    self.vtx.textures.insert(id, (view, img, img_allocation))
                    {
                        self.vtx.delete_queue.back_mut().unwrap().0.push(previous_texture);
                    }
                    self.vtx.device.cmd_pipeline_barrier(cb, vk::PipelineStageFlags::ALL_GRAPHICS,
                                                         vk::PipelineStageFlags::ALL_GRAPHICS, vk::DependencyFlags::default(), &[], &[], &[
                            vk::ImageMemoryBarrierBuilder::new()
                                .image(img)
                                .src_queue_family_index(self.vtx.queue_family)
                                .dst_queue_family_index(self.vtx.queue_family)
                                .old_layout(vk::ImageLayout::PREINITIALIZED)
                                .new_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
                                .src_access_mask(vk::AccessFlags::MEMORY_READ)
                                .dst_access_mask(vk::AccessFlags::MEMORY_WRITE)
                                .subresource_range(vk::ImageSubresourceRangeBuilder::new()
                                    .layer_count(1)
                                    .base_array_layer(0)
                                    .base_mip_level(0)
                                    .aspect_mask(vk::ImageAspectFlags::COLOR)
                                    .level_count(1)
                                    .build())
                        ]);
                }

                let (buffer, buffer_allocation, buffer_allocation_info) =
                    self.vtx.allocator.create_buffer(
                        &vk::BufferCreateInfoBuilder::new()
                            .usage(vk::BufferUsageFlags::TRANSFER_SRC)
                            .sharing_mode(vk::SharingMode::EXCLUSIVE)
                            .queue_family_indices(&[self.vtx.queue_family])
                            .size(size as u64),
                        &AllocationCreateInfo {
                            usage: MemoryUsage::CpuOnly,
                            flags: AllocationCreateFlags::MAPPED,
                            ..Default::default()
                        },
                    )?;
                dbg!(&buffer_allocation_info);
                let ptr = buffer_allocation_info.get_mapped_data();
                std::ptr::copy_nonoverlapping(pixels.as_ptr(), ptr, pixels.len());
                let tex = self.vtx.textures.get(&id).expect("failed to find texture in map").1;
                self.vtx.device.cmd_pipeline_barrier(cb, vk::PipelineStageFlags::ALL_GRAPHICS,
                                                     vk::PipelineStageFlags::ALL_GRAPHICS, vk::DependencyFlags::default(), &[], &[], &[
                    vk::ImageMemoryBarrierBuilder::new()
                        .image(tex)
                        .src_queue_family_index(self.vtx.queue_family)
                        .dst_queue_family_index(self.vtx.queue_family)
                        .old_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
                        .new_layout(vk::ImageLayout::TRANSFER_DST_OPTIMAL)
                        .src_access_mask(vk::AccessFlags::MEMORY_READ)
                        .dst_access_mask(vk::AccessFlags::MEMORY_WRITE)
                        .subresource_range(vk::ImageSubresourceRangeBuilder::new()
                            .layer_count(1)
                            .base_array_layer(0)
                            .base_mip_level(0)
                            .aspect_mask(vk::ImageAspectFlags::COLOR)
                            .level_count(1)
                            .build())
                ]);
                self.vtx.device.cmd_copy_buffer_to_image(cb, buffer, tex,
                vk::ImageLayout::TRANSFER_DST_OPTIMAL, &[vk::BufferImageCopyBuilder::new()
                        .buffer_image_height(height)
                        .buffer_offset(0)
                        .buffer_row_length(width)
                        .image_offset(vk::Offset3DBuilder::new().x(offset_x).y(offset_y).z(0).build())
                        .image_extent(vk::Extent3DBuilder::new()
                            .width(width)
                            .height(height)
                            .depth(1)
                            .build())
                        .image_subresource(vk::ImageSubresourceLayersBuilder::new()
                            .aspect_mask(vk::ImageAspectFlags::COLOR)
                            .base_array_layer(0)
                            .layer_count(1)
                            .mip_level(0)
                            .build())
                        ]);
                self.vtx.device.cmd_pipeline_barrier(cb, vk::PipelineStageFlags::ALL_GRAPHICS, vk::PipelineStageFlags::ALL_GRAPHICS, vk::DependencyFlags::default(), &[], &[], &[
                    vk::ImageMemoryBarrierBuilder::new()
                        .image(tex)
                        .src_queue_family_index(self.vtx.queue_family)
                        .dst_queue_family_index(self.vtx.queue_family)
                        .old_layout(vk::ImageLayout::TRANSFER_DST_OPTIMAL)
                        .new_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
                        .src_access_mask(vk::AccessFlags::MEMORY_WRITE)
                        .dst_access_mask(vk::AccessFlags::MEMORY_READ)
                        .subresource_range(vk::ImageSubresourceRangeBuilder::new()
                            .layer_count(1)
                            .base_array_layer(0)
                            .base_mip_level(0)
                            .aspect_mask(vk::ImageAspectFlags::COLOR)
                            .level_count(1)
                            .build())
                ]);

                self.vtx.delete_queue.back_mut().unwrap().1.push((buffer, buffer_allocation));
                dbg!(buffer_allocation);
            }
            self.egui_state.tick(
                &mut self.vtx,
                cb,
                fb,
                vk::Rect2DBuilder::new()
                    .offset(vk::Offset2DBuilder::new().x(0).y(0).build())
                    .extent(self.stx.swapchain.extent()).build(),
                shapes,
                tex_update,
                window.window_state.scale[0]
            )?;
            dev.end_command_buffer(cb).result()?;
            dev.queue_submit(
                *self.vtx.queue,
                &[vk::SubmitInfoBuilder::new()
                    .command_buffers(&[cb])
                    .wait_semaphores(&[frame.ready])
                    .wait_dst_stage_mask(&[vk::PipelineStageFlags::ALL_GRAPHICS])
                    .signal_semaphores(&[self.synctx[frame.frame_index]])],
                frame.complete,
            )
                .result()?;
            if let Err(e) = self
                .stx
                .swapchain
                .queue_present(
                    &dev,
                    *self.vtx.queue,
                    self.synctx[frame.frame_index],
                    frame.image_index,
                )
                .result()
            {
                dbg!(e, "queue present failed");
            }
            
        }
        Ok(())
    }
}

unsafe extern "system" fn debug_callback(
    message_severity: vk::DebugUtilsMessageSeverityFlagBitsEXT,
    _message_types: vk::DebugUtilsMessageTypeFlagsEXT,
    p_callback_data: *const vk::DebugUtilsMessengerCallbackDataEXT,
    _p_user_data: *mut std::ffi::c_void,
) -> vk::Bool32 {
    let message = CStr::from_ptr((*p_callback_data).p_message);
    match message_severity {
        DebugUtilsMessageSeverityFlagBitsEXT::WARNING_EXT => {
            warn!("{:#?}", message);
        }
        DebugUtilsMessageSeverityFlagBitsEXT::ERROR_EXT => {
            error!("{:#?}", message);
        }
        DebugUtilsMessageSeverityFlagBitsEXT::INFO_EXT => {
            info!("{:#?}", message);
        }
        DebugUtilsMessageSeverityFlagBitsEXT::VERBOSE_EXT => {
            debug!("{:#?}", message)
        }
        _ => error!("unknown severity flag bits: {:#?}", message),
    }

    vk::FALSE
}
