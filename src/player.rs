use bevy::{
    camera::{Hdr, visibility::RenderLayers},
    prelude::*,
};

use avian3d::prelude::*;
use bevy_inspector_egui::{InspectorOptions, inspector_options::ReflectInspectorOptions};
use std::time::Duration;

use crate::{
    character_controller::CharacterControllerBundle, editor::GameViewCam, game_var::GameVar,
};

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
#[derive(Resource, Reflect, InspectorOptions)]
#[reflect(Resource, InspectorOptions)]
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
        app.init_resource::<PlayerVar>()
            .register_type::<PlayerVar>()
            .init_resource::<TickTimer>()
            .add_systems(Update, (update_flashlight, update_player_var));
    }
}

//const THIRD_PERSON_CAMERA_DISTANCE: f32 = 10.0;

pub fn default_player(
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    pos: Vec3,
) -> impl Bundle {
    (
        Player,
        Name::new("Player"),
        CharacterControllerBundle::new(Collider::capsule(PLAYER_RADIUS, PLAYER_HALF_LENGTH_STAND))
            .with_movement(30.0, 0.92, 4.0, 30f32.to_radians()),
        Friction::ZERO.with_combine_rule(CoefficientCombine::Min),
        Restitution::ZERO.with_combine_rule(CoefficientCombine::Min),
        Mass(80.0),
        RenderLayers::layer(1),
        Mesh3d(meshes.add(Capsule3d {
            radius: PLAYER_RADIUS,
            half_length: PLAYER_HALF_LENGTH_STAND,
        })),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(1.0, 0.0, 0.0),
            ..default()
        })),
        Transform::from_translation(pos),
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
            //Bloom::NATURAL,
            Hdr,
            Msaa::Off,
            GameViewCam,
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

//fn to update the useful player var like coords etc
fn update_player_var(
    mut player_var: ResMut<PlayerVar>,
    player_query: Query<(&Transform, &LinearVelocity), With<Player>>,
) {
    let Ok((transform, velocity)) = player_query.single() else {
        return;
    };
    player_var.speed = velocity.abs().max_element();
    player_var.coord = transform.translation;
}
