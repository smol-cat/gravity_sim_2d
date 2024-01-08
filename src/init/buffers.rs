use anyhow::Result;
use std::mem::size_of;
use std::ptr::copy_nonoverlapping as memcpy;

use vulkanalia::prelude::v1_0::*;

use crate::{
    data::{
        buffers_data::BuffersData, commands_data::CommandsData, common_data::CommonData, globals,
        swapchain_data::SwapchainData, uniform_buffer_object::UniformBufferObject, vertex::Vertex,
    },
    utils::resources,
};

pub unsafe fn create_shader_storage_buffers(
    instance: &Instance,
    device: &Device,
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
        device,
        common,
        size,
        vk::BufferUsageFlags::TRANSFER_SRC,
        vk::MemoryPropertyFlags::HOST_COHERENT | vk::MemoryPropertyFlags::HOST_VISIBLE,
    )?;

    let memory = device.map_memory(staging_buffer_memory, 0, size, vk::MemoryMapFlags::empty())?;
    memcpy(vertices.as_ptr(), memory.cast(), vertices.len());
    device.unmap_memory(staging_buffer_memory);

    for _ in 0..globals::MAX_FRAMES_IN_FLIGHT {
        let (storage_buffer, storage_buffer_memory) = resources::create_buffer(
            instance,
            device,
            common,
            size,
            vk::BufferUsageFlags::TRANSFER_DST
                | vk::BufferUsageFlags::VERTEX_BUFFER
                | vk::BufferUsageFlags::STORAGE_BUFFER,
            vk::MemoryPropertyFlags::DEVICE_LOCAL,
        )?;

        resources::copy_buffer(
            device,
            common,
            commands,
            staging_buffer,
            storage_buffer,
            size,
        )?;

        storage_buffers.push(storage_buffer);
        storage_buffer_memories.push(storage_buffer_memory);
    }

    device.destroy_buffer(staging_buffer, None);
    device.free_memory(staging_buffer_memory, None);

    buffers.storage_buffers = storage_buffers;
    buffers.storage_buffer_memories = storage_buffer_memories;
    Ok(())
}

pub unsafe fn create_uniform_buffers(
    instance: &Instance,
    device: &Device,
    common: &CommonData,
    swapchain: &SwapchainData,
    buffers: &mut BuffersData,
) -> Result<()> {
    buffers.uniform_buffers.clear();
    buffers.uniform_buffers_memory.clear();

    for _ in 0..swapchain.swapchain_images.len() {
        let (uniform_buffer, uniform_buffer_memory) = resources::create_buffer(
            instance,
            device,
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
    device: &Device,
    common: &CommonData,
    swapchain: &SwapchainData,
) -> Result<(Vec<vk::Image>, Vec<vk::DeviceMemory>)> {
    let mut images = vec![];
    let mut image_memories = vec![];

    for _ in 0..swapchain.swapchain_images.len() {
        let (image, image_memory) = resources::create_image(
            instance,
            device,
            common,
            swapchain.swapchain_extent.width,
            swapchain.swapchain_extent.height,
            1,
            vk::SampleCountFlags::_1,
            vk::Format::R32_SFLOAT,
            vk::ImageTiling::OPTIMAL,
            vk::ImageUsageFlags::SAMPLED | vk::ImageUsageFlags::COLOR_ATTACHMENT,
            vk::MemoryPropertyFlags::DEVICE_LOCAL,
        )?;

        images.push(image);
        image_memories.push(image_memory);
    }

    Ok((images, image_memories))
}
