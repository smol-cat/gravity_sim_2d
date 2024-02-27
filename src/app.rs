use anyhow::{anyhow, Ok, Result};
use log::info;

use std::mem::size_of;
use std::ptr::copy_nonoverlapping as memcpy;
use std::time::Instant;
use vulkanalia::loader::{LibloadingLoader, LIBRARY};
use vulkanalia::prelude::v1_0::*;
use vulkanalia::window as vk_window;
use winit::window::Window;

use vulkanalia::vk::KhrSwapchainExtension;
use vulkanalia::vk::{DeviceFaultInfoEXT, KhrSurfaceExtension};
use vulkanalia::vk::{ExtDebugUtilsExtension, Fence};

use crate::data::buffers_data::BuffersData;
use crate::data::commands_data::CommandsData;
use crate::data::descriptors_data::DescriptorsData;
use crate::data::globals;
use crate::data::pipeline_data::PipelineData;
use crate::data::swapchain_data::SwapchainData;
use crate::data::sync_data::SyncData;
use crate::data::uniform_buffer_object::UniformBufferObject;
use crate::data::vertex::Vertex;
use crate::generators::random_generator;
use crate::init::{buffers, commands, descriptors, framebuffers, pipeline, swapchain, sync};
use crate::{
    data::common_data::CommonData,
    init::{device, instance},
};

#[derive(Debug)]
pub struct App {
    instance: Instance,
    frame: usize,
    pub resized: bool,
    prev_duration: f32,
    start: Instant,

    _entry: Entry,
    buffers: BuffersData,
    common: CommonData,
    commands: CommandsData,
    render_pipeline: PipelineData,
    mass_pipeline: PipelineData,
    gravity_pipeline: PipelineData,
    swapchain: SwapchainData,
    gravity_descriptors: DescriptorsData,
    mass_descriptors: DescriptorsData,
    sync: SyncData,

    vertices: Vec<Vertex>,
}

impl App {
    pub unsafe fn create(window: &Window) -> Result<Self> {
        let loader = LibloadingLoader::new(LIBRARY)?;
        let entry = Entry::new(loader).map_err(|b| anyhow!("{}", b))?;

        let mut buffers = BuffersData::default();
        let mut common = CommonData::default();
        let mut commands = CommandsData::default();
        let mut render_pipeline = PipelineData::default();
        let mut mass_pipeline = PipelineData::default();
        let mut gravity_pipeline = PipelineData::default();
        let mut swapchain = SwapchainData::default();
        let mut sync = SyncData::default();
        let mut gravity_descriptors = DescriptorsData::default();
        let mut mass_descriptors = DescriptorsData::default();

        let instance = instance::create_instance(window, &entry, &mut common)?;
        common.surface = vk_window::create_surface(&instance, &window, &window)?;

        device::pick_physical_device(&instance, &mut common)?;
        let device = device::create_logical_device(&instance, &mut common)?;
        globals::set_device(&device);

        swapchain::create_swapchain(&common, &instance, &window, &mut swapchain)?;
        swapchain.swapchain_image_views = swapchain::create_swapchain_image_views(&swapchain)?;

        commands.main_command_pool = commands::create_command_pool(&instance, &common)?;
        buffers.offscreen_images = buffers::create_offscreen_images(&instance, &common, &commands)?;

        // Render passes
        swapchain.render_pass = swapchain::create_render_pass(swapchain.swapchain_format)?;

        // Descriptor layouts
        gravity_descriptors.descriptor_set_layout =
            descriptors::create_gravity_descriptor_set_layout()?;
        mass_descriptors.descriptor_set_layout = descriptors::create_mass_descriptor_set_layout()?;

        // Pipelines
        pipeline::create_mass_compute_pipeline(&mass_descriptors, &mut mass_pipeline)?;
        pipeline::create_gravity_compute_pipeline(&gravity_descriptors, &mut gravity_pipeline)?;
        pipeline::create_pipeline(&swapchain, &mut render_pipeline)?;

        gravity_descriptors.descriptor_pool =
            descriptors::create_gravity_descriptor_pool(&swapchain)?;
        mass_descriptors.descriptor_pool = descriptors::create_mass_descriptor_pool(&swapchain)?;

        swapchain.present_framebuffers = framebuffers::create_framebuffers(
            swapchain.render_pass,
            &swapchain.swapchain_extent,
            &swapchain.swapchain_image_views,
        )?;

        //let vertices = random_generator::generate_random_vertices(500000);
        //let vertices = random_generator::generate_two_clusters(2000000);
        //let vertices = random_generator::generate_circular_cluster(1000000, 0.5, 0.001);
        let vertices = random_generator::generate_2_circular_clusters(1000000, 0.2, 0.0001);

        buffers::create_uniform_buffers(&instance, &common, &swapchain, &mut buffers)?;

        buffers::create_shader_storage_buffers(
            &instance,
            &vertices,
            &common,
            &commands,
            &mut buffers,
        )?;

        descriptors::create_gravity_descriptor_sets(&buffers, &vertices, &mut gravity_descriptors)?;

        descriptors::create_mass_descriptor_sets(&buffers, &vertices, &mut mass_descriptors)?;

        commands.command_buffers = commands::create_command_buffers(
            swapchain.swapchain_images.len(),
            commands.main_command_pool,
        )?;

        commands.gravity_compute_command_buffers = commands::create_command_buffers(
            globals::MAX_FRAMES_IN_FLIGHT,
            commands.main_command_pool,
        )?;

        commands.mass_compute_command_buffers = commands::create_command_buffers(
            globals::MAX_FRAMES_IN_FLIGHT,
            commands.main_command_pool,
        )?;

        commands.image_clear_command_buffers = commands::create_command_buffers(
            globals::MAX_FRAMES_IN_FLIGHT,
            commands.main_command_pool,
        )?;

        sync::create_sync_objects(&swapchain, &mut sync)?;
        let _self = Self {
            gravity_descriptors,
            mass_descriptors,
            _entry: entry,
            instance,
            frame: 0,
            resized: false,
            prev_duration: 0.0,
            start: Instant::now(),
            buffers,
            common,
            commands,
            render_pipeline,
            mass_pipeline,
            gravity_pipeline,
            swapchain,
            sync,
            vertices,
        };

        Ok(_self)
    }

