use bevy::render::mesh::Mesh;

use bevy::{camera_controller::free_camera::FreeCameraPlugin, prelude::*};
use bevy_inspector_egui::bevy_egui::EguiGlobalSettings;
use bevy_rapier3d::prelude::*;

pub mod editor;
pub mod game_var;
pub mod map_generator;

pub mod player;
use player::default_player;

pub mod input;
use input::PlayerInputPlugin;

use crate::editor::EditorPlugin;
use crate::game_var::GameVar;

use crate::load_chunks::ChunkPlugin;
use crate::player::PlayerPlugin;

pub mod player_ui;

use player_ui::UiPlugin;

pub mod free_camera;
use free_camera::FreeCamPlugin;

pub mod world_mesh;

pub mod chunk;

pub mod load_chunks;

fn main() {
    App::new()
        .init_resource::<GameVar>()
        .add_plugins(DefaultPlugins)
        .add_plugins(RapierPhysicsPlugin::<NoUserData>::default())
        .add_plugins(EditorPlugin)
        .add_plugins(FreeCameraPlugin)
        .add_plugins(ChunkPlugin)
        .add_plugins((PlayerPlugin, PlayerInputPlugin, UiPlugin, FreeCamPlugin))
        .add_systems(Startup, (setup_lights, setup_scene, setup_player).chain())
        .run();
}

fn setup_lights(mut commands: Commands) {
    commands.spawn((
        DirectionalLight {
            shadow_maps_enabled: true,
            illuminance: 600.0,
            ..default()
        },
        Transform::from_xyz(3.0, 3.0, 3.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));
    //commands.spawn(AmbientLight::default());
}

fn setup_scene(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(20.0, 1.0, 20.0))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::WHITE,
            perceptual_roughness: 1.0,
            ..default()
        })),
        Transform::from_xyz(0.0, -0.5, 0.0),
        RigidBody::Fixed,
        Collider::cuboid(10.0, 0.5, 10.0),
    ));

    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(0.2, 2.0, 10.0))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.7, 0.7, 0.8),
            perceptual_roughness: 0.05,
            metallic: 1.0,
            ..default()
        })),
        Transform::from_xyz(5.0, 1.0, -5.0),
        RigidBody::Fixed,
        Collider::cuboid(0.1, 1.0, 5.0),
    ));

    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(1.0, 1.0, 1.0))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgba(1.0, 0.0, 0.0, 0.1),
            emissive: LinearRgba {
                red: 2.0,
                green: 0.0,
                blue: 0.0,
                alpha: 1.0,
            },
            ..default()
        })),
        Transform::from_xyz(1.0, 0.5, -2.0),
        RigidBody::Fixed,
        Collider::cuboid(0.5, 0.5, 0.5),
    ));
}

fn setup_player(
    mut commands: Commands,
    meshes: ResMut<Assets<Mesh>>,
    materials: ResMut<Assets<StandardMaterial>>,
    mut egui_global_settings: ResMut<EguiGlobalSettings>,
) {
    egui_global_settings.auto_create_primary_context = false;

    commands.spawn(default_player(meshes, materials));
}
