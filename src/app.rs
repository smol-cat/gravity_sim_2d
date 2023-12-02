use anyhow::{anyhow, Ok, Result};
use log::info;

use cgmath::{point3, vec3, Deg};
use std::mem::size_of;
use std::ptr::copy_nonoverlapping as memcpy;
use std::time::Instant;
use thiserror::Error;
use vulkanalia::loader::{LibloadingLoader, LIBRARY};
use vulkanalia::prelude::v1_0::*;
use vulkanalia::window as vk_window;
use winit::window::Window;

use vulkanalia::vk::ExtDebugUtilsExtension;
use vulkanalia::vk::KhrSurfaceExtension;
use vulkanalia::vk::KhrSwapchainExtension;

use crate::data::buffers_data::BuffersData;
use crate::data::commands_data::CommandsData;
use crate::data::globals;
use crate::data::pipeline_data::PipelineData;
use crate::data::swapchain_data::SwapchainData;
use crate::data::sync_data::SyncData;
use crate::data::vertex::Vertex;
use crate::generators::random_generator;
use crate::init::{buffers, commands, framebuffers, pipeline, render_pass, swapchain, sync};
use crate::utils::queue_family_indices;
use crate::{
    data::common_data::CommonData,
    init::{device, instance},
};

#[derive(Clone, Debug)]
pub struct App {
    instance: Instance,
    pub device: Device,
    frame: usize,
    pub resized: bool,
    start: Instant,

    entry: Entry,
    buffers: BuffersData,
    common: CommonData,
    commands: CommandsData,
    pipeline: PipelineData,
    swapchain: SwapchainData,
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
        let mut pipeline = PipelineData::default();
        let mut swapchain = SwapchainData::default();
        let mut sync = SyncData::default();

        let instance = instance::create_instance(window, &entry, &mut common)?;
        common.surface = vk_window::create_surface(&instance, &window, &window)?;

        device::pick_physical_device(&instance, &mut common)?;
        let device = device::create_logical_device(&instance, &mut common)?;

        swapchain::create_swapchain(&device, &common, &instance, &window, &mut swapchain)?;
        swapchain.swapchain_image_views =
            swapchain::create_swapchain_image_views(&device, &swapchain)?;

        pipeline.render_pass = render_pass::create_render_pass(&device, &swapchain)?;
        pipeline::create_pipeline(&device, &swapchain, &mut pipeline)?;

        commands::create_command_pools(&instance, &device, &common, &swapchain, &mut commands)?;
        swapchain.framebuffers =
            framebuffers::create_framebuffers(&device, pipeline.render_pass, &swapchain)?;

        let vertices = random_generator::generate_vertices(1000);
        (buffers.vertex_buffer, buffers.vertex_buffer_memory) =
            buffers::create_vertex_buffer(&instance, &device, &vertices, &common, &commands)?;

        commands.command_buffers =
            commands::create_command_buffers(&device, &swapchain, commands.main_command_pool)?;

        sync::create_sync_objects(&device, &swapchain, &mut sync)?;
        let _self = Self {
            entry,
            instance,
            device,
            frame: 0,
            resized: false,
            start: Instant::now(),
            buffers,
            common,
            commands,
            pipeline,
            swapchain,
            sync,
            vertices,
        };

