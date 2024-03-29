use anyhow::{Ok, Result};
use vulkanalia::prelude::v1_0::*;

use crate::data::{globals, swapchain_data::SwapchainData, sync_data::SyncData};

pub unsafe fn create_sync_objects(swapchain: &SwapchainData, sync: &mut SyncData) -> Result<()> {
    let semaphore_info = vk::SemaphoreCreateInfo::builder();
    let fence_info = vk::FenceCreateInfo::builder().flags(vk::FenceCreateFlags::SIGNALED);
    sync.first_gravity_compute = true;

    for _ in 0..globals::MAX_FRAMES_IN_FLIGHT {
        sync.image_available_semaphores
            .push(globals::get_device().create_semaphore(&semaphore_info, None)?);
        sync.image_clear_finished_semaphores
            .push(globals::get_device().create_semaphore(&semaphore_info, None)?);
        sync.render_finished_semaphores
            .push(globals::get_device().create_semaphore(&semaphore_info, None)?);

        sync.gravity_compute_finished_semaphores
            .push(globals::get_device().create_semaphore(&semaphore_info, None)?);
        sync.mass_compute_finished_semaphores
            .push(globals::get_device().create_semaphore(&semaphore_info, None)?);

        sync.gravity_read_finished_semaphores
            .push(globals::get_device().create_semaphore(&semaphore_info, None)?);

        sync.in_flight_fences
            .push(globals::get_device().create_fence(&fence_info, None)?);
    }

    sync.images_in_flight = swapchain
        .swapchain_images
        .iter()
        .map(|_| vk::Fence::null())
        .collect();

    Ok(())
}
