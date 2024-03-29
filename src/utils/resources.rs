use anyhow::{anyhow, Result};
use vulkanalia::prelude::v1_0::*;

use crate::data::{commands_data::CommandsData, common_data::CommonData, globals};

pub unsafe fn create_buffer(
    instance: &Instance,
    common: &CommonData,
    size: vk::DeviceSize,
    usage: vk::BufferUsageFlags,
    properties: vk::MemoryPropertyFlags,
) -> Result<(vk::Buffer, vk::DeviceMemory)> {
    let buffer_info = vk::BufferCreateInfo::builder()
        .size(size)
        .usage(usage)
        .sharing_mode(vk::SharingMode::EXCLUSIVE);

    let buffer = globals::get_device().create_buffer(&buffer_info, None)?;
    let requirements = globals::get_device().get_buffer_memory_requirements(buffer);
    let memory_info = vk::MemoryAllocateInfo::builder()
        .allocation_size(requirements.size)
        .memory_type_index(get_memory_type_index(
            instance,
            common,
            properties,
            requirements,
        )?);

    let buffer_memory = globals::get_device().allocate_memory(&memory_info, None)?;
    globals::get_device().bind_buffer_memory(buffer, buffer_memory, 0)?;

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
    common: &CommonData,
    commands: &CommandsData,
    source: vk::Buffer,
    destination: vk::Buffer,
    size: vk::DeviceSize,
) -> Result<()> {
    let command_buffer = begin_single_time_commands(commands)?;
    let regions = vk::BufferCopy::builder().size(size);
    globals::get_device().cmd_copy_buffer(command_buffer, source, destination, &[regions]);
    end_single_time_commands(common, commands, command_buffer)?;

    Ok(())
}

pub unsafe fn transition_image_layout(
    common: &CommonData,
    commands: &CommandsData,
    image: vk::Image,
    format: vk::Format,
    old_layout: vk::ImageLayout,
    new_layout: vk::ImageLayout,
    mip_levels: u32,
) -> Result<()> {
    let aspect_mask = if new_layout == vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL {
        match format {
            vk::Format::D32_SFLOAT_S8_UINT | vk::Format::D24_UNORM_S8_UINT => {
                vk::ImageAspectFlags::DEPTH | vk::ImageAspectFlags::STENCIL
            }
            _ => vk::ImageAspectFlags::DEPTH,
        }
    } else {
        vk::ImageAspectFlags::COLOR
    };

    let (src_access_mask, dst_access_mask, src_stage_mask, dst_stage_mask) =
        match (old_layout, new_layout) {
            (vk::ImageLayout::UNDEFINED, vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL) => (
                vk::AccessFlags::empty(),
                vk::AccessFlags::DEPTH_STENCIL_ATTACHMENT_READ
                    | vk::AccessFlags::DEPTH_STENCIL_ATTACHMENT_WRITE,
                vk::PipelineStageFlags::TOP_OF_PIPE,
                vk::PipelineStageFlags::EARLY_FRAGMENT_TESTS,
            ),
            (vk::ImageLayout::UNDEFINED, vk::ImageLayout::TRANSFER_DST_OPTIMAL) => (
                vk::AccessFlags::empty(),
                vk::AccessFlags::TRANSFER_WRITE,
                vk::PipelineStageFlags::TOP_OF_PIPE,
                vk::PipelineStageFlags::TRANSFER,
            ),
            (vk::ImageLayout::TRANSFER_DST_OPTIMAL, vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL) => (
                vk::AccessFlags::TRANSFER_WRITE,
                vk::AccessFlags::SHADER_READ,
                vk::PipelineStageFlags::TRANSFER,
                vk::PipelineStageFlags::FRAGMENT_SHADER,
            ),
            (vk::ImageLayout::UNDEFINED, vk::ImageLayout::GENERAL) => (
                vk::AccessFlags::empty(),
                vk::AccessFlags::SHADER_READ | vk::AccessFlags::SHADER_WRITE,
                vk::PipelineStageFlags::TRANSFER,
                vk::PipelineStageFlags::COMPUTE_SHADER,
            ),
            _ => return Err(anyhow!("Unsupported image layout transition")),
        };

    let command_buffer = begin_single_time_commands(commands)?;

    let subresource = vk::ImageSubresourceRange::builder()
        .aspect_mask(aspect_mask)
        .base_mip_level(0)
        .level_count(mip_levels)
        .base_array_layer(0)
        .layer_count(1);

    let barrier = vk::ImageMemoryBarrier::builder()
        .old_layout(old_layout)
        .new_layout(new_layout)
        .src_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
        .dst_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
        .image(image)
        .subresource_range(subresource)
        .src_access_mask(src_access_mask)
        .dst_access_mask(dst_access_mask);

    globals::get_device().cmd_pipeline_barrier(
        command_buffer,
        src_stage_mask,
        dst_stage_mask,
        vk::DependencyFlags::empty(),
        &[] as &[vk::MemoryBarrier],
        &[] as &[vk::BufferMemoryBarrier],
        &[barrier],
    );

    end_single_time_commands(common, commands, command_buffer)?;

    Ok(())
}

