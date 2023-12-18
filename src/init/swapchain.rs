use anyhow::{Ok, Result};
use log::info;
use vulkanalia::prelude::v1_0::*;
use vulkanalia::vk::KhrSwapchainExtension;
use winit::window::Window;

use crate::{
    data::{common_data::CommonData, swapchain_data::SwapchainData},
    utils::{queue_family_indices::QueueFamilyIndices, swapchain_support::SwapchainSupport, utils},
};

pub unsafe fn create_swapchain(
    device: &Device,
    common: &CommonData,
    instance: &Instance,
    window: &Window,
    swapchain: &mut SwapchainData,
) -> Result<()> {
    let indices = QueueFamilyIndices::get(instance, common, common.physical_device)?;
    let support = SwapchainSupport::get(instance, common.surface, common.physical_device)?;

    let swapchain_format = get_swapchain_surface_format(&support.formats);
    let swapchain_present_mode = get_swapchain_present_mode(&support.present_modes);
    let swaphchain_extent = get_swapchain_extent(window, support.capabilities);

    let mut image_count = support.capabilities.min_image_count + 1;
    info!("Swapchain images count: {}", image_count);

    if support.capabilities.max_image_count != 0
        && image_count > support.capabilities.max_image_count
    {
        image_count = support.capabilities.max_image_count;
    }

    let mut queue_family_indices = vec![];
    let image_sharing_mode = if indices.graphics_compute != indices.present {
        queue_family_indices.push(indices.graphics_compute);
        queue_family_indices.push(indices.present);
        vk::SharingMode::CONCURRENT
    } else {
        vk::SharingMode::EXCLUSIVE
    };

    let info = vk::SwapchainCreateInfoKHR::builder()
        .surface(common.surface)
        .min_image_count(image_count)
        .image_format(swapchain_format.format)
        .image_color_space(swapchain_format.color_space)
        .image_extent(swaphchain_extent)
        .image_array_layers(1)
        .image_usage(vk::ImageUsageFlags::COLOR_ATTACHMENT)
        .image_sharing_mode(image_sharing_mode)
        .queue_family_indices(&queue_family_indices)
        .pre_transform(support.capabilities.current_transform) // current_transform for no rotations
        .composite_alpha(vk::CompositeAlphaFlagsKHR::OPAQUE)
        .present_mode(swapchain_present_mode)
        .clipped(true)
        .old_swapchain(vk::SwapchainKHR::null());

    swapchain.swapchain = device.create_swapchain_khr(&info, None)?;
    swapchain.swapchain_images = device.get_swapchain_images_khr(swapchain.swapchain)?;

    swapchain.swapchain_format = swapchain_format.format;
    swapchain.swapchain_extent = swaphchain_extent;

    Ok(())
}

pub fn get_swapchain_surface_format(formats: &[vk::SurfaceFormatKHR]) -> vk::SurfaceFormatKHR {
    formats
        .iter()
        .cloned()
        .find(|f| {
            f.format == vk::Format::B8G8R8A8_SRGB
                && f.color_space == vk::ColorSpaceKHR::SRGB_NONLINEAR
        })
        .unwrap_or_else(|| formats[0])
}

pub fn get_swapchain_present_mode(present_modes: &[vk::PresentModeKHR]) -> vk::PresentModeKHR {
    present_modes
        .iter()
        .cloned()
        .find(|m| *m == vk::PresentModeKHR::MAILBOX)
        .unwrap_or(vk::PresentModeKHR::FIFO)
}

pub fn get_swapchain_extent(
    window: &Window,
    capabilities: vk::SurfaceCapabilitiesKHR,
) -> vk::Extent2D {
    if capabilities.current_extent.width != u32::max_value() {
        capabilities.current_extent
    } else {
        let size = window.inner_size();
        let clamp = |min: u32, max: u32, v: u32| min.max(max.min(v));

        vk::Extent2D::builder()
            .width(clamp(
                capabilities.min_image_extent.width,
                capabilities.max_image_extent.width,
                size.width,
            ))
            .height(clamp(
                capabilities.min_image_extent.height,
                capabilities.max_image_extent.height,
                size.height,
            ))
            .build()
    }
}

pub unsafe fn create_swapchain_image_views(
    device: &Device,
    swapchain: &SwapchainData,
) -> Result<Vec<vk::ImageView>> {
    Ok(swapchain
        .swapchain_images
        .iter()
        .map(|i| {
            utils::create_image_view(
                device,
                *i,
                swapchain.swapchain_format,
                vk::ImageAspectFlags::COLOR,
                1,
            )
        })
        .collect::<Result<Vec<_>, _>>()?)
}