        Ok(_self)
    }

    pub unsafe fn render(&mut self, window: &Window) -> Result<()> {
        self.device.wait_for_fences(
            &[self.sync.in_flight_fences[self.frame]],
            true,
            u64::max_value(),
        )?;

        let result: Result<(u32, vk::SuccessCode), vk::ErrorCode> =
            self.device.acquire_next_image_khr(
                self.swapchain.swapchain,
                u64::max_value(),
                self.sync.image_available_semaphores[self.frame],
                vk::Fence::null(),
            );

        let image_index = match result {
            Result::Ok((image_index, _)) => image_index as usize,
            //Err(vk::ErrorCode::OUT_OF_DATE_KHR) => return self.recreate_swapchain(window),
            Err(e) => return Err(anyhow!(e)),
        };

        if !self.sync.images_in_flight[image_index as usize].is_null() {
            self.device.wait_for_fences(
                &[self.sync.images_in_flight[image_index as usize]],
                true,
                u64::max_value(),
            )?;
        }

        self.sync.images_in_flight[image_index as usize] = self.sync.in_flight_fences[self.frame];
        self.update_command_buffer(image_index)?;

        let wait_semaphores = &[self.sync.image_available_semaphores[self.frame]];
        let wait_stages = &[vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT];
        let command_buffers = &[self.commands.command_buffers[image_index as usize]];
        let signal_semaphores = &[self.sync.render_finished_semaphores[self.frame]];

        let submit_info = vk::SubmitInfo::builder()
            .wait_semaphores(wait_semaphores)
            .wait_dst_stage_mask(wait_stages)
            .command_buffers(command_buffers)
            .signal_semaphores(signal_semaphores);

        self.device
            .reset_fences(&[self.sync.in_flight_fences[self.frame]])?;

        self.device.queue_submit(
            self.common.graphics_queue,
            &[submit_info],
            self.sync.in_flight_fences[self.frame],
        )?;

        let swapchains = &[self.swapchain.swapchain];
        let image_indices = &[image_index as u32];
        let present_info = vk::PresentInfoKHR::builder()
            .wait_semaphores(signal_semaphores)
            .swapchains(swapchains)
            .image_indices(image_indices);

        let result = self
            .device
            .queue_present_khr(self.common.present_queue, &present_info);

        let changed = result == Result::Ok(vk::SuccessCode::SUBOPTIMAL_KHR)
            || result == Err(vk::ErrorCode::OUT_OF_DATE_KHR);

        if self.resized || changed {
            self.resized = false;
            self.recreate_swapchain(window)?;
        } else if let Err(e) = result {
            return Err(anyhow!(e));
        }

        self.frame = (self.frame + 1) % globals::MAX_FRAMES_IN_FLIGHT;

        Ok(())
    }

    pub unsafe fn recreate_swapchain(&mut self, window: &Window) -> Result<()> {
        self.device.device_wait_idle()?;
        self.destroy_swapchain();
        swapchain::create_swapchain(
            &self.device,
            &self.common,
            &self.instance,
            window,
            &mut self.swapchain,
        )?;

        self.swapchain.swapchain_image_views =
            swapchain::create_swapchain_image_views(&self.device, &mut self.swapchain)?;
        self.pipeline.render_pass = render_pass::create_render_pass(&self.device, &self.swapchain)?;

        pipeline::create_pipeline(&self.device, &self.swapchain, &mut self.pipeline)?;

        self.swapchain.framebuffers = framebuffers::create_framebuffers(
            &self.device,
            self.pipeline.render_pass,
            &mut self.swapchain,
        )?;

        self.commands.command_buffers = commands::create_command_buffers(
            &self.device,
            &self.swapchain,
            self.commands.main_command_pool,
        )?;

        self.sync
            .images_in_flight
            .resize(self.swapchain.swapchain_images.len(), vk::Fence::null());

        Ok(())
    }

    unsafe fn update_command_buffer(&mut self, image_index: usize) -> Result<()> {
        let command_pool = self.commands.command_pools[image_index];
        self.device
            .reset_command_pool(command_pool, vk::CommandPoolResetFlags::empty())?;

        let command_buffer = self.commands.command_buffers[image_index];

        let inheritance = vk::CommandBufferInheritanceInfo::builder();
        let info = vk::CommandBufferBeginInfo::builder()
            .flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT)
            .inheritance_info(&inheritance);

        self.device.begin_command_buffer(command_buffer, &info)?;

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
            .render_pass(self.pipeline.render_pass)
            .framebuffer(self.swapchain.framebuffers[image_index])
            .render_area(render_area)
            .clear_values(clear_values);

        self.device
            .cmd_begin_render_pass(command_buffer, &info, vk::SubpassContents::INLINE);

        self.device.cmd_bind_pipeline(
            command_buffer,
            vk::PipelineBindPoint::GRAPHICS,
            self.pipeline.pipeline,
        );

        self.device
            .cmd_bind_vertex_buffers(command_buffer, 0, &[self.buffers.vertex_buffer], &[0]);

        self.device
            .cmd_draw(command_buffer, self.vertices.len() as u32, 1, 0, 0);

        self.device.cmd_end_render_pass(command_buffer);
        self.device.end_command_buffer(command_buffer)?;

        Ok(())
    }

    pub unsafe fn destroy(&self) {
        self.device.device_wait_idle().unwrap();

        self.destroy_swapchain();
        self.commands
            .command_pools
            .iter()
            .for_each(|cp| self.device.destroy_command_pool(*cp, None));

        self.sync
            .in_flight_fences
            .iter()
            .for_each(|f| self.device.destroy_fence(*f, None));
        self.sync
            .render_finished_semaphores
            .iter()
            .for_each(|s| self.device.destroy_semaphore(*s, None));
        self.sync
            .image_available_semaphores
            .iter()
            .for_each(|s| self.device.destroy_semaphore(*s, None));

        self.device.destroy_buffer(self.buffers.vertex_buffer, None);
        self.device
            .free_memory(self.buffers.vertex_buffer_memory, None);

        self.device
            .destroy_command_pool(self.commands.main_command_pool, None);

        self.device.destroy_device(None);
        self.instance.destroy_surface_khr(self.common.surface, None);

        if globals::VALIDATION_ENABLED {
            self.instance
                .destroy_debug_utils_messenger_ext(self.common.messenger, None);
        }

        self.instance.destroy_instance(None);
    }

    pub unsafe fn destroy_swapchain(&self) {
        self.swapchain
            .framebuffers
            .iter()
            .for_each(|f| self.device.destroy_framebuffer(*f, None));

        self.device.destroy_pipeline(self.pipeline.pipeline, None);
        self.device
            .destroy_pipeline_layout(self.pipeline.pipeline_layout, None);

        self.device
            .destroy_render_pass(self.pipeline.render_pass, None);
        self.swapchain
            .swapchain_image_views
            .iter()
            .for_each(|v| self.device.destroy_image_view(*v, None));
        self.device
            .destroy_swapchain_khr(self.swapchain.swapchain, None);
    }
}
