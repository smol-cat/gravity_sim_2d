use vulkanalia::vk;

#[derive(Clone, Debug, Default)]
pub struct ImageData {
    pub image: vk::Image,
    pub image_view: vk::ImageView,
    pub image_memory: vk::DeviceMemory,
}
