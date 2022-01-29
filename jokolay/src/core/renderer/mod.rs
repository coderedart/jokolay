pub mod egui_state;
pub mod init;

use std::ffi::CStr;
use std::sync::Arc;

use anyhow::Context;
use erupt::vk::{DebugUtilsMessageSeverityFlagBitsEXT, QueueFlags};
use erupt_bootstrap::{DebugMessenger, DeviceBuilder, InstanceBuilder, ValidationLayers};

use tracing::{debug, error, info, warn};

use crate::core::renderer::egui_state::EguiState;
use crate::core::window::OverlayWindow;

use erupt::vk::DebugUtilsMessengerEXT;
use erupt::{vk, DeviceLoader, EntryLoader, InstanceLoader};
use erupt_bootstrap::{DeviceMetadata, InstanceMetadata};

pub struct Renderer {
    pub cmd_pool: vk::CommandPool,
    pub cmd_buffers: erupt::SmallVec<vk::CommandBuffer>,
    pub synctx: Vec<vk::Semaphore>,
    pub egui_state: egui_state::EguiState,
    pub stx: SurfaceCtx,
    pub vtx: Arc<VulkanCtx>,
}

#[derive(Debug, Clone)]
pub struct VulkanCtx {
    pub queue_family: u32,
    pub queue: Arc<vk::Queue>,
    pub device: Arc<DeviceLoader>,
    debug_messenger: Arc<Option<DebugUtilsMessengerEXT>>,
    pub instance: Arc<InstanceLoader>,
    _entry: Arc<EntryLoader>,
    _instance_metadata: Arc<InstanceMetadata>,
    pub device_metadata: Arc<DeviceMetadata>,
}
impl Drop for VulkanCtx {
    fn drop(&mut self) {
        unsafe {
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

pub struct SurfaceCtx {
    pub invalidate_count_frames_reset: usize,
    pub frame_buffers: Vec<(vk::ImageView, vk::Framebuffer)>,
    pub old_frame_buffers: Vec<(vk::ImageView, vk::Framebuffer)>,
    pub swapchain: erupt_bootstrap::Swapchain,
    pub surface: vk::SurfaceKHR,
    pub vtx: Arc<VulkanCtx>,
}
impl Drop for SurfaceCtx {
    fn drop(&mut self) {
        unsafe {
            for &(iv, fb) in &self.frame_buffers {
                self.vtx.device.destroy_image_view(iv, None);
                self.vtx.device.destroy_framebuffer(fb, None);
            }
            for &(iv, fb) in &self.old_frame_buffers {
                self.vtx.device.destroy_image_view(iv, None);
                self.vtx.device.destroy_framebuffer(fb, None);
            }
            self.swapchain.destroy(&self.vtx.device);
            self.vtx.instance.destroy_surface_khr(self.surface, None);
        }
    }
}
impl SurfaceCtx {
    pub fn tick(&mut self, render_pass: vk::RenderPass) -> anyhow::Result<erupt_bootstrap::AcquiredFrame> {
        unsafe {
            let current_frame = self
                .swapchain
                .acquire(&self.vtx.instance, &self.vtx.device, u64::MAX)
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
                    self.vtx.device.destroy_image_view(iv, None);
                    self.vtx.device.destroy_framebuffer(fb, None);
                }
                self.old_frame_buffers.clear();
            }
            assert!(!self.swapchain.images().is_empty());
            if self.frame_buffers.is_empty() {
                for &scimage in self.swapchain.images() {
                    let iv = self
                        .vtx
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
                    let fb = self
                        .vtx
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
                dbg!(self.swapchain.format(), self.swapchain.images().len(), self.swapchain.frames_in_flight());
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

            info!("destroyed vulkan instance");
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
                .composite_alpha(vk::CompositeAlphaFlagBitsKHR::PRE_MULTIPLIED_KHR)
                .format_preference(&[vk::SurfaceFormatKHRBuilder::new()
                    .format(vk::Format::B8G8R8A8_SRGB)
                    .color_space(vk::ColorSpaceKHR::SRGB_NONLINEAR_KHR)
                    .build()]);
            let swapchain = erupt_bootstrap::Swapchain::new(
                options,
                surface,
                device_metadata.physical_device(),
                &device,
                swapchain_image_extent,
            );
            let vtx = Arc::new(VulkanCtx {
                queue_family,
                queue: Arc::new(queue),
                _entry: entry,
                instance,
                _instance_metadata: Arc::new(instance_metadata),
                device,
                device_metadata: Arc::new(device_metadata),
                debug_messenger: Arc::new(debug_messenger),
            });

            let cmd_pool_create_info =
                vk::CommandPoolCreateInfoBuilder::new().queue_family_index(queue_family);
            let cmd_pool = vtx
                .device
                .create_command_pool(&cmd_pool_create_info, None)
                .result()?;

            let alloc_info = vk::CommandBufferAllocateInfoBuilder::new()
                .command_pool(cmd_pool)
                .command_buffer_count(swapchain.frames_in_flight() as u32);
            let cmd_buffers = vtx.device.allocate_command_buffers(&alloc_info).result()?;
            let mut synctx = vec![];
            for _ in  0..swapchain.frames_in_flight() {
                
                synctx.push(vtx.device.create_semaphore(&vk::SemaphoreCreateInfo::default(), None).result()?);
            }
            let stx = SurfaceCtx {
                invalidate_count_frames_reset: 0,

                frame_buffers: vec![],
                old_frame_buffers: vec![],
                swapchain,
                surface,
                vtx: vtx.clone(),
            };
            let egui_state = EguiState::new(vtx.clone(), &stx)?;

            
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
    pub fn tick(&mut self) -> anyhow::Result<()> {
        unsafe {
            let frame = self.stx.tick(self.egui_state.render_pass)?;
            let dev = self.vtx.device.clone();
            let cb = self.cmd_buffers[frame.frame_index];
            let fb = self.stx.frame_buffers[frame.image_index].1;

            dev.begin_command_buffer(cb, &vk::CommandBufferBeginInfoBuilder::new()).result()?;
            self.egui_state.tick(cb, fb, vk::Rect2D { offset: vk::Offset2D { x: 0, y: 0 }, extent: vk::Extent2D { width: 800, height: 600 }})?;
            dev.end_command_buffer(cb).result()?;
            dev.queue_submit(*self.vtx.queue, &[vk::SubmitInfoBuilder::new().command_buffers(&[cb])
            .wait_semaphores(&[frame.ready])
            .signal_semaphores(&[self.synctx[frame.frame_index]])], frame.complete).result()?;
            if let Err(e) = self.stx.swapchain.queue_present(&dev, *self.vtx.queue, self.synctx[frame.frame_index], frame.image_index).result() {
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
