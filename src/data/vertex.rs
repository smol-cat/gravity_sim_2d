use std::mem::size_of;
use vulkanalia::vk::{self, HasBuilder};

type Vec2 = cgmath::Vector2<f32>;

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct Vertex {
    pub pos: Vec2,
    pub velocity: Vec2,
}

impl Vertex {
    pub fn new(pos: Vec2, velocity: Vec2) -> Self {
        Self { pos, velocity }
    }

    pub fn binding_description() -> vk::VertexInputBindingDescription {
        vk::VertexInputBindingDescription::builder()
            .binding(0)
            .stride(size_of::<Vertex>() as u32)
            .input_rate(vk::VertexInputRate::VERTEX)
            .build()
    }

    pub fn attribute_descriptions() -> [vk::VertexInputAttributeDescription; 2] {
        let pos = vk::VertexInputAttributeDescription::builder()
            .binding(0)
            .location(0)
            .format(vk::Format::R32G32_SFLOAT)
            .offset(0)
            .build();

        let velocity = vk::VertexInputAttributeDescription::builder()
            .binding(0)
            .location(1)
            .format(vk::Format::R32G32_SFLOAT)
            .offset(size_of::<Vec2>() as u32)
            .build();

        [pos, velocity]
    }
}
