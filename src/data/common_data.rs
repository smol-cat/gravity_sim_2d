use vulkanalia::prelude::v1_0::*;

#[derive(Clone, Debug, Default)]
pub struct CommonData {
    pub messenger: vk::DebugUtilsMessengerEXT,
    pub surface: vk::SurfaceKHR,
    pub physical_device: vk::PhysicalDevice,

    pub present_queue: vk::Queue,
    pub graphics_queue: vk::Queue,
}