    pub unsafe fn render(&mut self, window: &Window) -> Result<()> {
        globals::get_device().wait_for_fences(
            &[self.sync.in_flight_fences[self.frame]],
            true,
            u64::max_value(),
        )?;

        let result: Result<(u32, vk::SuccessCode), vk::ErrorCode> = globals::get_device()
            .acquire_next_image_khr(
                self.swapchain.swapchain,
                u64::max_value(),
                self.sync.image_available_semaphores[self.frame],
                vk::Fence::null(),
            );

        let image_index = match result {
            Result::Ok((image_index, _)) => image_index as usize,
            Err(vk::ErrorCode::OUT_OF_DATE_KHR) => return self.recreate_swapchain(window),
            Err(e) => return Err(anyhow!(e)),
        };

        if !self.sync.images_in_flight[image_index as usize].is_null() {
            globals::get_device().wait_for_fences(
                &[self.sync.images_in_flight[image_index as usize]],
                true,
                u64::max_value(),
            )?;
        }

        self.sync.images_in_flight[image_index as usize] = self.sync.in_flight_fences[self.frame];
        globals::get_device().reset_fences(&[self.sync.in_flight_fences[self.frame]])?;

        self.update_mass_command_buffers()?;
        self.update_command_buffer(image_index)?;
        self.update_uniform_buffer(image_index)?;
        self.update_gravity_compute_command_buffers()?;
        self.update_clear_command_buffer()?;

        self.submit_image_clear()?;
        self.submit_mass_compute()?;
        self.submit_gravity_compute()?;

        self.submit_render(image_index)?;
        self.submit_present(window, image_index)?;

        self.frame = (self.frame + 1) % globals::MAX_FRAMES_IN_FLIGHT;

        Ok(())
    }

    unsafe fn submit_image_clear(&mut self) -> Result<()> {
        let command_buffers = &[self.commands.image_clear_command_buffers[self.frame]];
        let wait_semaphores = &[self.sync.image_available_semaphores[self.frame]];
        let wait_stages = &[vk::PipelineStageFlags::TOP_OF_PIPE];
        let signal_semaphores = &[self.sync.image_clear_finished_semaphores[self.frame]];

        let submit_info = vk::SubmitInfo::builder()
            .wait_semaphores(wait_semaphores)
            .wait_dst_stage_mask(wait_stages)
            .command_buffers(command_buffers)
            .signal_semaphores(signal_semaphores);

        globals::get_device().queue_submit(
            self.common.graphics_queue,
            &[submit_info],
            Fence::null(),
        )?;
        Ok(())
    }

