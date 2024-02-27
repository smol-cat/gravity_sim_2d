use log::{info, warn};
use vulkanalia::vk::{self, DeviceV1_0, KhrSwapchainExtension};

use super::globals;

#[derive(Debug, Default)]
pub struct SwapchainData {
    pub swapchain: vk::SwapchainKHR,
    pub swapchain_images: Vec<vk::Image>,
    pub swapchain_image_views: Vec<vk::ImageView>,

    pub swapchain_format: vk::Format,
    pub swapchain_extent: vk::Extent2D,

    pub render_pass: vk::RenderPass,
    pub present_framebuffers: Vec<vk::Framebuffer>,
}

impl Drop for SwapchainData {
    fn drop(&mut self) {
        unsafe {
            info!("destroying swapchain data");
            if globals::get_device_opt().is_none() {
                return;
            }

            if globals::get_device().device_wait_idle().is_err() {
                warn!("destroying swapchain data failed");
                return;
            }

            self.present_framebuffers
                .iter()
                .for_each(|f| globals::get_device().destroy_framebuffer(*f, None));

            globals::get_device().destroy_render_pass(self.render_pass, None);
            self.swapchain_image_views
                .iter()
                .for_each(|v| globals::get_device().destroy_image_view(*v, None));
            globals::get_device().destroy_swapchain_khr(self.swapchain, None);
        }
    }
}
