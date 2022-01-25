use std::ffi::{c_void, CStr, CString};

use anyhow::Context as _;
use erupt::{cstr, EntryLoader, utils::*, vk};
use erupt::vk::DebugUtilsMessengerEXT;
use erupt_bootstrap::{DebugMessenger, DeviceBuilder, DeviceMetadata, InstanceBuilder, InstanceMetadata, QueueFamilyRequirements, ValidationLayers};
use glfw::Context;
use tracing::{debug, info, warn};

use crate::core::window::OverlayWindow;

pub struct Renderer {
    pub device: vk::Device,
    pub queue: vk::Queue,
    pub adapter: vk::PhysicalDevice,
    debug_messenger: Option<DebugUtilsMessengerEXT>,
    pub instance: erupt::vk::Instance,
    pub entry: erupt::EntryLoader,
    pub instance_metadata: InstanceMetadata,
    pub device_metadata: DeviceMetadata,

}

pub const ENGINE_NAME: &str = "Jokolay";

impl Renderer {
    pub unsafe fn initialize_vulkan(window: &OverlayWindow, validation: bool) -> anyhow::Result<Self> {
        info!("vulkan supported: {}", window.glfw.vulkan_supported());
       let entry= EntryLoader::new().context("failed to create EntryLoader for vulkan")?;
        // get all supported instance layers/ exts
        {
            let all_instance_exts = unsafe { entry.enumerate_instance_extension_properties(None, None) }?;
            let all_instance_layers = unsafe { entry.enumerate_instance_layer_properties(None) }?;
            info!("all instance extensions supported: {:#?}", &all_instance_exts);
            info!("all instance layers supported: {:#?}", &all_instance_exts);
        }

        let messenger = unsafe { instance.create_debug_utils_messenger_ext(&messenger_info, None) }.unwrap();

        let instance_builder = InstanceBuilder::new()
            .app_name(super::super::window::OverlayWindow::WINDOW_TITLE)?
            .app_version(0, 1)
            .engine_name("Jokolay Engine")?
            .engine_version(0, 1)
            .app_version(1, 2)
            .validation_layers(if validation {ValidationLayers::Require} else {ValidationLayers::Disable})
            .debug_message_severity(vk::DebugUtilsMessageSeverityFlagsEXT::VERBOSE_EXT
                | vk::DebugUtilsMessageSeverityFlagsEXT::WARNING_EXT
                | vk::DebugUtilsMessageSeverityFlagsEXT::ERROR_EXT)
            .debug_message_type(vk::DebugUtilsMessageTypeFlagsEXT::GENERAL_EXT
                | vk::DebugUtilsMessageTypeFlagsEXT::VALIDATION_EXT
                | vk::DebugUtilsMessageTypeFlagsEXT::PERFORMANCE_EXT)
            .request_debug_messenger(DebugMessenger::Custom { callback: debug_callback, user_data_pointer: std::ptr::null_mut() })
            .require_surface_extensions(&window.window)
            .context("failed to build instance_builder")?;

        let (instance, debug_messenger, instance_metadata) =
            unsafe { instance_builder.build(&entry) }.context("failed to create instance")?;

        info!("validation layers enabled? : {}", validation);
        info!("enabled extensions instance: {:#?}", instance_metadata.enabled_extensions());
        info!("enabled layers instance: {:#?}", instance_metadata.enabled_layers());

        let surface =
            unsafe { erupt::utils::surface::create_surface(&instance, &window.window, None) }.unwrap();

        let graphics_present = QueueFamilyRequirements::graphics_present();
        let transfer = QueueFamilyRequirements::preferably_separate_transfer();

        let device_features = vk::PhysicalDeviceFeatures2Builder::new().features(
            vk::PhysicalDeviceFeaturesBuilder::new()
                //.alpha_to_one(true)
                .build(),
        );

        let device_builder = DeviceBuilder::new()
            .require_queue_family(graphics_present)
            .require_queue_family(transfer)
            .require_features(&device_features)
            .for_surface(surface);
        let (device, device_metadata) =
            unsafe { device_builder.build(&instance, &instance_metadata) }.context("failed to build device")?;
        let graphics_present = device_metadata
            .device_queue(&instance, &device, graphics_present, 0)
            .context("failed to create graphics queue")?
            .context("failed ot unwrap option of graphics queue")?;
        let transfer = device_metadata
            .device_queue(&instance, &device, transfer, 0)
            .context("failed to create  transfer queue")?
            .context("failed ot unwrap option of  transfer queue")?;
        
            todo!()
    }
}

unsafe extern "system" fn debug_callback(
    _message_severity: vk::DebugUtilsMessageSeverityFlagBitsEXT,
    _message_types: vk::DebugUtilsMessageTypeFlagsEXT,
    p_callback_data: *const vk::DebugUtilsMessengerCallbackDataEXT,
    _p_user_data: *mut std::ffi::c_void,
) -> vk::Bool32 {
    debug!(
        "{:#?}",
        CStr::from_ptr((*p_callback_data).p_message)
    );

    vk::FALSE
}