    unsafe fn submit_mass_compute(&mut self) -> Result<()> {
        let command_buffers = &[self.commands.mass_compute_command_buffers[self.frame]];
        let mut wait_semaphores = vec![self.sync.image_clear_finished_semaphores[self.frame]];
        let mut wait_stages = vec![vk::PipelineStageFlags::COMPUTE_SHADER];
        if !self.sync.first_gravity_compute {
            wait_semaphores.push(
                self.sync.gravity_read_finished_semaphores
                    [(self.frame + 1) % globals::MAX_FRAMES_IN_FLIGHT],
            );

            wait_stages.push(vk::PipelineStageFlags::COMPUTE_SHADER)
        }

        let signal_semaphores = &[self.sync.mass_compute_finished_semaphores[self.frame]];

        let submit_info = vk::SubmitInfo::builder()
            .wait_semaphores(&wait_semaphores)
            .wait_dst_stage_mask(&wait_stages)
            .command_buffers(command_buffers)
            .signal_semaphores(signal_semaphores);

        globals::get_device().queue_submit(
            self.common.graphics_queue,
            &[submit_info],
            Fence::null(),
        )?;

        Ok(())
    }

    unsafe fn submit_gravity_compute(&mut self) -> Result<()> {
        let command_buffers = &[self.commands.gravity_compute_command_buffers[self.frame]];

        let wait_semaphores = &[self.sync.mass_compute_finished_semaphores[self.frame]];
        let wait_stages = &[vk::PipelineStageFlags::COMPUTE_SHADER];

        let signal_semaphores = &[
            self.sync.gravity_compute_finished_semaphores[self.frame],
            self.sync.gravity_read_finished_semaphores[self.frame],
        ];

        let compute_submit_info = vk::SubmitInfo::builder()
            .wait_semaphores(wait_semaphores)
            .wait_dst_stage_mask(wait_stages)
            .command_buffers(command_buffers)
            .signal_semaphores(signal_semaphores);

        globals::get_device().queue_submit(
            self.common.compute_queue,
            &[compute_submit_info],
            Fence::null(),
        )?;

        self.sync.first_gravity_compute = false;
        Ok(())
    }

    unsafe fn submit_render(&mut self, image_index: usize) -> Result<()> {
        let command_buffers = &[self.commands.command_buffers[image_index as usize]];
        let wait_semaphores = &[self.sync.gravity_compute_finished_semaphores[self.frame]];
        let wait_stages = &[vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT];
        let signal_semaphores = &[self.sync.render_finished_semaphores[self.frame]];

        let submit_info = vk::SubmitInfo::builder()
            .wait_semaphores(wait_semaphores)
            .wait_dst_stage_mask(wait_stages)
            .command_buffers(command_buffers)
            .signal_semaphores(signal_semaphores);

        globals::get_device().queue_submit(
            self.common.graphics_queue,
            &[submit_info],
            self.sync.in_flight_fences[self.frame],
        )?;

        Ok(())
    }

    unsafe fn submit_present(&mut self, window: &Window, image_index: usize) -> Result<()> {
        let swapchains = &[self.swapchain.swapchain];
        let image_indices = &[image_index as u32];

        let wait_semaphores = &[self.sync.render_finished_semaphores[self.frame]];
        let present_info = vk::PresentInfoKHR::builder()
            .wait_semaphores(wait_semaphores)
            .swapchains(swapchains)
            .image_indices(image_indices);

        let result =
            globals::get_device().queue_present_khr(self.common.present_queue, &present_info);

        let changed = result == Result::Ok(vk::SuccessCode::SUBOPTIMAL_KHR)
            || result == Err(vk::ErrorCode::OUT_OF_DATE_KHR);

        if self.resized || changed {
            self.resized = false;
            self.recreate_swapchain(window)?;
            self.update_gravity_compute_command_buffers()?;
        } else if let Err(e) = result {
            return Err(anyhow!(e));
        }

        Ok(())
    }

