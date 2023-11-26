use anyhow::Result;
use app::App;
use vulkanalia::prelude::v1_0::*;
use winit::{
    dpi::LogicalSize,
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

mod app;
mod data;
mod generators;
mod init;
mod utils;

fn main() -> Result<()> {
    pretty_env_logger::init();

    let event_loop = EventLoop::new();
    let mut destroying = false;
    let window = WindowBuilder::new()
        .with_inner_size(LogicalSize::new(1024, 768))
        .build(&event_loop)?;

    let mut app = unsafe { App::create(&window)? };
    event_loop.run(move |event, _, control_flow| {
        match event {
            Event::MainEventsCleared if !destroying => unsafe {
                app.render().unwrap();
            },
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => {
                destroying = true;
                *control_flow = ControlFlow::Exit;
                unsafe {
                    app.device.device_wait_idle().unwrap();
                    app.destroy();
                }
            }
            _ => {}
        };
    });
}
