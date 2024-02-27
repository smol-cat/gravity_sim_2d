use log::{info, warn};
use vulkanalia::vk::{self, DeviceV1_0};

use super::globals;

#[derive(Debug, Default)]
pub struct ImageData {
    pub image: vk::Image,
    pub image_view: vk::ImageView,
    pub image_memory: vk::DeviceMemory,
}

impl Drop for ImageData {
    fn drop(&mut self) {
        unsafe {
            info!("destroying image");
            if globals::get_device().device_wait_idle().is_err() {
                warn!("destroying image failed");
                return;
            }

            globals::get_device().destroy_image(self.image, None);
            globals::get_device().destroy_image_view(self.image_view, None);
            globals::get_device().free_memory(self.image_memory, None);
        }
    }
}
