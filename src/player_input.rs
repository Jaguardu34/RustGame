use crate::GameState;
use crate::player::{Player, PlayerCamera, PlayerCameraSensitivity, player_jump};
use crate::spawn_objects::spawn_object;

use bevy::{
    input::mouse::AccumulatedMouseMotion,
    prelude::*,
    window::{CursorGrabMode, CursorOptions},
};
use bevy_rapier3d::prelude::*;

use crate::player::PlayerVar;

use std::f32::consts::FRAC_PI_2;

const SPAWN_POINT: Vec3 = Vec3::new(0.0, 2.0, 0.0);

pub struct PlayerInputPlugin;

impl Plugin for PlayerInputPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (handle_input, grab_mouse, player_jump, check_fall));
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
        let dash_dir = if direction != Vec3::ZERO {
            direction
        } else {
            *transform.forward()
        };

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
        player_var.base_speed * 2.5
    } else if player_var.crouching {
        player_var.base_speed / 3.0
    } else {
        player_var.base_speed
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

fn check_fall(mut query: Query<&mut Transform, With<Player>>) {
    let Ok(mut transform) = query.single_mut() else {
        return;
    };

    let y = transform.translation.y;

    if y < -10.0 {
        transform.translation = SPAWN_POINT;
        transform.rotation = Quat::from_xyzw(0.0, 0.0, 0.0, 0.0);
    }
}
