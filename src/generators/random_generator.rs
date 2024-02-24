use crate::data::vertex::Vertex;
use cgmath::{num_traits::Pow, vec2};
use rand::Rng;

#[allow(dead_code)]
pub fn generate_random_vertices(count: u32) -> Vec<Vertex> {
    let mut vertices: Vec<Vertex> = vec![];
    let mut rng = rand::thread_rng();
    for _ in 0..count {
        vertices.push(Vertex::new(
            vec2(rng.gen_range(-1.0..1.0), rng.gen_range(-1.0..1.0)),
            vec2(rng.gen_range(-0.001..0.001), rng.gen_range(-0.001..0.001)),
        ));
    }

    vertices
}

#[allow(dead_code)]
pub fn generate_two_clusters(count: u32) -> Vec<Vertex> {
    let mut vertices: Vec<Vertex> = vec![];
    let mut rng = rand::thread_rng();

    for _ in 0..(count / 2) {
        vertices.push(Vertex::new(
            vec2(rng.gen_range(-0.8..-0.3), rng.gen_range(-0.5..0.5)),
            vec2(rng.gen_range(-0.008..0.008), rng.gen_range(-0.008..0.008)),
        ));
    }

    for _ in 0..(count / 2) {
        vertices.push(Vertex::new(
            vec2(rng.gen_range(0.3..0.8), rng.gen_range(-0.5..0.5)),
            vec2(rng.gen_range(-0.008..0.008), rng.gen_range(-0.008..0.008)),
        ));
    }

    vertices
}

fn get_circle_pos(radius: f32, x: f32, offset: f32) -> f32 {
    let pos: f32 = radius.pow(2) - (x - offset).pow(2);
    pos.sqrt()
}

#[allow(dead_code)]
pub fn generate_circular_cluster(count: u32, radius: f32, thickness: f32) -> Vec<Vertex> {
    let mut vertices: Vec<Vertex> = vec![];
    let mut rng = rand::thread_rng();
    let offset = 0.0;

    for _ in 0..count {
        let adjusted_radius = rng.gen_range(radius..(radius + thickness));
        let x = rng.gen_range(-adjusted_radius..adjusted_radius) + offset;
        let factor = if rng.gen_bool(0.5) { 1 } else { -1 };

        vertices.push(Vertex::new(
            vec2(
                x,
                factor as f32 * get_circle_pos(adjusted_radius, x, offset),
            ),
            //vec2(rng.gen_range(-0.008..0.008), rng.gen_range(-0.008..0.008)),
            vec2(0.0, 0.0),
        ));
    }

    vertices
}

pub fn generate_2_circular_clusters(count: u32, radius: f32, thickness: f32) -> Vec<Vertex> {
    let mut vertices: Vec<Vertex> = vec![];
    let mut rng = rand::thread_rng();
    let offset = -0.5;

    for _ in 0..(count / 2) {
        let adjusted_radius = rng.gen_range(radius..(radius + thickness));
        let x = rng.gen_range(-adjusted_radius..adjusted_radius) + offset;
        let factor = if rng.gen_bool(0.5) { 1 } else { -1 };

        vertices.push(Vertex::new(
            vec2(x, factor as f32 * get_circle_pos(adjusted_radius, x, -0.2)),
            vec2(rng.gen_range(-0.008..0.008), rng.gen_range(-0.008..0.008)),
        ));
    }

    let offset = 0.5;
    for _ in 0..(count / 2) {
        let adjusted_radius = rng.gen_range(radius..(radius + thickness));
        let x = rng.gen_range(-adjusted_radius..adjusted_radius) + offset;
        let factor = if rng.gen_bool(0.5) { 1 } else { -1 };

        vertices.push(Vertex::new(
            vec2(x, factor as f32 * get_circle_pos(adjusted_radius, x, 0.2)),
            vec2(rng.gen_range(-0.008..0.008), rng.gen_range(-0.008..0.008)),
        ));
    }

    vertices
}
