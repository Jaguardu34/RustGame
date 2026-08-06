use std::collections::HashMap;
use std::collections::HashSet;

use bevy::prelude::*;

use crate::{
    chunk::Chunk, editor::EditorState, free_camera::FreeCam, game_var::GameVar,
    map_generator::MapGenerator, player::PlayerCamera,
};

#[derive(Resource, Default)]
pub struct ChunkMap(pub HashMap<IVec2, Chunk>);

#[derive(Message)]
pub struct RegenerateMapEvent;

#[derive(Resource)]
pub struct MapParams {
    pub seed: u32,
    pub scale: f64,
    pub lacunarity: f64,
    pub persistance: f64,
    pub height_mult: f64,
    pub octaves: u32,
}

impl Default for MapParams {
    fn default() -> Self {
        Self {
            seed: 1,
            scale: 0.001,
            lacunarity: 2.0,
            persistance: 0.5,
            height_mult: 60.0,
            octaves: 4,
        }
    }
}

#[derive(Resource)]
pub struct MapGen(pub MapGenerator);

impl FromWorld for MapGen {
    fn from_world(world: &mut World) -> Self {
        let settings = world.resource::<MapParams>();
        Self(MapGenerator::new(
            settings.seed,
            settings.scale,
            settings.lacunarity,
            settings.persistance,
            settings.octaves,
        ))
    }
}

pub struct ChunkPlugin;

impl Plugin for ChunkPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ChunkMap>()
            .init_resource::<MapParams>()
            .init_resource::<MapGen>()
            .add_message::<RegenerateMapEvent>()
            .add_systems(Startup, generate_map_on_spawn)
            .add_systems(Update, (load_unload_chunks, regenerate_map_observer));
    }
}

fn load_unload_chunks(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    game_var: Res<GameVar>,
    player_cam_query: Query<&GlobalTransform, (With<PlayerCamera>, Without<FreeCam>)>,
    free_cam_query: Query<&GlobalTransform, (With<FreeCam>, Without<PlayerCamera>)>,
    mut chunk_map: ResMut<ChunkMap>,
    mut last_player_pos: Local<IVec2>,
    map_gen: Res<MapGen>,
    editor_state: Res<EditorState>,
    map_params: Res<MapParams>,
    mut regenerate_events: MessageReader<RegenerateMapEvent>,
    mut last_camera_rotation: Local<Quat>,
) {
    let Ok(cam_transform) = (if editor_state.game_playing {
        player_cam_query.single()
    } else {
        free_cam_query.single()
    }) else {
        return;
    };

    let player_chunk_x = (cam_transform.translation().x / 16.0).floor();
    let player_chunk_z = (cam_transform.translation().z / 16.0).floor();

    let player_chunk_pos = IVec2 {
        x: player_chunk_x as i32,
        y: player_chunk_z as i32,
    };

    let force_reload = !regenerate_events.is_empty();
    regenerate_events.clear();

    if player_chunk_pos != *last_player_pos
        || force_reload
        || *last_camera_rotation != cam_transform.rotation()
    {
        let render_chunk = calculate_player_view(
            player_chunk_pos,
            game_var.render_distance as i32,
            cam_transform,
            120.0,
        );
        for &chunk_pos in &render_chunk {
            if let Some(chunk) = chunk_map.0.get_mut(&chunk_pos) {
                chunk.spawn(&mut commands, &mut meshes, &mut materials);
            } else {
                let mut current_chunk = Chunk::new(
                    chunk_pos,
                    map_params.height_mult,
                    &mut meshes,
                    &mut materials,
                    &map_gen.0,
                );
                current_chunk.spawn(&mut commands, &mut meshes, &mut materials);
                chunk_map.0.insert(chunk_pos, current_chunk);
            }
        }

        for (&pos, chunk) in chunk_map.0.iter_mut() {
            if !render_chunk.contains(&pos) {
                chunk.despawn(&mut commands);
            }
        }
    }

    *last_camera_rotation = cam_transform.rotation();
    *last_player_pos = player_chunk_pos;
}

fn calculate_player_view(
    player_chunk_pos: IVec2,
    render_distance: i32,
    camera_transform: &GlobalTransform,
    fov_degrees: f32,
) -> HashSet<IVec2> {
    let mut chunks = HashSet::with_capacity(((render_distance * 2 + 1) as usize).pow(2));
    let r2 = render_distance * render_distance;

    let forward = camera_transform.forward();
    let forward_2d = Vec2::new(forward.x, forward.z).normalize_or_zero();

    let half_fov_rad = (fov_degrees / 2.0).to_radians();

    let always_load_radius = 2;
    let always_load_r2 = always_load_radius * always_load_radius;

    for x in -render_distance..=render_distance {
        for z in -render_distance..=render_distance {
            let dist2 = x * x + z * z;
            if dist2 > r2 {
                continue;
            }

            if dist2 <= always_load_r2 {
                chunks.insert(IVec2::new(player_chunk_pos.x + x, player_chunk_pos.y + z));
                continue;
            }

            let chunk_dir = Vec2::new(x as f32, z as f32).normalize_or_zero();
            if chunk_dir == Vec2::ZERO {
                continue;
            }
            let dot = forward_2d.dot(chunk_dir).clamp(-1.0, 1.0);
            let angle = dot.acos();

            if angle <= half_fov_rad {
                chunks.insert(IVec2::new(player_chunk_pos.x + x, player_chunk_pos.y + z));
            }
        }
    }
    chunks
}

pub fn clear_chunk_cache(world: &mut World) {
    let entities: Vec<Entity> = {
        let mut chunk_map = world.get_resource_mut::<ChunkMap>().unwrap();
        let entities: Vec<Entity> = chunk_map
            .0
            .values_mut()
            .filter_map(|c| c.entity.take())
            .collect();
        chunk_map.0.clear();
        entities
    };

    for entity in entities {
        world.despawn(entity);
    }
}

fn regenerate_map_observer(
    game_var: Res<GameVar>,
    mut last_render_distance: Local<u32>,

    mut regenerate_writer: MessageWriter<RegenerateMapEvent>,
) {
    if *last_render_distance != game_var.render_distance {
        regenerate_writer.write(RegenerateMapEvent);
    }
    *last_render_distance = game_var.render_distance;
}

fn generate_map_on_spawn(mut regenerate_writer: MessageWriter<RegenerateMapEvent>) {
    regenerate_writer.write(RegenerateMapEvent);
}
