use vulkanalia::prelude::v1_0::*;

#[derive(Clone, Debug, Default)]
pub struct PipelineData {
    pub render_pass: vk::RenderPass,
    pub pipeline: vk::Pipeline,
    pub pipeline_layout: vk::PipelineLayout,

    pub gravity_compute_pipeline: vk::Pipeline,
    pub gravity_compute_pipeline_layout: vk::PipelineLayout,

    pub mass_render_pass: vk::RenderPass,
    pub mass_compute_pipeline: vk::Pipeline,
    pub mass_compute_pipeline_layout: vk::PipelineLayout,
}
