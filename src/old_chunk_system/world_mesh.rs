use bevy::{
    asset::RenderAssetUsages,
    mesh::Mesh,
    prelude::*,
    render::mesh::{Indices, PrimitiveTopology},
};

use crate::map_generator::MapGenerator;

pub fn generate_terrain_mesh(
    perlin_pos: Vec2,
    size: u32,
    resolution: u32,
    height_mult: f64,
    map_gen: &MapGenerator,
) -> Mesh {
    let vertices_per_side = resolution + 1;
    let step = size as f32 / resolution as f32; // world-space distance between vertices

    let mut positions = Vec::new();
    let mut normals = Vec::new();
    let mut uvs = Vec::new();
    let mut indices = Vec::new();
    let pos_x = perlin_pos.x as i64;
    let pos_y = perlin_pos.y as i64;

    for z in 0..vertices_per_side {
        for x in 0..vertices_per_side {
            let world_x = x as f32 * step;
            let world_z = z as f32 * step;

            let height =
                map_gen.get_height(pos_x + world_x as i64, pos_y + world_z as i64) * height_mult;

            positions.push([world_x, height as f32, world_z]);
            normals.push([0.0, 1.0, 0.0]);
            uvs.push([world_x / size as f32, world_z / size as f32]);
        }
    }

    for z in 0..resolution {
        for x in 0..resolution {
            let i = z * vertices_per_side + x;
            indices.push(i);
            indices.push(i + vertices_per_side);
            indices.push(i + 1);
            indices.push(i + 1);
            indices.push(i + vertices_per_side);
            indices.push(i + vertices_per_side + 1);
        }
    }

    let mut mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    );
    let colors: Vec<[f32; 4]> = positions.iter().map(|p| height_to_color(p[1])).collect();
    mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, colors);
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    mesh.insert_indices(Indices::U32(indices));
    mesh.compute_normals();
    mesh
}

fn height_to_color(y: f32) -> [f32; 4] {
    if y < 8.0 {
        [0.8, 0.7, 0.3, 1.0] // sable
    } else if y < 20.0 {
        [0.2, 0.6, 0.2, 1.0] // herbe
    } else {
        [0.9, 0.9, 0.9, 1.0] // neige / sommets
    }
}
