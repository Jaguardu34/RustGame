use bevy::{
    camera::{Hdr, visibility::RenderLayers},
    post_process::bloom::Bloom,
    prelude::*,
};
use bevy_rapier3d::prelude::*;

use std::time::Duration;

use crate::game_var::GameVar;

//tick timer for ui
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

//CONST
const PLAYER_HEIGHT: f32 = 1.0;
const PLAYER_RADIUS: f32 = 0.2;
//valeur capsule debout
const PLAYER_HALF_LENGTH_STAND: f32 = PLAYER_HEIGHT / 2.0 - PLAYER_RADIUS; // 0.3
//valeur en crouch
const PLAYER_HEIGHT_CROUCH: f32 = PLAYER_HEIGHT / 2.0; // 0.5
//valeur capsule en crouch
const PLAYER_HALF_LENGTH_CROUCH: f32 = PLAYER_HEIGHT_CROUCH / 2.0 - PLAYER_RADIUS; // 0.05

#[derive(Component, Default)]
pub struct Player;

#[derive(Component)]
pub struct PlayerCamera;

#[derive(Component)]
pub struct PlayerFlashLight;

//useful player var
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
    pub camera_sensitivity: Vec2,
    pub spawn_point: Vec3,
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
            camera_sensitivity: Vec2::new(0.003, 0.002),
            spawn_point: Vec3::new(0.0, 2.0, 0.0),
        } // Custom default value
    }
}

pub struct PlayerPlugin;
//player plugin
impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<TickTimer>().add_systems(
            Update,
            (update_flashlight, crouch, update_player_var, player_jump),
        );
    }
}

//const THIRD_PERSON_CAMERA_DISTANCE: f32 = 10.0;

pub fn default_player(
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) -> impl Bundle {
    (
        Player,
        Name::new("Player"),
        Transform::from_xyz(0.0, 2.0, 0.0),
        RigidBody::Dynamic,
        Collider::capsule_y(PLAYER_HEIGHT / 2.0 - (PLAYER_RADIUS * 2.0), PLAYER_RADIUS),
        Velocity::default(),
        LockedAxes::ROTATION_LOCKED,
        ActiveEvents::COLLISION_EVENTS,
        ExternalForce::default(),
        RenderLayers::layer(1),
        ColliderMassProperties::Density(2.0),
        Mesh3d(meshes.add(Capsule3d {
            radius: PLAYER_RADIUS,
            half_length: PLAYER_HEIGHT / 2.0 - (PLAYER_RADIUS * 2.0),
        })),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(1.0, 0.0, 0.0),
            ..default()
        })),
        children![(
            //camera of the player
            Camera3d::default(),
            Name::new("PlayerCamera"),
            Camera {
                is_active: true,
                order: 0,
                ..default()
            },
            Transform::from_xyz(0.0, 0.5, 0.0),
            PlayerCamera,
            Projection::from(PerspectiveProjection {
                fov: 90.0_f32.to_radians(),
                ..default()
            }),
            Bloom::NATURAL,
            Hdr,
            Msaa::Off,
            children![(
                //flashlight of the player
                SpotLight {
                    range: 30.0,
                    shadow_maps_enabled: true,
                    inner_angle: 0.4,
                    intensity: 1_000_000.0,
                    outer_angle: 0.8,
                    ..default()
                },
                Name::new("PlayerFlashlight"),
                Transform::from_xyz(0.0, 0.0, 0.0),
                PlayerFlashLight
            )]
        ),],
    )
}

//fn to toggle flashlight
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

//fn to crouch
fn crouch(
    mut player_query: Query<(&mut Collider, &mut Mesh3d), With<Player>>,
    mut camera_query: Query<&mut Transform, With<PlayerCamera>>,
    player_var: Res<PlayerVar>,
    time: Res<Time>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    let Ok((mut collider, mut meshe)) = player_query.single_mut() else {
        return;
    };

    let Ok(mut camera_transform) = camera_query.single_mut() else {
        return;
    };

    let Some(capsule) = collider.as_capsule() else {
        return;
    };

    //getting the half lenght
    let mut half_length = capsule.segment().length() / 2.0;

    //setting halg lenght
    let half_length_goal = if player_var.crouching {
        PLAYER_HALF_LENGTH_CROUCH
    } else {
        PLAYER_HALF_LENGTH_STAND
    };

    half_length = half_length.lerp(half_length_goal, time.delta_secs() * 8.0);

    if (half_length - half_length_goal).abs() > 0.001
        || half_length != capsule.segment().length() / 2.0
    {
        *collider = Collider::capsule_y(half_length, PLAYER_RADIUS);
        *meshe = Mesh3d(meshes.add(Capsule3d {
            radius: PLAYER_RADIUS,
            half_length,
        }));
    }

    let camera_goal = if player_var.crouching {
        PLAYER_HEIGHT_CROUCH / 2.0 - 0.1
    } else {
        PLAYER_HEIGHT / 2.0 - 0.1
    };
    let camera_height = camera_transform
        .translation
        .y
        .lerp(camera_goal, time.delta_secs() * 8.0);

    camera_transform.translation = Vec3 {
        x: 0.0,
        y: camera_height,
        z: 0.0,
    };
}

//fn to update the useful player var like coords etc
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
    keyboard: Res<ButtonInput<KeyCode>>,
    game_var: Res<GameVar>,
    player_var: Res<PlayerVar>,
) {
    if !game_var.mouse_grabbed {
        return;
    }

    let Ok((entity, transform, mut velocity, collider)) = query.single_mut() else {
        return;
    };

    if !keyboard.pressed(KeyCode::Space) {
        return;
    };

    let Ok(context) = rapier_context.single() else {
        return;
    };

    let Some(capsule) = collider.as_capsule() else {
        return;
    };
    let half_height = capsule.height() / 2.0; // ou capsule.segment().length()/2.0 selon la version
    let radius = capsule.radius();

    let ray_origin = transform.translation;
    let ray_dir = Vec3::NEG_Y;
    let max_distance = half_height + radius + 0.1;
    let filter = QueryFilter::default().exclude_collider(entity);

    let is_grounded = context
        .cast_ray(ray_origin, ray_dir, max_distance, true, filter)
        .is_some();

    if is_grounded {
        velocity.linear.y = player_var.jump_force;
    }
}
