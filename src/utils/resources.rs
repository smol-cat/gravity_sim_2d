use anyhow::{anyhow, Result};
use vulkanalia::prelude::v1_0::*;

use crate::data::{commands_data::CommandsData, common_data::CommonData};

pub unsafe fn create_buffer(
    instance: &Instance,
    device: &Device,
    common: &CommonData,
    size: vk::DeviceSize,
    usage: vk::BufferUsageFlags,
    properties: vk::MemoryPropertyFlags,
) -> Result<(vk::Buffer, vk::DeviceMemory)> {
    let buffer_info = vk::BufferCreateInfo::builder()
        .size(size)
        .usage(usage)
        .sharing_mode(vk::SharingMode::EXCLUSIVE);

    let buffer = device.create_buffer(&buffer_info, None)?;
    let requirements = device.get_buffer_memory_requirements(buffer);
    let memory_info = vk::MemoryAllocateInfo::builder()
        .allocation_size(requirements.size)
        .memory_type_index(get_memory_type_index(
            instance,
            common,
            properties,
            requirements,
        )?);

    let buffer_memory = device.allocate_memory(&memory_info, None)?;
    device.bind_buffer_memory(buffer, buffer_memory, 0)?;

    Ok((buffer, buffer_memory))
}

unsafe fn get_memory_type_index(
    instance: &Instance,
    data: &CommonData,
    properties: vk::MemoryPropertyFlags,
    requirements: vk::MemoryRequirements,
) -> Result<u32> {
    let memory = instance.get_physical_device_memory_properties(data.physical_device);
    (0..memory.memory_type_count)
        .find(|i| {
            let suitable = (requirements.memory_type_bits & (1 << i)) != 0;
            let memory_type = memory.memory_types[*i as usize];
            suitable && memory_type.property_flags.contains(properties)
        })
        .ok_or_else(|| anyhow!("Failed to find suitable memory type"))
}

pub unsafe fn copy_buffer(
    device: &Device,
    common: &CommonData,
    commands: &CommandsData,
    source: vk::Buffer,
    destination: vk::Buffer,
    size: vk::DeviceSize,
) -> Result<()> {
    let command_buffer = begin_single_time_commands(device, commands)?;
    let regions = vk::BufferCopy::builder().size(size);
    device.cmd_copy_buffer(command_buffer, source, destination, &[regions]);
    end_single_time_commands(device, common, commands, command_buffer)?;

    Ok(())
}
pub unsafe fn begin_single_time_commands(
    device: &Device,
    commands: &CommandsData,
) -> Result<vk::CommandBuffer> {
    let info = vk::CommandBufferAllocateInfo::builder()
        .level(vk::CommandBufferLevel::PRIMARY)
        .command_pool(commands.main_command_pool)
        .command_buffer_count(1);

    let command_buffer = device.allocate_command_buffers(&info)?[0];

    let info =
        vk::CommandBufferBeginInfo::builder().flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT);

    device.begin_command_buffer(command_buffer, &info)?;
    Ok(command_buffer)
}

pub unsafe fn end_single_time_commands(
    device: &Device,
    common: &CommonData,
    commands: &CommandsData,
    command_buffer: vk::CommandBuffer,
) -> Result<()> {
    device.end_command_buffer(command_buffer)?;

    let command_buffers = &[command_buffer];
    let info = vk::SubmitInfo::builder().command_buffers(command_buffers);

    device.queue_submit(common.graphics_queue, &[info], vk::Fence::null())?;
    device.queue_wait_idle(common.graphics_queue)?;

    device.free_command_buffers(commands.main_command_pool, &[command_buffer]);

    Ok(())
}

pub unsafe fn create_image(
    instance: &Instance,
    device: &Device,
    common: &CommonData,
    width: u32,
    height: u32,
    mip_levels: u32,
    samples: vk::SampleCountFlags,
    format: vk::Format,
    tiling: vk::ImageTiling,
    usage: vk::ImageUsageFlags,
    properties: vk::MemoryPropertyFlags,
) -> Result<(vk::Image, vk::DeviceMemory)> {
    let info = vk::ImageCreateInfo::builder()
        .image_type(vk::ImageType::_2D)
        .extent(vk::Extent3D {
            width,
            height,
            depth: 1,
        })
        .mip_levels(mip_levels)
        .array_layers(1)
        .format(format)
        .tiling(tiling)
        .initial_layout(vk::ImageLayout::UNDEFINED)
        .usage(usage)
        .samples(samples)
        .sharing_mode(vk::SharingMode::EXCLUSIVE)
        .flags(vk::ImageCreateFlags::empty()); // voxels material

    let image = device.create_image(&info, None)?;

    let requirements = device.get_image_memory_requirements(image);

    let info = vk::MemoryAllocateInfo::builder()
        .allocation_size(requirements.size)
        .memory_type_index(get_memory_type_index(
            instance,
            common,
            properties,
            requirements,
        )?);

    let image_memory = device.allocate_memory(&info, None)?;
    device.bind_image_memory(image, image_memory, 0)?;

    Ok((image, image_memory))
}

pub unsafe fn create_image_view(
    device: &Device,
    image: vk::Image,
    format: vk::Format,
    aspects: vk::ImageAspectFlags,
    mip_levels: u32,
) -> Result<vk::ImageView> {
    let subresource_range = vk::ImageSubresourceRange::builder()
        .aspect_mask(aspects)
        .base_mip_level(0)
        .level_count(mip_levels)
        .base_array_layer(0)
        .layer_count(1);

    let info = vk::ImageViewCreateInfo::builder()
        .format(format)
        .image(image)
        .view_type(vk::ImageViewType::_2D)
        .subresource_range(subresource_range);

    Ok(device.create_image_view(&info, None)?)
}

pub unsafe fn create_image_views(
    device: &Device,
    images: &Vec<vk::Image>,
    format: vk::Format,
    aspect: vk::ImageAspectFlags,
    mip_levels: u32,
) -> Result<Vec<vk::ImageView>> {
    Ok(images
        .iter()
        .map(|i| create_image_view(device, *i, format, aspect, mip_levels))
        .collect::<Result<Vec<_>, _>>()?)
}
