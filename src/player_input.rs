use crate::GameState;
use crate::player::{Player, PlayerCamera, PlayerCameraSensitivity, PlayerFlashLight};
use crate::spawn_objects::spawn_object;

use bevy::{
    input::mouse::AccumulatedMouseMotion,
    prelude::*,
    window::{CursorGrabMode, CursorOptions},
};
use bevy_rapier3d::prelude::*;

use crate::player::PlayerVar;

use std::f32::consts::FRAC_PI_2;

const SPEED: f32 = 6.0;
const JUMP_FORCE: f32 = 4.0;
const GROUND_CHECK_DISTANCE: f32 = 1.0;
const SPAWN_POINT: Vec3 = Vec3::new(0.0, 2.0, 0.0);

pub struct PlayerInputPlugin;

impl Plugin for PlayerInputPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_crosshair)
            .add_systems(Update, (handle_input, grab_mouse, player_jump, check_fall));
    }
}

fn handle_input(
    gamestate: Res<GameState>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut player_query: Query<
        (&mut Transform, &mut Velocity, &PlayerCameraSensitivity),
        With<Player>,
    >,
    mut camera_query: Query<&mut Transform, (With<PlayerCamera>, Without<Player>)>,
    mut player_var: ResMut<PlayerVar>,
    accumulated_mouse_motion: Res<AccumulatedMouseMotion>,
    commands: Commands,
    meshes: ResMut<Assets<Mesh>>,
    materials: ResMut<Assets<StandardMaterial>>,
    time: Res<Time>,
) {
    let Ok((mut transform, mut velocity, playercamerasensitivity)) = player_query.single_mut()
    else {
        return;
    };

    let Ok(mut camera_transform) = camera_query.single_mut() else {
        return;
    };

    let mut direction = Vec3::ZERO;

    if keyboard.pressed(KeyCode::KeyW) {
        direction += *transform.forward();
    }
    if keyboard.pressed(KeyCode::KeyS) {
        direction += *transform.back();
    }
    if keyboard.pressed(KeyCode::KeyA) {
        direction += *transform.left();
    }
    if keyboard.pressed(KeyCode::KeyD) {
        direction += *transform.right();
    }

    if keyboard.pressed(KeyCode::KeyR) {
        transform.translation = SPAWN_POINT;
    }

    if keyboard.pressed(KeyCode::KeyT) {
        spawn_object(commands, meshes, materials);
    }

    if keyboard.just_pressed(KeyCode::KeyL) {
        if player_var.flashlight {
            player_var.flashlight = false;
        } else {
            player_var.flashlight = true;
        }
    }

    if keyboard.just_pressed(KeyCode::KeyE) {
        let dash_dir = *transform.forward();

        velocity.linear.x = dash_dir.x * 20.0;
        velocity.linear.z = dash_dir.z * 20.0;
    }

    direction.y = 0.0;
    let direction = direction.normalize_or_zero();

    if keyboard.pressed(KeyCode::ShiftLeft) && !player_var.crouching {
        player_var.sprinting = true;
    } else {
        player_var.sprinting = false;
    }

    if keyboard.pressed(KeyCode::KeyC) && !player_var.sprinting {
        player_var.crouching = true;
    } else {
        player_var.crouching = false;
    }

    let goal_speed = if player_var.sprinting {
        SPEED * 3.0
    } else if player_var.crouching {
        SPEED / 4.0
    } else {
        SPEED
    };

    let target_velocity = Vec3::new(direction.x * goal_speed, 0.0, direction.z * goal_speed);
    let current = Vec3::new(velocity.linear.x, 0.0, velocity.linear.z);
    let new_vel = current.lerp(target_velocity, time.delta_secs() * 4.0);
    velocity.linear.x = new_vel.x;
    velocity.linear.z = new_vel.z;

    let delta = accumulated_mouse_motion.delta;

    if gamestate.mouse_grabbed {
        if delta != Vec2::ZERO {
            let delta_yaw = -delta.x * playercamerasensitivity.x;
            let delta_pitch = -delta.y * playercamerasensitivity.y;

            // Yaw sur le Player (corps + collider)
            let (yaw, _, _) = transform.rotation.to_euler(EulerRot::YXZ);
            let yaw = yaw + delta_yaw;
            transform.rotation = Quat::from_euler(EulerRot::YXZ, yaw, 0.0, 0.0);

            // Pitch UNIQUEMENT sur la caméra enfant
            let (_, pitch, _) = camera_transform.rotation.to_euler(EulerRot::YXZ);
            const PITCH_LIMIT: f32 = FRAC_PI_2 - 0.01;
            let pitch = (pitch + delta_pitch).clamp(-PITCH_LIMIT, PITCH_LIMIT);
            camera_transform.rotation = Quat::from_euler(EulerRot::YXZ, 0.0, pitch, 0.0);
        }
    }
}

fn grab_mouse(
    mut cursor_options: Single<&mut CursorOptions>,
    mouse: Res<ButtonInput<MouseButton>>,
    key: Res<ButtonInput<KeyCode>>,
    mut gamestate: ResMut<GameState>,
) {
    if mouse.just_pressed(MouseButton::Left) {
        cursor_options.visible = false;
        cursor_options.grab_mode = CursorGrabMode::Locked;
        gamestate.mouse_grabbed = true;
    }

    if key.just_pressed(KeyCode::Escape) {
        cursor_options.visible = true;
        cursor_options.grab_mode = CursorGrabMode::None;
        gamestate.mouse_grabbed = false;
    }
}

fn player_jump(
    keyboard: Res<ButtonInput<KeyCode>>,
    rapier_context: ReadRapierContext,
    mut query: Query<(Entity, &Transform, &mut Velocity), With<Player>>,
) {
    let Ok((entity, transform, mut velocity)) = query.single_mut() else {
        return;
    };

    if !keyboard.just_pressed(KeyCode::Space) {
        return;
    }

    let Ok(context) = rapier_context.single() else {
        return;
    };

    let ray_origin = transform.translation;
    let ray_dir = Vec3::NEG_Y;
    let max_distance = GROUND_CHECK_DISTANCE;
    let filter = QueryFilter::default().exclude_collider(entity);

    let is_grounded = context
        .cast_ray(ray_origin, ray_dir, max_distance, true, filter)
        .is_some();

    if is_grounded {
        velocity.linear.y = JUMP_FORCE;
    }
}

fn check_fall(mut query: Query<&mut Transform, With<Player>>) {
    let Ok(mut transform) = query.single_mut() else {
        return;
    };
    let y = transform.translation.y;

    if y < -10.0 {
        transform.translation = SPAWN_POINT;
    }
}

fn spawn_crosshair(mut commands: Commands) {
    println!("Crosshair est spawn");
    commands
        .spawn(Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            position_type: PositionType::Absolute,
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..default()
        })
        .with_children(|parent| {
            parent.spawn((
                Node {
                    width: Val::Px(4.0),
                    height: Val::Px(4.0),
                    ..default()
                },
                BackgroundColor(Color::WHITE),
            ));
        });
}
