use std::ffi::{c_void, CStr, CString};
use std::sync::Arc;

use anyhow::Context;
use erupt::{cstr, DeviceLoader, EntryLoader, InstanceLoader, utils::*, vk};
use erupt::vk::{DebugUtilsMessageSeverityFlagBitsEXT, DebugUtilsMessageSeverityFlagsEXT, DebugUtilsMessengerEXT, QueueFlags};
use erupt_bootstrap::{DebugMessenger, DeviceBuilder, DeviceMetadata, InstanceBuilder, InstanceMetadata, QueueFamilyRequirements, ValidationLayers};
use glfw::Context as _;
use tracing::{debug, error, info, warn};

use crate::core::window::OverlayWindow;

pub struct Renderer {
    pub queue: vk::Queue,
    pub swapchain_image_views: Vec<vk::ImageView>,
    pub swapchain: vk::SwapchainKHR,
    pub surface: vk::SurfaceKHR,
    pub device: Arc<DeviceLoader>,
    debug_messenger: Option<DebugUtilsMessengerEXT>,
    pub instance: Arc<InstanceLoader>,
    pub entry: Arc<EntryLoader>,
    pub instance_metadata: InstanceMetadata,
    pub device_metadata: DeviceMetadata,

}

impl Drop for Renderer {
    fn drop(&mut self) {
        unsafe {
            for &iv in &self.swapchain_image_views {
                self.device.destroy_image_view(iv, None);
            }
            self.device.destroy_swapchain_khr(self.swapchain, None);
            self.instance.destroy_surface_khr(self.surface, None);
            self.device.destroy_device(None);
            if let Some(messenger) = self.debug_messenger {
                self.instance.destroy_debug_utils_messenger_ext(messenger, None);
            }
            self.instance.destroy_instance(None);
            info!("destroyed vulkan instance");
        }
    }
}

pub const ENGINE_NAME: &str = "Jokolay";

