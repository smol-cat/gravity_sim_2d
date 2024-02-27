use anyhow::{Ok, Result};
use vulkanalia::{prelude::v1_0::*, vk::SampleCountFlags};
use vulkanalia::vk::KhrSwapchainExtension;
use winit::window::Window;

use crate::{
    data::{common_data::CommonData, globals, swapchain_data::SwapchainData},
    utils::{
        queue_family_indices::QueueFamilyIndices, resources, swapchain_support::SwapchainSupport,
    },
};

pub unsafe fn create_swapchain(
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

    swapchain.swapchain = globals::get_device().create_swapchain_khr(&info, None)?;
    swapchain.swapchain_images =
        globals::get_device().get_swapchain_images_khr(swapchain.swapchain)?;

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
    swapchain: &SwapchainData,
) -> Result<Vec<vk::ImageView>> {
    resources::create_image_views(
        &swapchain.swapchain_images,
        swapchain.swapchain_format,
        vk::ImageAspectFlags::COLOR,
    )
}

pub unsafe fn create_render_pass(format: vk::Format) -> Result<vk::RenderPass> {
    let color_attachment = vk::AttachmentDescription::builder()
        .format(format)
        .samples(SampleCountFlags::_1)
        .load_op(vk::AttachmentLoadOp::CLEAR)
        .store_op(vk::AttachmentStoreOp::STORE)
        .stencil_load_op(vk::AttachmentLoadOp::DONT_CARE)
        .stencil_store_op(vk::AttachmentStoreOp::DONT_CARE)
        .initial_layout(vk::ImageLayout::UNDEFINED)
        .final_layout(vk::ImageLayout::PRESENT_SRC_KHR);

    let color_attachment_ref = vk::AttachmentReference::builder()
        .attachment(0)
        .layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL);

    let color_attachments = &[color_attachment_ref];
    let subpass = vk::SubpassDescription::builder()
        .pipeline_bind_point(vk::PipelineBindPoint::GRAPHICS)
        .color_attachments(color_attachments);

    let dependency = vk::SubpassDependency::builder()
        .src_subpass(vk::SUBPASS_EXTERNAL)
        .dst_subpass(0)
        .src_stage_mask(vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT)
        .src_access_mask(vk::AccessFlags::empty())
        .dst_stage_mask(vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT)
        .dst_access_mask(vk::AccessFlags::COLOR_ATTACHMENT_WRITE);

    let attachments = &[color_attachment];
    let subpasses = &[subpass];
    let dependencies = &[dependency];
    let info = vk::RenderPassCreateInfo::builder()
        .attachments(attachments)
        .subpasses(subpasses)
        .dependencies(dependencies);

    Ok(globals::get_device().create_render_pass(&info, None)?)
}
