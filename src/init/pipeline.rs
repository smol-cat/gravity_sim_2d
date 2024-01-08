use anyhow::{anyhow, Ok, Result};
use vulkanalia::prelude::v1_0::*;

use crate::data::{
    descriptors_data::DescriptorsData, pipeline_data::PipelineData, swapchain_data::SwapchainData,
    vertex::Vertex,
};

pub unsafe fn create_pipeline(
    device: &Device,
    swapchain: &SwapchainData,
    pipeline: &mut PipelineData,
) -> Result<()> {
    let vert = include_bytes!("../../shaders/vert.spv");
    let frag = include_bytes!("../../shaders/frag.spv");

    let vert_shader_module = create_shader_module(device, &vert[..])?;
    let frag_shader_module = create_shader_module(device, &frag[..])?;

    let vert_stage = vk::PipelineShaderStageCreateInfo::builder()
        .stage(vk::ShaderStageFlags::VERTEX)
        .module(vert_shader_module)
        .name(b"main\0");

    let frag_stage = vk::PipelineShaderStageCreateInfo::builder()
        .stage(vk::ShaderStageFlags::FRAGMENT)
        .module(frag_shader_module)
        .name(b"main\0");

    let binding_descriptions = &[Vertex::binding_description()];
    let attribute_descriptions = Vertex::attribute_descriptions();
    let vertex_input_state = vk::PipelineVertexInputStateCreateInfo::builder()
        .vertex_binding_descriptions(binding_descriptions)
        .vertex_attribute_descriptions(&attribute_descriptions);

    let input_assembly_state = vk::PipelineInputAssemblyStateCreateInfo::builder()
        .topology(vk::PrimitiveTopology::POINT_LIST)
        .primitive_restart_enable(false);

    let viewport = vk::Viewport::builder()
        .x(0.0)
        .y(0.0)
        .width(swapchain.swapchain_extent.width as f32)
        .height(swapchain.swapchain_extent.height as f32)
        .min_depth(0.0)
        .max_depth(1.0);

    let scissor = vk::Rect2D::builder()
        .offset(vk::Offset2D { x: 0, y: 0 })
        .extent(swapchain.swapchain_extent);

    let viewports = &[viewport];
    let scissors = &[scissor];
    let viewport_state = vk::PipelineViewportStateCreateInfo::builder()
        .viewports(viewports)
        .scissors(scissors);

    let rasterization_state = vk::PipelineRasterizationStateCreateInfo::builder()
        .depth_clamp_enable(false)
        .rasterizer_discard_enable(false)
        .polygon_mode(vk::PolygonMode::POINT)
        .line_width(1.0)
        .depth_bias_enable(false);

    let attachment = vk::PipelineColorBlendAttachmentState::builder()
        .color_write_mask(vk::ColorComponentFlags::all())
        .blend_enable(true)
        .src_color_blend_factor(vk::BlendFactor::SRC_ALPHA)
        .dst_color_blend_factor(vk::BlendFactor::ONE_MINUS_SRC_ALPHA)
        .color_blend_op(vk::BlendOp::ADD)
        .src_alpha_blend_factor(vk::BlendFactor::ONE)
        .dst_alpha_blend_factor(vk::BlendFactor::ZERO)
        .alpha_blend_op(vk::BlendOp::ADD);

    let attachments = &[attachment];
    let color_blend_state = vk::PipelineColorBlendStateCreateInfo::builder()
        .logic_op_enable(false)
        .logic_op(vk::LogicOp::COPY)
        .attachments(attachments)
        .blend_constants([0.0, 0.0, 0.0, 0.0]);

    let layout_info = vk::PipelineLayoutCreateInfo::builder();
    pipeline.pipeline_layout = device.create_pipeline_layout(&layout_info, None)?;

    let multisample_state = vk::PipelineMultisampleStateCreateInfo::builder()
        .rasterization_samples(vk::SampleCountFlags::_1)
        .sample_shading_enable(false);

    let stages = &[vert_stage, frag_stage];
    let info = vk::GraphicsPipelineCreateInfo::builder()
        .stages(stages)
        .vertex_input_state(&vertex_input_state)
        .input_assembly_state(&input_assembly_state)
        .viewport_state(&viewport_state)
        .rasterization_state(&rasterization_state)
        .multisample_state(&multisample_state)
        .color_blend_state(&color_blend_state)
        .layout(pipeline.mass_compute_pipeline_layout)
        .render_pass(pipeline.render_pass)
        .subpass(0);

    pipeline.pipeline = device
        .create_graphics_pipelines(vk::PipelineCache::null(), &[info], None)?
        .0[0];

    device.destroy_shader_module(vert_shader_module, None);
    device.destroy_shader_module(frag_shader_module, None);

    Ok(())
}

