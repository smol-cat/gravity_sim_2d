use vulkanalia::prelude::v1_0::*;

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