impl Renderer {
    pub unsafe fn initialize_vulkan(window: &OverlayWindow, validation: bool) -> anyhow::Result<Self> {
        info!("vulkan supported: {}", window.glfw.vulkan_supported());
        let entry = Arc::new(EntryLoader::new().context("failed to create EntryLoader for vulkan")?);



        let instance_builder = InstanceBuilder::new()
            .app_name(super::super::window::OverlayWindow::WINDOW_TITLE)?
            .app_version(0, 1)
            .engine_name("Jokolay Engine")?
            .engine_version(0, 1)
            .require_api_version(1, 2)
            .validation_layers(if validation { ValidationLayers::Require } else { ValidationLayers::Disable })
            .debug_message_severity(vk::DebugUtilsMessageSeverityFlagsEXT::all())
            .debug_message_type(vk::DebugUtilsMessageTypeFlagsEXT::all())
            .request_debug_messenger(DebugMessenger::Custom { callback: debug_callback, user_data_pointer: std::ptr::null_mut() })
            .require_extension(erupt::extensions::khr_get_surface_capabilities2::KHR_GET_SURFACE_CAPABILITIES_2_EXTENSION_NAME)
            .require_extension(erupt::extensions::khr_get_physical_device_properties2::KHR_GET_PHYSICAL_DEVICE_PROPERTIES_2_EXTENSION_NAME)
            .require_surface_extensions(&window.window)
            .context("failed to build instance_builder")?;

        let (instance, debug_messenger, instance_metadata) =
            unsafe { instance_builder.build(&entry) }.context("failed to create instance")?;
        let instance = Arc::new(instance);
        info!("validation layers enabled? : {}", validation);
        info!("instance created: {:#?}", &instance_metadata);

        let surface =
            unsafe { erupt::utils::surface::create_surface(&instance, &window.window, None) }.unwrap();


        let device_features = vk::PhysicalDeviceFeatures2Builder::new().features(
            vk::PhysicalDeviceFeaturesBuilder::new()
                .multi_draw_indirect(true)

                //.alpha_to_one(true)
                .build(),
        );
        let queue_requirements = QueueFamilyRequirements::graphics_present()
            .must_support(QueueFlags::GRAPHICS | QueueFlags::TRANSFER);
        let device_builder = DeviceBuilder::new()
            .require_version(1, 2)
            .require_queue_family(queue_requirements)
            .require_features(&device_features)
            .require_extension(erupt::extensions::khr_synchronization2::KHR_SYNCHRONIZATION_2_EXTENSION_NAME)
            .require_extension(erupt::extensions::khr_timeline_semaphore::KHR_TIMELINE_SEMAPHORE_EXTENSION_NAME)
            .require_extension(erupt::extensions::khr_swapchain::KHR_SWAPCHAIN_EXTENSION_NAME)
            .for_surface(surface);
        let (device, device_metadata) =
            unsafe { device_builder.build(&instance, &instance_metadata) }.context("failed to build device")?;
        let device = Arc::new(device);
        info!("created device: {:#?}", &device_metadata);
        info!("phy dev props 2: {:#?}", unsafe {instance.get_physical_device_properties2(device_metadata.physical_device(), None)});
        let graphics_present = device_metadata
            .device_queue(&instance, &device, queue_requirements, 0)
            .context("failed to create graphics queue")?
            .context("failed ot unwrap option of graphics queue")?;
        // https://vulkan-tutorial.com/Drawing_a_triangle/Presentation/Swap_chain
        let surface_caps =
            unsafe { instance.get_physical_device_surface_capabilities_khr(device_metadata.physical_device(), surface) }
                .unwrap();
        info!("surface capabilities: {:#?}", surface_caps);

        let mut image_count = surface_caps.min_image_count + 1;
        if surface_caps.max_image_count > 0 && image_count > surface_caps.max_image_count {
            image_count = surface_caps.max_image_count;
        }

        // https://www.khronos.org/registry/vulkan/specs/1.2-extensions/man/html/VkSurfaceCapabilitiesKHR.html#_description
        let swapchain_image_extent = match surface_caps.current_extent {
            vk::Extent2D {
                width: u32::MAX,
                height: u32::MAX,
            } => {
                let (width, height) = window.window.get_framebuffer_size();
                vk::Extent2D { width: width as u32, height: height as u32 }
            }
            normal => normal,
        };
        let formats = instance
            .get_physical_device_surface_formats_khr(device_metadata.physical_device(), surface, None)
            .unwrap();
        let format = match formats
            .iter()
            .find(|surface_format| {
                (surface_format.format == vk::Format::B8G8R8A8_SRGB
                    || surface_format.format == vk::Format::R8G8B8A8_SRGB)
                    && surface_format.color_space == vk::ColorSpaceKHR::SRGB_NONLINEAR_KHR
            })
        {
            Some(surface_format) => *surface_format,
            None => {
                error!("failed to get rgba8_srb format for surface. list of formats: {:#?}", &formats);
                anyhow::bail!("unable to get suitable format for surface");
            },
        };

        let swapchain_info = vk::SwapchainCreateInfoKHRBuilder::new()
            .surface(surface)
            .min_image_count(image_count)
            .image_format(format.format)
            .image_color_space(format.color_space)
            .image_extent(swapchain_image_extent)
            .image_array_layers(1)
            .image_usage(vk::ImageUsageFlags::COLOR_ATTACHMENT)
            .image_sharing_mode(vk::SharingMode::EXCLUSIVE)
            .pre_transform(surface_caps.current_transform)
            .composite_alpha(vk::CompositeAlphaFlagBitsKHR::OPAQUE_KHR)
            .present_mode(vk::PresentModeKHR::IMMEDIATE_KHR)
            .clipped(true)
            .old_swapchain(vk::SwapchainKHR::null());

        let swapchain = unsafe { device.create_swapchain_khr(&swapchain_info, None) }.unwrap();
        let swapchain_images = unsafe { device.get_swapchain_images_khr(swapchain, None) }.unwrap();

        // https://vulkan-tutorial.com/Drawing_a_triangle/Presentation/Image_views
        let swapchain_image_views: Vec<_> = swapchain_images
            .iter()
            .map(|swapchain_image| {
                let image_view_info = vk::ImageViewCreateInfoBuilder::new()
                    .image(*swapchain_image)
                    .view_type(vk::ImageViewType::_2D)
                    .format(format.format)
                    .components(vk::ComponentMapping {
                        r: vk::ComponentSwizzle::IDENTITY,
                        g: vk::ComponentSwizzle::IDENTITY,
                        b: vk::ComponentSwizzle::IDENTITY,
                        a: vk::ComponentSwizzle::IDENTITY,
                    })
                    .subresource_range(
                        vk::ImageSubresourceRangeBuilder::new()
                            .aspect_mask(vk::ImageAspectFlags::COLOR)
                            .base_mip_level(0)
                            .level_count(1)
                            .base_array_layer(0)
                            .layer_count(1)
                            .build(),
                    );
                unsafe { device.create_image_view(&image_view_info, None) }.unwrap()
            })
            .collect();
        let renderer = Self {
            device,
            queue: graphics_present.0,
            swapchain_image_views,
            swapchain,
            surface,
            debug_messenger,
            instance,
            entry,
            instance_metadata,
            device_metadata,
        };
        Ok(renderer)
    }
}

unsafe extern "system" fn debug_callback(
    message_severity: vk::DebugUtilsMessageSeverityFlagBitsEXT,
    message_types: vk::DebugUtilsMessageTypeFlagsEXT,
    p_callback_data: *const vk::DebugUtilsMessengerCallbackDataEXT,
    _p_user_data: *mut std::ffi::c_void,
) -> vk::Bool32 {
    let message =         CStr::from_ptr((*p_callback_data).p_message);
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
        _ => error!("unknown severity flag bits: {:#?}", message)
    }

    vk::FALSE
}
