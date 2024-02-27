use anyhow::Result;
use vulkanalia::prelude::v1_0::*;

use crate::{data::{common_data::CommonData, globals}, utils::queue_family_indices::QueueFamilyIndices};

pub unsafe fn create_command_pool(
    instance: &Instance,
    common: &CommonData,
) -> Result<vk::CommandPool> {
    let indices = QueueFamilyIndices::get(instance, common, common.physical_device)?;
    let info = vk::CommandPoolCreateInfo::builder()
        .flags(
            vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER
                | vk::CommandPoolCreateFlags::TRANSIENT,
        )
        .queue_family_index(indices.graphics_compute);

    Ok(globals::get_device().create_command_pool(&info, None)?)
}

pub unsafe fn create_command_buffers(
    count: usize,
    command_pool: vk::CommandPool,
) -> Result<Vec<vk::CommandBuffer>> {
    let mut command_buffers = vec![];
    for _ in 0..count {
        let allocate_info = vk::CommandBufferAllocateInfo::builder()
            .command_pool(command_pool)
            .level(vk::CommandBufferLevel::PRIMARY)
            .command_buffer_count(1);

        let command_buffer = globals::get_device().allocate_command_buffers(&allocate_info)?[0];
        command_buffers.push(command_buffer);
    }

    Ok(command_buffers)
}
