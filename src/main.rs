use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

pub mod player;
use player::PlayerVar;
use player::default_player;

pub mod player_input;
use player_input::PlayerInputPlugin;

use crate::player::PlayerPlugin;

pub mod spawn_objects;

pub mod ui;
use ui::UiPlugin;

#[derive(Resource, Default)]
pub struct GameState {
    pub mouse_grabbed: bool,
}

fn main() {
    App::new()
        .init_resource::<GameState>()
        .init_resource::<PlayerVar>()
        .add_plugins(DefaultPlugins)
        .add_plugins(RapierPhysicsPlugin::<NoUserData>::default())
        .add_plugins((PlayerPlugin, PlayerInputPlugin, UiPlugin))
        .add_systems(Startup, (setup_lights, setup_scene, setup_player))
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
    commands.spawn(AmbientLight::default());
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

fn setup_player(mut commands: Commands) {
    commands.spawn(default_player());
}
