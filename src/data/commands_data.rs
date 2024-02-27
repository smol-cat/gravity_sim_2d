use log::{info, warn};
use vulkanalia::prelude::v1_0::*;

use crate::data::globals;

#[derive(Clone, Debug, Default)]
pub struct CommandsData {
    pub main_command_pool: vk::CommandPool,

    pub command_buffers: Vec<vk::CommandBuffer>,
    pub gravity_compute_command_buffers: Vec<vk::CommandBuffer>,
    pub mass_compute_command_buffers: Vec<vk::CommandBuffer>,
    pub image_clear_command_buffers: Vec<vk::CommandBuffer>,
}

impl Drop for CommandsData {
    fn drop(&mut self) {
        unsafe {
            info!("destroying commands data");
            if globals::get_device().device_wait_idle().is_err() {
                warn!("destroying commands data failed");
                return;
            }

            globals::get_device().destroy_command_pool(self.main_command_pool, None);
        }
    }
}
