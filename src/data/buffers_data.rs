use log::{info, warn};
use vulkanalia::prelude::v1_0::*;

use super::{globals, image_data::ImageData};

#[derive(Debug, Default)]
pub struct BuffersData {
    pub storage_buffers: Vec<vk::Buffer>,
    pub storage_buffer_memories: Vec<vk::DeviceMemory>,

    pub uniform_buffers: Vec<vk::Buffer>,
    pub uniform_buffers_memory: Vec<vk::DeviceMemory>,

    pub offscreen_images: Vec<Vec<ImageData>>,
}

impl Drop for BuffersData {
    fn drop(&mut self) {
        unsafe {
            info!("destroying buffers data");
            if globals::get_device_opt().is_none() {
                return;
            }

            if globals::get_device().device_wait_idle().is_err() {
                warn!("destroying buffers data failed");
                return;
            }

            self.uniform_buffers
                .iter()
                .for_each(|s| globals::get_device().destroy_buffer(*s, None));
            self.uniform_buffers_memory
                .iter()
                .for_each(|s| globals::get_device().free_memory(*s, None));

            self.storage_buffers
                .iter()
                .for_each(|s| globals::get_device().destroy_buffer(*s, None));
            self.storage_buffer_memories
                .iter()
                .for_each(|s| globals::get_device().free_memory(*s, None));
        }
    }
}
