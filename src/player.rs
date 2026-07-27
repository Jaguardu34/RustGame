use bevy::{post_process::bloom::Bloom, prelude::*};
use bevy_rapier3d::{na::Scale3, prelude::*};

#[derive(Component, Default)]
pub struct Player;

#[derive(Component)]
pub struct PlayerCamera;

#[derive(Component)]
pub struct PlayerFlashLight;

#[derive(Debug, Component, Deref, DerefMut)]
pub struct PlayerCameraSensitivity(Vec2);

#[derive(Resource, Default)]
pub struct PlayerVar {
    pub flashlight: bool,
    pub sprinting: bool,
    pub crouching: bool,
    pub speed: f32,
}

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (update_flashlight, crouch));
    }
}

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
        ColliderMassProperties::Density(10.0),
        children![(
            Camera3d::default(),
            Transform::from_xyz(0.0, 0.5, 0.0),
            PlayerCamera,
            Projection::from(PerspectiveProjection {
                fov: 90.0_f32.to_radians(),
                ..default()
            }),
            Bloom::NATURAL,
            Msaa::Off,
            IsDefaultUiCamera,
            children![(
                SpotLight {
                    range: 30.0,
                    shadow_maps_enabled: true,
                    inner_angle: 0.4,
                    intensity: 1_000_000.0,
                    outer_angle: 0.6,
                    ..default()
                },
                Transform::from_xyz(0.0, 0.0, 0.0),
                PlayerFlashLight
            )]
        ),],
    )
}

fn update_flashlight(
    mut flashlight_query: Query<&mut SpotLight, With<PlayerFlashLight>>,
    player_var: Res<PlayerVar>,
) {
    let Ok(mut flashlight) = flashlight_query.single_mut() else {
        return;
    };

    if player_var.flashlight {
        flashlight.intensity = 1_000_000.0;
    } else {
        flashlight.intensity = 0.0;
    }
}

fn crouch(
    mut player_query: Query<&mut Collider, With<Player>>,
    mut camera_query: Query<&mut Transform, With<PlayerCamera>>,
    player_var: Res<PlayerVar>,
    time: Res<Time>,
) {
    let Ok(mut collider) = player_query.single_mut() else {
        return;
    };

    let Ok(mut camera_transform) = camera_query.single_mut() else {
        return;
    };

    let mut camera_height = camera_transform.translation.y;

    let camera_goal = if player_var.crouching { 0.0 } else { 0.5 };

    let mut collider_scale = collider.scale().y;

    let collider_scale_goal = if player_var.crouching { 0.25 } else { 0.5 };

    collider_scale = collider_scale.lerp(collider_scale_goal, time.delta_secs() * 0.8);

    collider.set_scale(
        Vec3 {
            x: 0.4,
            y: collider_scale,
            z: 0.4,
        },
        8,
    );

    camera_height = camera_height.lerp(camera_goal, time.delta_secs() * 8.0);
    camera_transform.translation = Vec3 {
        x: 0.0,
        y: camera_height,
        z: 0.0,
    };
}
