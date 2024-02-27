use log::{info, warn};
use vulkanalia::prelude::v1_0::*;

use super::globals;

#[derive(Clone, Debug, Default)]
pub struct DescriptorsData {
    pub descriptor_set_layout: vk::DescriptorSetLayout,
    pub descriptor_pool: vk::DescriptorPool,
    pub descriptor_sets: Vec<vk::DescriptorSet>,
}

impl Drop for DescriptorsData {
    fn drop(&mut self) {
        unsafe {
            info!("destroying descriptors");
            if globals::get_device().device_wait_idle().is_err() {
                warn!("destroying descriptors failed");
                return;
            }

            globals::get_device().destroy_descriptor_set_layout(self.descriptor_set_layout, None);
            globals::get_device().destroy_descriptor_pool(self.descriptor_pool, None);
        }
    }
}
