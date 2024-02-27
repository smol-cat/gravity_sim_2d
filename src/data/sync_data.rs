use log::{info, warn};
use vulkanalia::prelude::v1_0::*;

use crate::data::globals;

#[derive(Clone, Debug, Default)]
pub struct SyncData {
    pub image_available_semaphores: Vec<vk::Semaphore>,
    pub image_clear_finished_semaphores: Vec<vk::Semaphore>,
    pub render_finished_semaphores: Vec<vk::Semaphore>,

    pub gravity_read_finished_semaphores: Vec<vk::Semaphore>,
    pub mass_compute_finished_semaphores: Vec<vk::Semaphore>,

    pub gravity_compute_finished_semaphores: Vec<vk::Semaphore>,

    pub in_flight_fences: Vec<vk::Fence>,
    pub images_in_flight: Vec<vk::Fence>,

    pub first_gravity_compute: bool,
}

impl Drop for SyncData {
    fn drop(&mut self) {
        unsafe {
            info!("destroying sync data");
            if globals::get_device().device_wait_idle().is_err() {
                warn!("destroying sync data failed");
                return;
            }

            self.image_available_semaphores
                .iter()
                .for_each(|s| globals::get_device().destroy_semaphore(*s, None));
            self.image_clear_finished_semaphores
                .iter()
                .for_each(|s| globals::get_device().destroy_semaphore(*s, None));

            self.gravity_read_finished_semaphores
                .iter()
                .for_each(|s| globals::get_device().destroy_semaphore(*s, None));
            self.mass_compute_finished_semaphores
                .iter()
                .for_each(|s| globals::get_device().destroy_semaphore(*s, None));
            self.gravity_compute_finished_semaphores
                .iter()
                .for_each(|s| globals::get_device().destroy_semaphore(*s, None));

            self.render_finished_semaphores
                .iter()
                .for_each(|s| globals::get_device().destroy_semaphore(*s, None));

            self.in_flight_fences
                .iter()
                .for_each(|f| globals::get_device().destroy_fence(*f, None));
        }
    }
}
