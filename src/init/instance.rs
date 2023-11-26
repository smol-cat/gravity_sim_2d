use std::collections::HashSet;
use std::ffi::CStr;
use std::os::raw::c_void;

use anyhow::{anyhow, Ok, Result};
use log::{debug, error, trace, warn};
use vulkanalia::prelude::v1_0::*;
use vulkanalia::vk::ExtDebugUtilsExtension;
use vulkanalia::window as vk_window;
use winit::window::Window;

use crate::data::globals;

pub unsafe fn create_instance(
    entry: &Entry,
    window: &Window,
) -> Result<(Instance, vk::DebugUtilsMessengerEXT)> {
    let app_info = vk::ApplicationInfo::builder()
        .application_name(b"Sample\0")
        .application_version(vk::make_version(1, 0, 0))
        .engine_name(b"Vortex\0")
        .engine_version(vk::make_version(1, 0, 0))
        .api_version(vk::make_version(1, 0, 0));

    let available_layers: HashSet<vk::StringArray<256>> = entry
        .enumerate_instance_layer_properties()?
        .iter()
        .map(|l| l.layer_name)
        .collect::<HashSet<_>>();

    if globals::VALIDATION_ENABLED && !available_layers.contains(&globals::VALIDATION_LAYER) {
        return Err(anyhow!("Validation layer requested but not supported."));
    }

    let layers = if globals::VALIDATION_ENABLED {
        vec![globals::VALIDATION_LAYER.as_ptr()]
    } else {
        Vec::new()
    };

    let mut extensions = vk_window::get_required_instance_extensions(window)
        .iter()
        .map(|e| e.as_ptr())
        .collect::<Vec<_>>();

    if globals::VALIDATION_ENABLED {
        extensions.push(vk::EXT_DEBUG_UTILS_EXTENSION.name.as_ptr());
    }

    let flags = vk::InstanceCreateFlags::empty();

    let mut info: vk::InstanceCreateInfoBuilder = vk::InstanceCreateInfo::builder()
        .application_info(&app_info)
        .enabled_layer_names(&layers)
        .enabled_extension_names(&extensions)
        .flags(flags);

    let mut debug_info = vk::DebugUtilsMessengerCreateInfoEXT::builder()
        .message_severity(vk::DebugUtilsMessageSeverityFlagsEXT::all())
        .message_type(vk::DebugUtilsMessageTypeFlagsEXT::all())
        .user_callback(Some(debug_callback));

    if globals::VALIDATION_ENABLED {
        info = info.push_next(&mut debug_info);
    }

    let instance: Instance = entry.create_instance(&info, None)?;

    let mut messenger = vk::DebugUtilsMessengerEXT::default();
    if globals::VALIDATION_ENABLED {
        messenger = instance.create_debug_utils_messenger_ext(&debug_info, None)?;
    }

    Ok((instance, messenger))
}

extern "system" fn debug_callback(
    severity: vk::DebugUtilsMessageSeverityFlagsEXT,
    type_: vk::DebugUtilsMessageTypeFlagsEXT,
    data: *const vk::DebugUtilsMessengerCallbackDataEXT,
    _: *mut c_void,
) -> vk::Bool32 {
    let data = unsafe { *data };
    let message = unsafe { CStr::from_ptr(data.message) }.to_string_lossy();

    if severity >= vk::DebugUtilsMessageSeverityFlagsEXT::ERROR {
        error!("({:?}) {}", type_, message);
    } else if severity >= vk::DebugUtilsMessageSeverityFlagsEXT::WARNING {
        warn!("({:?}) {}", type_, message);
    } else if severity >= vk::DebugUtilsMessageSeverityFlagsEXT::INFO {
        debug!("({:?}) {}", type_, message);
    } else {
        trace!("({:?}) {}", type_, message);
    }

    vk::FALSE
}
