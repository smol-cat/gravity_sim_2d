use vulkanalia::prelude::v1_0::*;

#[derive(Clone, Debug, Default)]
pub struct BuffersData {
    pub vertex_buffer: vk::Buffer,
    pub vertex_buffer_memory: vk::DeviceMemory,
}
