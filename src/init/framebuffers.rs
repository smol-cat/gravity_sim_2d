use anyhow::Result;
use vulkanalia::prelude::v1_0::*;

use crate::data::swapchain_data::SwapchainData;

pub unsafe fn create_framebuffers(
    device: &Device,
    render_pass: vk::RenderPass,
    swapchain: &SwapchainData,
) -> Result<Vec<vk::Framebuffer>> {
    let framebuffers = swapchain
        .swapchain_image_views
        .iter()
        .map(|i| {
            let attachments = &[*i];
            let create_info = vk::FramebufferCreateInfo::builder()
                .render_pass(render_pass)
                .attachments(attachments)
                .width(swapchain.swapchain_extent.width)
                .height(swapchain.swapchain_extent.height)
                .layers(1);

            device.create_framebuffer(&create_info, None)
        })
        .collect::<Result<Vec<_>, _>>()?;

    Ok(framebuffers)
}
