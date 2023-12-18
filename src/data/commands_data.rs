use vulkanalia::prelude::v1_0::*;

#[derive(Clone, Debug, Default)]
pub struct CommandsData {
    pub main_command_pool: vk::CommandPool,
    pub command_pools: Vec<vk::CommandPool>,
    pub command_buffers: Vec<vk::CommandBuffer>,
    pub compute_commands_buffers: Vec<vk::CommandBuffer>,
}
