use vulkanalia::vk;

#[derive(Clone, Debug, Default)]
pub struct SwapchainData {
    pub swapchain: vk::SwapchainKHR,
    pub swapchain_images: Vec<vk::Image>,
    pub swapchain_image_views: Vec<vk::ImageView>,

    pub swapchain_format: vk::Format,
    pub swapchain_extent: vk::Extent2D,
}
