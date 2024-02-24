use anyhow::Result;
use vulkanalia::prelude::v1_0::*;

pub unsafe fn create_framebuffers(
    device: &Device,
    render_pass: vk::RenderPass,
    extent: &vk::Extent2D,
    image_views: &Vec<vk::ImageView>,
) -> Result<Vec<vk::Framebuffer>> {
    let framebuffers = image_views
        .iter()
        .map(|i| {
            let attachments = &[*i];
            let create_info = vk::FramebufferCreateInfo::builder()
                .render_pass(render_pass)
                .attachments(attachments)
                .width(extent.width)
                .height(extent.height)
                .layers(1);

            device.create_framebuffer(&create_info, None)
        })
        .collect::<Result<Vec<_>, _>>()?;

    Ok(framebuffers)
}