pub unsafe fn begin_single_time_commands(commands: &CommandsData) -> Result<vk::CommandBuffer> {
    let info = vk::CommandBufferAllocateInfo::builder()
        .level(vk::CommandBufferLevel::PRIMARY)
        .command_pool(commands.main_command_pool)
        .command_buffer_count(1);

    let command_buffer = globals::get_device().allocate_command_buffers(&info)?[0];

    let info =
        vk::CommandBufferBeginInfo::builder().flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT);

    globals::get_device().begin_command_buffer(command_buffer, &info)?;
    Ok(command_buffer)
}

pub unsafe fn end_single_time_commands(
    common: &CommonData,
    commands: &CommandsData,
    command_buffer: vk::CommandBuffer,
) -> Result<()> {
    globals::get_device().end_command_buffer(command_buffer)?;

    let command_buffers = &[command_buffer];
    let info = vk::SubmitInfo::builder().command_buffers(command_buffers);

    globals::get_device().queue_submit(common.graphics_queue, &[info], vk::Fence::null())?;
    globals::get_device().queue_wait_idle(common.graphics_queue)?;

    globals::get_device().free_command_buffers(commands.main_command_pool, &[command_buffer]);

    Ok(())
}

pub unsafe fn create_image(
    instance: &Instance,
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

    let image = globals::get_device().create_image(&info, None)?;

    let requirements = globals::get_device().get_image_memory_requirements(image);

    let info = vk::MemoryAllocateInfo::builder()
        .allocation_size(requirements.size)
        .memory_type_index(get_memory_type_index(
            instance,
            common,
            properties,
            requirements,
        )?);

    let image_memory = globals::get_device().allocate_memory(&info, None)?;
    globals::get_device().bind_image_memory(image, image_memory, 0)?;

    Ok((image, image_memory))
}

pub unsafe fn create_image_view(
    image: vk::Image,
    format: vk::Format,
    aspects: vk::ImageAspectFlags,
    base_level: u32,
    mip_levels: u32,
) -> Result<vk::ImageView> {
    let subresource_range = vk::ImageSubresourceRange::builder()
        .aspect_mask(aspects)
        .base_mip_level(base_level)
        .level_count(mip_levels)
        .base_array_layer(0)
        .layer_count(1);

    let info = vk::ImageViewCreateInfo::builder()
        .format(format)
        .image(image)
        .view_type(vk::ImageViewType::_2D)
        .subresource_range(subresource_range);

    Ok(globals::get_device().create_image_view(&info, None)?)
}

pub unsafe fn create_image_views(
    images: &Vec<vk::Image>,
    format: vk::Format,
    aspect: vk::ImageAspectFlags,
) -> Result<Vec<vk::ImageView>> {
    Ok(images
        .iter()
        .map(|i| create_image_view(*i, format, aspect, 0, 1))
        .collect::<Result<Vec<_>, _>>()?)
}

//pub fn get_mip_levels(swapchain: &SwapchainData) -> u32 {
//(swapchain
//.swapchain_extent
//.width
//.max(swapchain.swapchain_extent.height) as f32)
//.log(globals::MIP_LEVEL_DOWNSAMLING as f32)
//.floor() as u32
//+ 1
//}
