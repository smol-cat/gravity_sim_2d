use std::mem::size_of;

use anyhow::{Ok, Result};
use vulkanalia::prelude::v1_0::*;

use crate::data::{
    buffers_data::BuffersData, descriptors_data::DescriptorsData, globals,
    swapchain_data::SwapchainData, uniform_buffer_object::UniformBufferObject, vertex::Vertex,
};

pub unsafe fn create_gravity_descriptor_set_layout(
    device: &Device,
) -> Result<vk::DescriptorSetLayout> {
    let storage_binding = vk::DescriptorSetLayoutBinding::builder()
        .binding(0)
        .descriptor_type(vk::DescriptorType::STORAGE_BUFFER)
        .descriptor_count(1)
        .stage_flags(vk::ShaderStageFlags::COMPUTE);

    let ubo_binding = vk::DescriptorSetLayoutBinding::builder()
        .binding(2)
        .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
        .descriptor_count(1)
        .stage_flags(vk::ShaderStageFlags::COMPUTE);

    let image_storage_binding = vk::DescriptorSetLayoutBinding::builder()
        .binding(3)
        .descriptor_type(vk::DescriptorType::STORAGE_IMAGE)
        .descriptor_count(globals::MAX_MIP_LEVELS)
        .stage_flags(vk::ShaderStageFlags::COMPUTE);

    let bindings = &[
        storage_binding,
        storage_binding.binding(1),
        ubo_binding,
        image_storage_binding,
    ];
    let info = vk::DescriptorSetLayoutCreateInfo::builder().bindings(bindings);
    Ok(device.create_descriptor_set_layout(&info, None)?)
}

pub unsafe fn create_mass_descriptor_set_layout(
    device: &Device,
) -> Result<vk::DescriptorSetLayout> {
    let storage_binding = vk::DescriptorSetLayoutBinding::builder()
        .binding(0)
        .descriptor_type(vk::DescriptorType::STORAGE_BUFFER)
        .descriptor_count(1)
        .stage_flags(vk::ShaderStageFlags::COMPUTE);

    let image_storage_binding = vk::DescriptorSetLayoutBinding::builder()
        .binding(1)
        .descriptor_type(vk::DescriptorType::STORAGE_IMAGE)
        .descriptor_count(globals::MAX_MIP_LEVELS)
        .stage_flags(vk::ShaderStageFlags::COMPUTE);

    let bindings = &[storage_binding, image_storage_binding];
    let info = vk::DescriptorSetLayoutCreateInfo::builder().bindings(bindings);
    Ok(device.create_descriptor_set_layout(&info, None)?)
}

pub unsafe fn create_gravity_descriptor_pool(
    device: &Device,
    swapchain: &SwapchainData,
) -> Result<vk::DescriptorPool> {
    let storage_buffer_size = vk::DescriptorPoolSize::builder()
        .type_(vk::DescriptorType::STORAGE_BUFFER)
        .descriptor_count(swapchain.swapchain_images.len() as u32);

    let ubo_size = vk::DescriptorPoolSize::builder()
        .type_(vk::DescriptorType::UNIFORM_BUFFER)
        .descriptor_count(swapchain.swapchain_images.len() as u32);

    let image_storage_buffer_size = vk::DescriptorPoolSize::builder()
        .type_(vk::DescriptorType::STORAGE_IMAGE)
        .descriptor_count(swapchain.swapchain_images.len() as u32 * globals::MAX_MIP_LEVELS);

    let pool_sizes = &[
        storage_buffer_size,
        storage_buffer_size,
        ubo_size,
        image_storage_buffer_size,
    ];
    let info = vk::DescriptorPoolCreateInfo::builder()
        .pool_sizes(pool_sizes)
        .max_sets(swapchain.swapchain_images.len() as u32);

    Ok(device.create_descriptor_pool(&info, None)?)
}

pub unsafe fn create_mass_descriptor_pool(
    device: &Device,
    swapchain: &SwapchainData,
) -> Result<vk::DescriptorPool> {
    let storage_buffer_size = vk::DescriptorPoolSize::builder()
        .type_(vk::DescriptorType::STORAGE_BUFFER)
        .descriptor_count(swapchain.swapchain_images.len() as u32);

    let image_storage_buffer_size = vk::DescriptorPoolSize::builder()
        .type_(vk::DescriptorType::STORAGE_IMAGE)
        .descriptor_count(swapchain.swapchain_images.len() as u32 * globals::MAX_MIP_LEVELS);

    let pool_sizes = &[storage_buffer_size, image_storage_buffer_size];
    let info = vk::DescriptorPoolCreateInfo::builder()
        .pool_sizes(pool_sizes)
        .max_sets(swapchain.swapchain_images.len() as u32);

    Ok(device.create_descriptor_pool(&info, None)?)
}

