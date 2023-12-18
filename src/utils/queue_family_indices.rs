use anyhow::{anyhow, Result};
use vulkanalia::prelude::v1_0::*;
use vulkanalia::vk;
use vulkanalia::vk::{InstanceV1_0, KhrSurfaceExtension};

use crate::data::common_data::CommonData;

#[derive(Copy, Clone, Debug)]
pub struct QueueFamilyIndices {
    pub graphics_compute: u32,
    pub present: u32,
}

impl QueueFamilyIndices {
    pub unsafe fn get(
        instance: &Instance,
        common: &CommonData,
        physical_device: vk::PhysicalDevice,
    ) -> Result<Self> {
        let properties = instance.get_physical_device_queue_family_properties(physical_device);

        let graphics: Option<u32> = properties
            .iter()
            .position(|p| {
                p.queue_flags.contains(vk::QueueFlags::GRAPHICS)
                    || p.queue_flags.contains(vk::QueueFlags::COMPUTE)
            })
            .map(|i| i as u32);

        let mut present = None;
        for (index, _properties) in properties.iter().enumerate() {
            if instance.get_physical_device_surface_support_khr(
                physical_device,
                index as u32,
                common.surface,
            )? {
                present = Some(index as u32);
                break;
            }
        }

        if let (Some(graphics), Some(present)) = (graphics, present) {
            Ok(Self {
                graphics_compute: graphics,
                present,
            })
        } else {
            Err(anyhow!("Missing required queue families."))
        }
    }
}