    pub unsafe fn recreate_swapchain(&mut self, window: &Window) -> Result<()> {
        globals::get_device().device_wait_idle()?;
        self.destroy_swapchain();

        self.swapchain = SwapchainData::default();
        swapchain::create_swapchain(&self.common, &self.instance, window, &mut self.swapchain)?;

        self.swapchain.swapchain_image_views =
            swapchain::create_swapchain_image_views(&mut self.swapchain)?;

        self.swapchain.render_pass =
            swapchain::create_render_pass(self.swapchain.swapchain_format)?;

        pipeline::create_pipeline(&self.swapchain, &mut self.render_pipeline)?;

        self.swapchain.present_framebuffers = framebuffers::create_framebuffers(
            self.swapchain.render_pass,
            &self.swapchain.swapchain_extent,
            &self.swapchain.swapchain_image_views,
        )?;

        self.commands.command_buffers = commands::create_command_buffers(
            self.swapchain.swapchain_images.len(),
            self.commands.main_command_pool,
        )?;

        self.sync
            .images_in_flight
            .resize(self.swapchain.swapchain_images.len(), vk::Fence::null());

        Ok(())
    }

    unsafe fn update_command_buffer(&mut self, image_index: usize) -> Result<()> {
        let command_buffer = self.commands.command_buffers[image_index];

        let info = vk::CommandBufferBeginInfo::builder()
            .flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT);
        globals::get_device().begin_command_buffer(command_buffer, &info)?;

        let render_area = vk::Rect2D::builder()
            .offset(vk::Offset2D::default())
            .extent(self.swapchain.swapchain_extent);

        let color_clear_value = vk::ClearValue {
            color: vk::ClearColorValue {
                float32: [0.0, 0.0, 0.0, 1.0],
            },
        };

        let clear_values = &[color_clear_value];

        let info = vk::RenderPassBeginInfo::builder()
            .render_pass(self.swapchain.render_pass)
            .framebuffer(self.swapchain.present_framebuffers[image_index])
            .render_area(render_area)
            .clear_values(clear_values);

        globals::get_device().cmd_begin_render_pass(
            command_buffer,
            &info,
            vk::SubpassContents::INLINE,
        );

        globals::get_device().cmd_bind_pipeline(
            command_buffer,
            vk::PipelineBindPoint::GRAPHICS,
            self.render_pipeline.pipeline,
        );

        globals::get_device().cmd_bind_vertex_buffers(
            command_buffer,
            0,
            &[self.buffers.storage_buffers[self.frame]],
            &[0],
        );

        globals::get_device().cmd_draw(command_buffer, self.vertices.len() as u32, 1, 0, 0);

        globals::get_device().cmd_end_render_pass(command_buffer);
        globals::get_device().end_command_buffer(command_buffer)?;

