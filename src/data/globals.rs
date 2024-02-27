use vulkanalia::prelude::v1_0::*;

pub const VALIDATION_ENABLED: bool = cfg!(debug_assertions);

pub const VALIDATION_LAYER: vk::ExtensionName =
    vk::ExtensionName::from_bytes(b"VK_LAYER_KHRONOS_validation");

pub const DEVICE_EXTENSIONS: &[vk::ExtensionName] = &[
    vk::KHR_SWAPCHAIN_EXTENSION.name,
    vk::KHR_SHADER_NON_SEMANTIC_INFO_EXTENSION.name,
];

pub const MIP_LEVEL_DOWNSAMLING: u32 = 3;
pub const MASS_FIELD_SIZE: u32 = MIP_LEVEL_DOWNSAMLING.pow(7);

pub const MAX_MIP_LEVELS: u32 = 12;
pub const MAX_FRAMES_IN_FLIGHT: usize = 2;
pub const SHADER_FORCE_REGION_RADIUS: u32 = 3;

static mut DEVICE: Option<Device> = None;

pub fn get_device() -> Device {
    unsafe { DEVICE.clone().unwrap() }
}

pub fn set_device(device: &Device) {
    unsafe {
        DEVICE = Some(device.clone());
    }
}
