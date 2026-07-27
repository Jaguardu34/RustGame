use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

use rand::RngExt;

pub fn spawn_object(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let mut rng = rand::rng();
    commands.spawn((
        Mesh3d(meshes.add(Sphere::new(0.05))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::WHITE,
            perceptual_roughness: 1.0,
            ..default()
        })),
        Transform::from_xyz(
            rng.random_range(-1.0..1.0),
            5.0,
            rng.random_range(-1.0..1.0),
        ),
        RigidBody::Dynamic,
        Collider::ball(0.05),
        ActiveEvents::COLLISION_EVENTS,
    ));
}
