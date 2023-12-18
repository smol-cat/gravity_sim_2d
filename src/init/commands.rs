use anyhow::Result;
use vulkanalia::prelude::v1_0::*;

use crate::{
    data::{commands_data::CommandsData, common_data::CommonData, swapchain_data::SwapchainData},
    utils::queue_family_indices::QueueFamilyIndices,
};

pub unsafe fn create_command_pools(
    instance: &Instance,
    device: &Device,
    common: &CommonData,
    swapchain: &SwapchainData,
    commands: &mut CommandsData,
) -> Result<()> {
    commands.main_command_pool = create_command_pool(instance, device, common)?;

    let images_count = swapchain.swapchain_images.len();
    for _ in 0..images_count {
        let command_pool = create_command_pool(instance, device, common)?;
        commands.command_pools.push(command_pool);
    }

    Ok(())
}

pub unsafe fn create_command_pool(
    instance: &Instance,
    device: &Device,
    common: &CommonData,
) -> Result<vk::CommandPool> {
    let indices = QueueFamilyIndices::get(instance, common, common.physical_device)?;
    let info = vk::CommandPoolCreateInfo::builder()
        .flags(
            vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER
                | vk::CommandPoolCreateFlags::TRANSIENT,
        )
        .queue_family_index(indices.graphics_compute);

    Ok(device.create_command_pool(&info, None)?)
}

pub unsafe fn create_command_buffers(
    device: &Device,
    swapchain: &SwapchainData,
    command_pool: vk::CommandPool,
) -> Result<Vec<vk::CommandBuffer>> {
    let images_count = swapchain.swapchain_images.len();
    let mut command_buffers = vec![];
    for _ in 0..images_count {
        let allocate_info = vk::CommandBufferAllocateInfo::builder()
            .command_pool(command_pool)
            .level(vk::CommandBufferLevel::PRIMARY)
            .command_buffer_count(1);

        let command_buffer = device.allocate_command_buffers(&allocate_info)?[0];
        command_buffers.push(command_buffer);
    }

    Ok(command_buffers)
}
