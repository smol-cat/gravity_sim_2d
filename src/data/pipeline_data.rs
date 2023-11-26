use vulkanalia::prelude::v1_0::*;

#[derive(Clone, Debug, Default)]
pub struct PipelineData {
    pub render_pass: vk::RenderPass,
    pub pipeline_layout: vk::PipelineLayout,
    pub pipeline: vk::Pipeline,
}
