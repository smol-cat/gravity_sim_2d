use anyhow::Result;

use vulkanalia::prelude::v1_0::*;

pub unsafe fn create_image_view(
    device: &Device,
    image: vk::Image,
    format: vk::Format,
    aspects: vk::ImageAspectFlags,
    mip_levels: u32,
) -> Result<vk::ImageView> {
    let subresource_range = vk::ImageSubresourceRange::builder()
        .aspect_mask(aspects)
        .base_mip_level(0)
        .level_count(mip_levels)
        .base_array_layer(0)
        .layer_count(1);

    let info = vk::ImageViewCreateInfo::builder()
        .format(format)
        .image(image)
        .view_type(vk::ImageViewType::_2D)
        .subresource_range(subresource_range);

    Ok(device.create_image_view(&info, None)?)
}
