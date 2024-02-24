use vulkanalia::vk;

pub const VALIDATION_ENABLED: bool = cfg!(debug_assertions);

pub const VALIDATION_LAYER: vk::ExtensionName =
    vk::ExtensionName::from_bytes(b"VK_LAYER_KHRONOS_validation");

pub const DEVICE_EXTENSIONS: &[vk::ExtensionName] = &[
    vk::KHR_SWAPCHAIN_EXTENSION.name,
    vk::KHR_SHADER_NON_SEMANTIC_INFO_EXTENSION.name,
];

pub const MAX_FRAMES_IN_FLIGHT: usize = 2;
pub const MAX_MIP_LEVELS: u32 = 12;
pub const MIP_LEVEL_DOWNSAMLING: u32 = 7;