pub unsafe fn create_gravity_compute_pipeline(
    device: &Device,
    descriptors: &DescriptorsData,
    pipeline: &mut PipelineData,
) -> Result<()> {
    let comp = include_bytes!("../../shaders/comp.spv");
    let comp_shader_module = create_shader_module(device, &comp[..])?;
    let comp_stage = vk::PipelineShaderStageCreateInfo::builder()
        .stage(vk::ShaderStageFlags::COMPUTE)
        .module(comp_shader_module)
        .name(b"main\0");

    let set_layouts = &[descriptors.descriptor_set_layout];
    let layout_info = vk::PipelineLayoutCreateInfo::builder().set_layouts(set_layouts);

    pipeline.gravity_compute_pipeline_layout = device.create_pipeline_layout(&layout_info, None)?;

    let info = vk::ComputePipelineCreateInfo::builder()
        .stage(comp_stage)
        .layout(pipeline.gravity_compute_pipeline_layout);

    let infos = &[info];

    pipeline.gravity_compute_pipeline = device
        .create_compute_pipelines(vk::PipelineCache::null(), infos, None)?
        .0[0];

    device.destroy_shader_module(comp_shader_module, None);
    Ok(())
}

pub unsafe fn create_mass_compute_pipeline(
    device: &Device,
    swapchain: &SwapchainData,
    descriptors: &DescriptorsData,
    pipeline: &mut PipelineData,
) -> Result<()> {
    let comp = include_bytes!("../../shaders/mass.comp.spv");

    let comp_shader_module = create_shader_module(device, &comp[..])?;

    let comp_stage = vk::PipelineShaderStageCreateInfo::builder()
        .stage(vk::ShaderStageFlags::COMPUTE)
        .module(comp_shader_module)
        .name(b"main\0");

    let set_layouts = &[descriptors.descriptor_set_layout];
    let layout_info = vk::PipelineLayoutCreateInfo::builder().set_layouts(set_layouts);

    pipeline.mass_compute_pipeline_layout = device.create_pipeline_layout(&layout_info, None)?;

    let info = vk::ComputePipelineCreateInfo::builder()
        .stage(comp_stage)
        .layout(pipeline.gravity_compute_pipeline_layout);

    let infos = &[info];

    pipeline.gravity_compute_pipeline = device
        .create_compute_pipelines(vk::PipelineCache::null(), infos, None)?
        .0[0];

    device.destroy_shader_module(comp_shader_module, None);
    Ok(())
}

unsafe fn create_shader_module(device: &Device, bytecode: &[u8]) -> Result<vk::ShaderModule> {
    let bytecode = Vec::<u8>::from(bytecode);
    let (prefix, code, suffix) = bytecode.align_to::<u32>();
    if !prefix.is_empty() || !suffix.is_empty() {
        return Err(anyhow!("Shader bytecode is not properly aligned"));
    }

    let info = vk::ShaderModuleCreateInfo::builder()
        .code_size(bytecode.len())
        .code(code);

    Ok(device.create_shader_module(&info, None)?)
}
