use crate::game_var::GameVar;
use crate::player::{Player, PlayerCamera};

use bevy::{
    input::mouse::AccumulatedMouseMotion,
    prelude::*,
    window::{CursorGrabMode, CursorOptions},
};
use bevy_rapier3d::prelude::*;

use crate::player::PlayerVar;

use std::f32::consts::FRAC_PI_2;

pub struct PlayerInputPlugin;

impl Plugin for PlayerInputPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                handle_input,
                grab_mouse,
                handle_mouse_input,
                handle_movement_input,
            ),
        );
    }
}

//bric broc fn to redo
pub fn handle_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut player_query: Query<&mut Transform, With<Player>>,
    mut player_var: ResMut<PlayerVar>,
    mut game_var: ResMut<GameVar>,
) {
    if keyboard.just_pressed(KeyCode::Escape) && game_var.free_cam {
        game_var.free_cam = false;
    }
    if !game_var.mouse_grabbed {
        return;
    }

    if keyboard.just_pressed(KeyCode::KeyP) {
        game_var.free_cam = !game_var.free_cam;
    }

    let Ok(mut transform) = player_query.single_mut() else {
        return;
    };

    if keyboard.pressed(KeyCode::KeyR) {
        transform.translation = player_var.spawn_point;
    }

    if keyboard.just_pressed(KeyCode::KeyL) {
        if player_var.flashlight {
            player_var.flashlight = false;
        } else {
            player_var.flashlight = true;
        }
    }
}

fn handle_movement_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    mut player_var: ResMut<PlayerVar>,
    mut player_query: Query<(&Transform, &mut Velocity), With<Player>>,
    game_var: Res<GameVar>,
) {
    if !game_var.mouse_grabbed {
        return;
    }
    let Ok((transform, mut velocity)) = player_query.single_mut() else {
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

    direction.y = 0.0;
    let direction = direction.normalize_or_zero();

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
}

fn handle_mouse_input(
    accumulated_mouse_motion: Res<AccumulatedMouseMotion>,
    game_var: Res<GameVar>,
    player_var: Res<PlayerVar>,
    mut player_query: Query<&mut Transform, (With<Player>, Without<PlayerCamera>)>,
    mut player_camera_query: Query<&mut Transform, (With<PlayerCamera>, Without<Player>)>,
) {
    let Ok(mut transform) = player_query.single_mut() else {
        return;
    };
    let Ok(mut camera_transform) = player_camera_query.single_mut() else {
        return;
    };

    let delta = accumulated_mouse_motion.delta;
    if game_var.mouse_grabbed {
        if delta != Vec2::ZERO {
            let delta_yaw = -delta.x * player_var.camera_sensitivity.x;
            let delta_pitch = -delta.y * player_var.camera_sensitivity.y;

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
    key: Res<ButtonInput<KeyCode>>,
    mut game_var: ResMut<GameVar>,
) {
    if game_var.free_cam {
        return;
    }

    if game_var.mouse_grabbed {
        cursor_options.visible = false;
        cursor_options.grab_mode = CursorGrabMode::Locked;
    } else {
        cursor_options.visible = true;
        cursor_options.grab_mode = CursorGrabMode::None;
    }

    if key.just_pressed(KeyCode::Escape) {
        game_var.mouse_grabbed = !game_var.mouse_grabbed;
    }
}
