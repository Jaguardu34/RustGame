use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

#[derive(Component, Default)]
pub struct Player;

#[derive(Component)]
pub struct PlayerCamera;

#[derive(Component)]
pub struct PlayerFlashLight;

#[derive(Debug, Component, Deref, DerefMut)]
pub struct PlayerCameraSensitivity(Vec2);

impl Default for PlayerCameraSensitivity {
    fn default() -> Self {
        Self(Vec2::new(0.003, 0.002))
    }
}

pub fn default_player() -> impl Bundle {
    (
        Player,
        Transform::from_xyz(0.0, 2.0, 0.0),
        RigidBody::Dynamic,
        Collider::cuboid(0.4, 0.5, 0.4),
        Velocity::default(),
        GravityScale(1.0),
        LockedAxes::ROTATION_LOCKED,
        PlayerCameraSensitivity::default(),
        children![
            (
                Camera3d::default(),
                Transform::from_xyz(0.0, 0.5, 0.0),
                PlayerCamera,
                Projection::from(PerspectiveProjection {
                    fov: 90.0_f32.to_radians(),
                    ..default()
                }),
            ),
            (
                PointLight::default(),
                Transform::from_xyz(0.0, 0.55, 0.0),
                PlayerFlashLight
            )
        ],
    )
}
