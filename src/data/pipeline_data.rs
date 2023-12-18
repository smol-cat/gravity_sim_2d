use vulkanalia::prelude::v1_0::*;

#[derive(Clone, Debug, Default)]
pub struct PipelineData {
    pub render_pass: vk::RenderPass,
    pub pipeline: vk::Pipeline,
    pub pipeline_layout: vk::PipelineLayout,
    pub compute_pipeline: vk::Pipeline,
    pub compute_pipeline_layout: vk::PipelineLayout,

}
