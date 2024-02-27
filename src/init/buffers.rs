use anyhow::Result;
use std::ptr::copy_nonoverlapping as memcpy;
use std::{cmp::max, mem::size_of};
use vulkanalia::prelude::v1_0::*;

use crate::data::image_data::ImageData;
use crate::{
    data::{
        buffers_data::BuffersData, commands_data::CommandsData, common_data::CommonData, globals,
        swapchain_data::SwapchainData, uniform_buffer_object::UniformBufferObject, vertex::Vertex,
    },
    utils::resources,
};

pub unsafe fn create_shader_storage_buffers(
    instance: &Instance,
    vertices: &Vec<Vertex>,
    common: &CommonData,
    commands: &CommandsData,
    buffers: &mut BuffersData,
) -> Result<()> {
    let size = (size_of::<Vertex>() * vertices.len()) as u64;
    let mut storage_buffers = vec![];
    let mut storage_buffer_memories = vec![];

    let (staging_buffer, staging_buffer_memory) = resources::create_buffer(
        instance,
        common,
        size,
        vk::BufferUsageFlags::TRANSFER_SRC,
        vk::MemoryPropertyFlags::HOST_COHERENT | vk::MemoryPropertyFlags::HOST_VISIBLE,
    )?;

    let memory = globals::get_device().map_memory(
        staging_buffer_memory,
        0,
        size,
        vk::MemoryMapFlags::empty(),
    )?;
    memcpy(vertices.as_ptr(), memory.cast(), vertices.len());
    globals::get_device().unmap_memory(staging_buffer_memory);

    for _ in 0..globals::MAX_FRAMES_IN_FLIGHT {
        let (storage_buffer, storage_buffer_memory) = resources::create_buffer(
            instance,
            common,
            size,
            vk::BufferUsageFlags::TRANSFER_DST
                | vk::BufferUsageFlags::VERTEX_BUFFER
                | vk::BufferUsageFlags::STORAGE_BUFFER,
            vk::MemoryPropertyFlags::DEVICE_LOCAL,
        )?;

        resources::copy_buffer(common, commands, staging_buffer, storage_buffer, size)?;

        storage_buffers.push(storage_buffer);
        storage_buffer_memories.push(storage_buffer_memory);
    }

    globals::get_device().destroy_buffer(staging_buffer, None);
    globals::get_device().free_memory(staging_buffer_memory, None);

    buffers.storage_buffers = storage_buffers;
    buffers.storage_buffer_memories = storage_buffer_memories;
    Ok(())
}

pub unsafe fn create_uniform_buffers(
    instance: &Instance,
    common: &CommonData,
    swapchain: &SwapchainData,
    buffers: &mut BuffersData,
) -> Result<()> {
    buffers.uniform_buffers.clear();
    buffers.uniform_buffers_memory.clear();

    for _ in 0..swapchain.swapchain_images.len() {
        let (uniform_buffer, uniform_buffer_memory) = resources::create_buffer(
            instance,
            common,
            size_of::<UniformBufferObject>() as u64,
            vk::BufferUsageFlags::UNIFORM_BUFFER,
            vk::MemoryPropertyFlags::HOST_COHERENT | vk::MemoryPropertyFlags::HOST_VISIBLE,
        )?;

        buffers.uniform_buffers.push(uniform_buffer);
        buffers.uniform_buffers_memory.push(uniform_buffer_memory);
    }

    Ok(())
}

pub unsafe fn create_offscreen_images(
    instance: &Instance,
    common: &CommonData,
    commands: &CommandsData,
) -> Result<Vec<Vec<ImageData>>> {
    let mut image_sets = vec![];

    for _ in 0..globals::MAX_FRAMES_IN_FLIGHT {
        image_sets.push(create_downsampled_images(
            instance, common, commands,
        )?);
    }

    Ok(image_sets)
}

unsafe fn create_downsampled_images(
    instance: &Instance,
    common: &CommonData,
    commands: &CommandsData,
) -> Result<Vec<ImageData>> {
    let mut images = vec![];

    let mut width = globals::MASS_FIELD_SIZE;
    let mut height = globals::MASS_FIELD_SIZE;

    let min_len = globals::SHADER_FORCE_REGION_RADIUS / 2 + 1;

    loop {
        let (image, image_memory) = resources::create_image(
            instance,
            common,
            width,
            height,
            1,
            vk::SampleCountFlags::_1,
            vk::Format::R32G32B32A32_SFLOAT,
            vk::ImageTiling::OPTIMAL,
            vk::ImageUsageFlags::STORAGE | vk::ImageUsageFlags::TRANSFER_DST,
            vk::MemoryPropertyFlags::DEVICE_LOCAL,
        )?;

        resources::transition_image_layout(
            common,
            commands,
            image,
            vk::Format::R32G32B32A32_SFLOAT,
            vk::ImageLayout::UNDEFINED,
            vk::ImageLayout::GENERAL,
            1,
        )?;

        let image_view = resources::create_image_view(
            image,
            vk::Format::R32G32B32A32_SFLOAT,
            vk::ImageAspectFlags::COLOR,
            0,
            1,
        )?;

        images.push(ImageData {
            image,
            image_memory,
            image_view,
        });

        if width <= min_len && height <= min_len {
            break;
        }

        height = max(1, height / globals::MIP_LEVEL_DOWNSAMLING);
        width = max(1, width / globals::MIP_LEVEL_DOWNSAMLING);
    }

    Ok(images)
}
