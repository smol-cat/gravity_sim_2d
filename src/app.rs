use std::time::Instant;

use anyhow::{anyhow, Ok, Result};
use log::info;
use vulkanalia::vk::KhrSwapchainExtension;
use vulkanalia::window as vk_window;
use vulkanalia::{
    loader::{LibloadingLoader, LIBRARY},
    prelude::v1_0::*,
};
use winit::window::Window;

use crate::data::buffers_data::BuffersData;
use crate::data::commands_data::CommandsData;
use crate::data::globals;
use crate::data::pipeline_data::PipelineData;
use crate::data::swapchain_data::SwapchainData;
use crate::data::sync_data::SyncData;
use crate::data::vertex::Vertex;
use crate::generators::random_generator;
use crate::init::{buffers, commands, framebuffers, pipeline, render_pass, swapchain, sync};
use crate::{
    data::common_data::CommonData,
    init::{devices, instance},
};

pub struct App {
    pub device: Device,
    frame: usize,
    start: Instant,
    pub resized: bool,

    common: CommonData,
    swapchain: SwapchainData,
    pipeline: PipelineData,
    commands: CommandsData,
    buffers: BuffersData,
    sync: SyncData,

    vertices: Vec<Vertex>,
}

impl App {
    pub unsafe fn create(window: &Window) -> Result<Self> {
        let loader = LibloadingLoader::new(LIBRARY)?;
        let entry = Entry::new(loader).map_err(|b| anyhow!("{}", b))?;

        let mut common = CommonData::default();
        let mut swapchain = SwapchainData::default();
        let mut pipeline = PipelineData::default();
        let mut commands = CommandsData::default();
        let mut buffers = BuffersData::default();
        let mut sync = SyncData::default();

        let (instance, messenger) = instance::create_instance(&entry, window)?;
        common.messenger = messenger;
        common.surface = vk_window::create_surface(&instance, &window, &window)?;

        common.physical_device = devices::get_physical_device(&instance, common.surface)?;
        let device: Device = devices::create_logical_device(
            &instance,
            common.surface,
            common.physical_device,
            &mut common,
        )?;

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

        Ok(Self {
            device,
            frame: 0,
            start: Instant::now(),
            resized: false,
            common,
            swapchain,
            pipeline,
            commands,
            buffers,
            sync,
            vertices,
        })
    }

    pub unsafe fn render(&mut self) -> Result<()> {
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

        //if self.resized || changed {
        //self.resized = false;
        //self.recreate_swapchain(window)?;
        //} else if let Err(e) = result {
        //return Err(anyhow!(e));
        //}

        self.frame = (self.frame + 1) % globals::MAX_FRAMES_IN_FLIGHT;

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
        self.device.destroy_device(None);
    }

    pub unsafe fn destroy_swapchain(&self) {
        self.device
            .destroy_swapchain_khr(self.swapchain.swapchain, None);
        self.swapchain
            .swapchain_image_views
            .iter()
            .for_each(|v| self.device.destroy_image_view(*v, None));
    }
}