pub unsafe fn create_gravity_descriptor_sets(
    device: &Device,
    buffers: &BuffersData,
    vertices: &Vec<Vertex>,
    descriptors: &mut DescriptorsData,
) -> Result<()> {
    let layouts = vec![descriptors.descriptor_set_layout; globals::MAX_FRAMES_IN_FLIGHT];
    let info = vk::DescriptorSetAllocateInfo::builder()
        .descriptor_pool(descriptors.descriptor_pool)
        .set_layouts(&layouts);

    descriptors.descriptor_sets = device.allocate_descriptor_sets(&info)?;

    for i in 0..globals::MAX_FRAMES_IN_FLIGHT {
        let info = vk::DescriptorBufferInfo::builder()
            .buffer(buffers.uniform_buffers[i])
            .offset(0)
            .range(size_of::<UniformBufferObject>() as u64);

        let buffer_info = &[info];
        let ubo_write = vk::WriteDescriptorSet::builder()
            .dst_set(descriptors.descriptor_sets[i])
            .dst_binding(2)
            .dst_array_element(0)
            .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
            .buffer_info(buffer_info);

        let storage_last_frame_info = vk::DescriptorBufferInfo::builder()
            .buffer(buffers.storage_buffers[(i + 1) % globals::MAX_FRAMES_IN_FLIGHT])
            .offset(0)
            .range((size_of::<Vertex>() * vertices.len()) as u64);

        let storage_infos = &[storage_last_frame_info];
        let ssbo_last_frame_write = vk::WriteDescriptorSet::builder()
            .dst_set(descriptors.descriptor_sets[i])
            .dst_binding(0)
            .dst_array_element(0)
            .descriptor_type(vk::DescriptorType::STORAGE_BUFFER)
            .buffer_info(storage_infos);

        let storage_curr_frame_info = vk::DescriptorBufferInfo::builder()
            .buffer(buffers.storage_buffers[i])
            .offset(0)
            .range((size_of::<Vertex>() * vertices.len()) as u64);

        let storage_infos = &[storage_curr_frame_info];
        let ssbo_curr_frame_write = vk::WriteDescriptorSet::builder()
            .dst_set(descriptors.descriptor_sets[i])
            .dst_binding(1)
            .dst_array_element(0)
            .descriptor_type(vk::DescriptorType::STORAGE_BUFFER)
            .buffer_info(storage_infos);

        let storage_image_infos = buffers.offscreen_images[i]
            .iter()
            .map(|image| {
                vk::DescriptorImageInfo::builder()
                    .image_view(image.image_view)
                    .image_layout(vk::ImageLayout::GENERAL)
            })
            .collect::<Vec<_>>();

        let storage_image_write = vk::WriteDescriptorSet::builder()
            .dst_set(descriptors.descriptor_sets[i])
            .dst_binding(3)
            .dst_array_element(0)
            .descriptor_type(vk::DescriptorType::STORAGE_IMAGE)
            .image_info(&storage_image_infos);

        device.update_descriptor_sets(
            &[
                ssbo_last_frame_write,
                ssbo_curr_frame_write,
                ubo_write,
                storage_image_write,
            ],
            &[] as &[vk::CopyDescriptorSet],
        );
    }

    Ok(())
}

pub unsafe fn create_mass_descriptor_sets(
    device: &Device,
    buffers: &BuffersData,
    vertices: &Vec<Vertex>,
    descriptors: &mut DescriptorsData,
) -> Result<()> {
    let layouts = vec![descriptors.descriptor_set_layout; globals::MAX_FRAMES_IN_FLIGHT];
    let info = vk::DescriptorSetAllocateInfo::builder()
        .descriptor_pool(descriptors.descriptor_pool)
        .set_layouts(&layouts);

    descriptors.descriptor_sets = device.allocate_descriptor_sets(&info)?;

    for i in 0..globals::MAX_FRAMES_IN_FLIGHT {
        let storage_buffer_info = vk::DescriptorBufferInfo::builder()
            .buffer(buffers.storage_buffers[(i + 1) % globals::MAX_FRAMES_IN_FLIGHT])
            .offset(0)
            .range((size_of::<Vertex>() * vertices.len()) as u64);

        let storage_infos = &[storage_buffer_info];
        let storage_buffer_write = vk::WriteDescriptorSet::builder()
            .dst_set(descriptors.descriptor_sets[i])
            .dst_binding(0)
            .dst_array_element(0)
            .descriptor_type(vk::DescriptorType::STORAGE_BUFFER)
            .buffer_info(storage_infos);

        let storage_image_infos = buffers.offscreen_images[i]
            .iter()
            .map(|image| {
                vk::DescriptorImageInfo::builder()
                    .image_view(image.image_view)
                    .image_layout(vk::ImageLayout::GENERAL)
            })
            .collect::<Vec<_>>();

        let storage_image_write = vk::WriteDescriptorSet::builder()
            .dst_set(descriptors.descriptor_sets[i])
            .dst_binding(1)
            .dst_array_element(0)
            .descriptor_type(vk::DescriptorType::STORAGE_IMAGE)
            .image_info(&storage_image_infos);

        device.update_descriptor_sets(
            &[storage_buffer_write, storage_image_write],
            &[] as &[vk::CopyDescriptorSet],
        );
    }

    Ok(())
}
