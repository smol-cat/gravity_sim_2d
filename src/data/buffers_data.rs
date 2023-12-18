use vulkanalia::prelude::v1_0::*;

#[derive(Clone, Debug, Default)]
pub struct BuffersData {
    pub storage_buffers: Vec<vk::Buffer>,
    pub storage_buffer_memories: Vec<vk::DeviceMemory>,

    pub uniform_buffers: Vec<vk::Buffer>,
    pub uniform_buffers_memory: Vec<vk::DeviceMemory>,
}
