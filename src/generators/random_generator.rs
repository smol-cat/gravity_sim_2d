use crate::data::vertex::Vertex;
use cgmath::vec2;
use rand::Rng;

#[allow(dead_code)]
pub fn generate_random_vertices(count: u32) -> Vec<Vertex> {
    let mut vertices: Vec<Vertex> = vec![];
    let mut rng = rand::thread_rng();
    for _ in 0..count {
        vertices.push(Vertex::new(
            vec2(rng.gen_range(-0.3..-0.1), rng.gen_range(-0.3..-0.1)),
            vec2(-0.0, -0.0),
            //vec2(rng.gen_range(-0.01..0.01), rng.gen_range(-0.01..0.01)),
        ));
    }


    vertices
}

#[allow(dead_code)]
pub fn generate_vertices() -> Vec<Vertex> {
    let mut vertices: Vec<Vertex> = vec![];
    for i in 0..2 {
        vertices.push(Vertex::new(
            vec2(i as f32 * 0.09 + 0.1, 0.0),
            vec2(0.0, 0.0),
        ));
    }

    vertices
}
