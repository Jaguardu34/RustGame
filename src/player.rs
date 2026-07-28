use bevy::{post_process::bloom::Bloom, prelude::*};
use bevy_rapier3d::prelude::*;

use std::time::Duration;

#[derive(Resource, Deref, DerefMut)]
pub struct TickTimer(Timer);

impl Default for TickTimer {
    fn default() -> Self {
        Self(Timer::new(
            Duration::from_secs_f32(0.1),
            TimerMode::Repeating,
        ))
    }
}

#[derive(Component, Default)]
pub struct Player;

#[derive(Component)]
pub struct PlayerCamera;

#[derive(Component)]
pub struct PlayerFlashLight;

#[derive(Debug, Component, Deref, DerefMut)]
pub struct PlayerCameraSensitivity(Vec2);

#[derive(Resource)]
pub struct PlayerVar {
    pub flashlight: bool,
    pub sprinting: bool,
    pub crouching: bool,
    pub speed: f32,
    pub coord: Vec3,
    pub can_place: bool,
    pub jump_force: f32,
    pub base_speed: f32,
}

impl Default for PlayerVar {
    fn default() -> Self {
        Self {
            flashlight: false,
            sprinting: false,
            crouching: false,
            speed: 0.0,
            coord: Vec3::ZERO,
            can_place: false,
            jump_force: 4.0,
            base_speed: 6.0,
        } // Custom default value
    }
}

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<TickTimer>().add_systems(
            Update,
            (update_flashlight, crouch, update_player_var, player_jump),
        );
    }
}

impl Default for PlayerCameraSensitivity {
    fn default() -> Self {
        Self(Vec2::new(0.003, 0.002))
    }
}

const THIRD_PERSON_CAMERA_DISTANCE: f32 = 10.0;

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

fn update_player_var(
    mut player_var: ResMut<PlayerVar>,
    player_query: Query<(&Transform, &Velocity), With<Player>>,
) {
    let Ok((transform, velocity)) = player_query.single() else {
        return;
    };
    player_var.speed = velocity.linear.abs().max_element();
    player_var.coord = transform.translation;
}

pub fn player_jump(
    rapier_context: ReadRapierContext,
    mut query: Query<(Entity, &Transform, &mut Velocity, &Collider), With<Player>>,
    player_var: Res<PlayerVar>,
    keyboard: Res<ButtonInput<KeyCode>>,
) {
    let Ok((entity, transform, mut velocity, collider)) = query.single_mut() else {
        return;
    };

    if !keyboard.pressed(KeyCode::Space) {
        return;
    };

    let Ok(context) = rapier_context.single() else {
        return;
    };

    let ray_origin = transform.translation;
    let ray_dir = Vec3::NEG_Y;
    let max_distance = collider.scale().y / 2.0;
    let filter = QueryFilter::default().exclude_collider(entity);

    let is_grounded = context
        .cast_ray(ray_origin, ray_dir, max_distance, true, filter)
        .is_some();

    if is_grounded {
        velocity.linear.y = player_var.jump_force;
    }
}
