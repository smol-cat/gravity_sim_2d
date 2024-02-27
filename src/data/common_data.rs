use vulkanalia::vk;

#[derive(Clone, Debug, Default)]
pub struct CommonData {
    pub messenger: vk::DebugUtilsMessengerEXT,
    pub physical_device: vk::PhysicalDevice,
    pub surface: vk::SurfaceKHR,

    pub graphics_queue: vk::Queue,
    pub present_queue: vk::Queue,
    pub compute_queue: vk::Queue,

    pub mip_levels: u32,
}
