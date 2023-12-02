use std::collections::HashSet;

use anyhow::{anyhow, Ok, Result};
use log::{info, warn};
use vulkanalia::prelude::v1_0::*;
use vulkanalia::vk;

use crate::data::common_data::{self, CommonData};
use crate::data::globals;
use crate::utils::queue_family_indices::QueueFamilyIndices;

unsafe fn check_physical_device(
    instance: &Instance,
    common: &CommonData,
    physical_device: vk::PhysicalDevice,
) -> Result<()> {
    QueueFamilyIndices::get(instance, common, physical_device)?;
    check_physical_device_extensions(instance, physical_device)?;
    Ok(())
}

unsafe fn check_physical_device_extensions(
    instance: &Instance,
    physical_device: vk::PhysicalDevice,
) -> Result<()> {
    let extensions = instance
        .enumerate_device_extension_properties(physical_device, None)?
        .iter()
        .map(|e| e.extension_name)
        .collect::<HashSet<_>>();

    if globals::DEVICE_EXTENSIONS
        .iter()
        .all(|e| extensions.contains(e))
    {
        Ok(())
    } else {
        Err(anyhow!("Missing required device extensions."))
    }
}

pub unsafe fn pick_physical_device(instance: &Instance, common: &mut CommonData) -> Result<()> {
    for physical_device in instance.enumerate_physical_devices()? {
        let properties = instance.get_physical_device_properties(physical_device);

        if let Err(error) = check_physical_device(instance, common, physical_device) {
            warn!(
                "Skipping physical device: {}: {}",
                properties.device_name, error
            );
        } else {
            info!("Picked physical device: {}", properties.device_name);
            common.physical_device = physical_device;
            return Ok(());
        }
    }

    Err(anyhow!("No suitable devices found"))
}

pub unsafe fn create_logical_device(
    instance: &Instance,
    common: &mut CommonData,
) -> Result<Device> {
    let indices = QueueFamilyIndices::get(instance, common, common.physical_device)?;

    let mut unique_indices = HashSet::new();

    unique_indices.insert(indices.graphics);
    unique_indices.insert(indices.present);

    let queue_priorities = &[1.0];
    let queue_infos = unique_indices
        .iter()
        .map(|i| {
            vk::DeviceQueueCreateInfo::builder()
                .queue_family_index(*i)
                .queue_priorities(queue_priorities)
        })
        .collect::<Vec<_>>();

    let layers = if globals::VALIDATION_ENABLED {
        vec![globals::VALIDATION_LAYER.as_ptr()]
    } else {
        vec![]
    };

    let extensions = globals::DEVICE_EXTENSIONS
        .iter()
        .map(|n| n.as_ptr())
        .collect::<Vec<_>>();

    let features = vk::PhysicalDeviceFeatures::builder().fill_mode_non_solid(true);

    let info = vk::DeviceCreateInfo::builder()
        .queue_create_infos(&queue_infos)
        .enabled_layer_names(&layers)
        .enabled_extension_names(&extensions)
        .enabled_features(&features);

    let device = instance.create_device(common.physical_device, &info, None)?;

    common.present_queue = device.get_device_queue(indices.present, 0);
    common.graphics_queue = device.get_device_queue(indices.graphics, 0);

    Ok(device)
}
