use anyhow::Result;
use std::mem::size_of;
use std::ptr::copy_nonoverlapping as memcpy;

use vulkanalia::prelude::v1_0::*;

use crate::{
    data::{commands_data::CommandsData, common_data::CommonData, vertex::Vertex},
    utils::resources,
};

pub unsafe fn create_vertex_buffer(
    instance: &Instance,
    device: &Device,
    vertices: &Vec<Vertex>,
    common: &CommonData,
    commands: &CommandsData,
) -> Result<(vk::Buffer, vk::DeviceMemory)> {
    let size = (size_of::<Vertex>() * vertices.len()) as u64;

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

    let (vertex_buffer, vertex_buffer_memory) = resources::create_buffer(
        instance,
        device,
        common,
        size,
        vk::BufferUsageFlags::TRANSFER_DST | vk::BufferUsageFlags::VERTEX_BUFFER,
        vk::MemoryPropertyFlags::HOST_COHERENT | vk::MemoryPropertyFlags::HOST_VISIBLE,
    )?;

    resources::copy_buffer(
        device,
        common,
        commands,
        staging_buffer,
        vertex_buffer,
        size,
    )?;

    device.destroy_buffer(staging_buffer, None);
    device.free_memory(staging_buffer_memory, None);

    Ok((vertex_buffer, vertex_buffer_memory))
}
