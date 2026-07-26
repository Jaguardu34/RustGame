use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

pub mod player;
use player::default_player;

pub mod player_input;
use player_input::PlayerInputPlugin;

#[derive(Resource, Default)]
pub struct GameState {
    pub mouse_grabbed: bool,
}

fn main() {
    App::new()
        .init_resource::<GameState>()
        .add_plugins(DefaultPlugins)
        .add_plugins(RapierPhysicsPlugin::<NoUserData>::default())
        .add_plugins(PlayerInputPlugin)
        .add_systems(Startup, (setup_lights, setup_scene, setup_player))
        .run();
}

fn setup_lights(mut commands: Commands) {
    commands.spawn(AmbientLight {
        affects_lightmapped_meshes: true,
        ..default()
    });
}

fn setup_scene(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.spawn((
        Mesh3d(meshes.add(Plane3d::default().mesh().size(20.0, 20.0))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::WHITE,
            perceptual_roughness: 1.0,
            ..default()
        })),
        RigidBody::Fixed,
        Collider::cuboid(10.0, 0.1, 10.0),
    ));

    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(1.0, 1.0, 1.0))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::WHITE,
            perceptual_roughness: 1.0,
            ..default()
        })),
        Transform::from_xyz(5.0, 1.0, 5.0),
        RigidBody::Dynamic,
        Collider::cuboid(0.5, 0.5, 0.5),
    ));
}

fn setup_player(mut commands: Commands) {
    commands.spawn(default_player());
}