        Ok(())
    }

    unsafe fn update_clear_command_buffer(&mut self) -> Result<()> {
        let command_buffer = self.commands.image_clear_command_buffers[self.frame];

        let inheritance = vk::CommandBufferInheritanceInfo::builder();
        let info = vk::CommandBufferBeginInfo::builder()
            .flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT)
            .inheritance_info(&inheritance);

        globals::get_device().begin_command_buffer(command_buffer, &info)?;

        let clear_color = vk::ClearColorValue {
            float32: [0.0, 0.0, 0.0, 0.0],
        };

        let subresource = vk::ImageSubresourceRange::builder()
            .aspect_mask(vk::ImageAspectFlags::COLOR)
            .base_array_layer(0)
            .layer_count(1)
            .level_count(1);

        let subresources = &[subresource];

        self.buffers.offscreen_images[self.frame]
            .iter()
            .for_each(|i| {
                globals::get_device().cmd_clear_color_image(
                    command_buffer,
                    i.image,
                    vk::ImageLayout::GENERAL,
                    &clear_color,
                    subresources,
                );
            });

        globals::get_device().end_command_buffer(command_buffer)?;
        Ok(())
    }

    unsafe fn update_uniform_buffer(&mut self, image_index: usize) -> Result<()> {
        let curr_duration = self.start.elapsed().as_secs_f32();
        let delta = curr_duration - self.prev_duration;
        self.prev_duration = curr_duration;

        let ubo = UniformBufferObject { delta_t: delta };
        let memory = globals::get_device().map_memory(
            self.buffers.uniform_buffers_memory[image_index],
            0,
            size_of::<UniformBufferObject>() as u64,
            vk::MemoryMapFlags::empty(),
        )?;

        memcpy(&ubo, memory.cast(), 1);

        globals::get_device().unmap_memory(self.buffers.uniform_buffers_memory[image_index]);
        Ok(())
    }

    unsafe fn update_mass_command_buffers(&self) -> Result<()> {
        let command_buffer = self.commands.mass_compute_command_buffers[self.frame];

        let inheritance = vk::CommandBufferInheritanceInfo::builder();
        let info = vk::CommandBufferBeginInfo::builder()
            .flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT)
            .inheritance_info(&inheritance);

        globals::get_device().begin_command_buffer(command_buffer, &info)?;

        globals::get_device().cmd_bind_pipeline(
            command_buffer,
            vk::PipelineBindPoint::COMPUTE,
            self.mass_pipeline.pipeline,
        );

        let descriptor_sets = &[self.mass_descriptors.descriptor_sets[self.frame]];
        globals::get_device().cmd_bind_descriptor_sets(
            command_buffer,
            vk::PipelineBindPoint::COMPUTE,
            self.mass_pipeline.pipeline_layout,
            0,
            descriptor_sets,
            &[],
        );

        let detail_levels = self.buffers.offscreen_images[0].len() as u32;
        let mip_levels_bytes = &detail_levels.to_ne_bytes();

        globals::get_device().cmd_push_constants(
            command_buffer,
            self.mass_pipeline.pipeline_layout,
            vk::ShaderStageFlags::COMPUTE,
            0,
            mip_levels_bytes,
        );

        globals::get_device().cmd_dispatch(
            command_buffer,
            (self.vertices.len() as f32 / 256.0).ceil() as u32,
            1,
            1,
        );

        globals::get_device().end_command_buffer(command_buffer)?;
        Ok(())
    }

    unsafe fn update_gravity_compute_command_buffers(&mut self) -> Result<()> {
        let command_buffer = self.commands.gravity_compute_command_buffers[self.frame];

        let inheritance = vk::CommandBufferInheritanceInfo::builder();
        let info = vk::CommandBufferBeginInfo::builder()
            .flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT)
            .inheritance_info(&inheritance);

        globals::get_device().begin_command_buffer(command_buffer, &info)?;

        globals::get_device().cmd_bind_pipeline(
            command_buffer,
            vk::PipelineBindPoint::COMPUTE,
            self.gravity_pipeline.pipeline,
        );

        let descriptor_sets = &[self.gravity_descriptors.descriptor_sets[self.frame]];
        globals::get_device().cmd_bind_descriptor_sets(
            command_buffer,
            vk::PipelineBindPoint::COMPUTE,
            self.gravity_pipeline.pipeline_layout,
            0,
            descriptor_sets,
            &[],
        );

        let detail_levels = self.buffers.offscreen_images[0].len() as u32;
        let detail_levels_bytes = &detail_levels.to_ne_bytes();

        globals::get_device().cmd_push_constants(
            command_buffer,
            self.mass_pipeline.pipeline_layout,
            vk::ShaderStageFlags::COMPUTE,
            0,
            detail_levels_bytes,
        );

        globals::get_device().cmd_dispatch(
            command_buffer,
            (self.vertices.len() as f32 / 256.0).ceil() as u32,
            1,
            1,
        );

        globals::get_device().end_command_buffer(command_buffer)?;
        Ok(())
    }

    pub unsafe fn destroy(&mut self) {
        self.destroy_swapchain();

        self.commands = CommandsData::default();
        self.mass_pipeline = PipelineData::default();
        self.gravity_pipeline = PipelineData::default();
        self.buffers = BuffersData::default();
        self.sync = SyncData::default();
        self.gravity_descriptors = DescriptorsData::default();
        self.mass_descriptors = DescriptorsData::default();

        globals::get_device().destroy_device(None);
        self.instance.destroy_surface_khr(self.common.surface, None);

        if globals::VALIDATION_ENABLED {
            self.instance
                .destroy_debug_utils_messenger_ext(self.common.messenger, None);
        }

        self.instance.destroy_instance(None);
    }

    pub unsafe fn destroy_swapchain(&mut self) {
        self.render_pipeline = PipelineData::default();
        self.swapchain = SwapchainData::default();
    }
}
