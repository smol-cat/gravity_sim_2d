use log::{info, warn};
use vulkanalia::prelude::v1_0::*;

use crate::data::globals;

#[derive(Clone, Debug, Default)]
pub struct PipelineData {
    pub pipeline: vk::Pipeline,
    pub pipeline_layout: vk::PipelineLayout,
}

impl Drop for PipelineData {
    fn drop(&mut self) {
        unsafe {
            info!("destroying pipeline data");
            if globals::get_device().device_wait_idle().is_err() {
                warn!("destroying pipeline data failed");
                return;
            }

            globals::get_device().destroy_pipeline(self.pipeline, None);
            globals::get_device().destroy_pipeline_layout(self.pipeline_layout, None);
        }
    }
}
