use vulkanalia::prelude::v1_0::*;

#[derive(Clone, Debug, Default)]
pub struct CommandsData {
    pub main_command_pool: vk::CommandPool,
    pub command_pools: Vec<vk::CommandPool>,

    pub command_buffers: Vec<vk::CommandBuffer>,
    pub gravity_compute_command_buffers: Vec<vk::CommandBuffer>,
    pub mass_compute_command_buffers: Vec<vk::CommandBuffer>,
}
