use crate::GameState;
use crate::player::{Player, PlayerCameraSensitivity};
use bevy::{
    input::mouse::AccumulatedMouseMotion,
    prelude::*,
    window::{CursorGrabMode, CursorOptions},
};
use bevy_rapier3d::prelude::*;

use std::f32::consts::FRAC_PI_2;

const SPEED: f32 = 6.0;
const JUMP_FORCE: f32 = 8.0;
const GROUND_CHECK_DISTANCE: f32 = 1.0;

pub struct PlayerInputPlugin;

impl Plugin for PlayerInputPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (handle_input, grab_mouse, player_jump));
    }
}

fn handle_input(
    gamestate: Res<GameState>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut query: Query<(&mut Transform, &mut Velocity, &PlayerCameraSensitivity), With<Player>>,
    accumulated_mouse_motion: Res<AccumulatedMouseMotion>,
) {
    let Ok((mut transform, mut velocity, playercamerasensitivity)) = query.single_mut() else {
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

    // on ignore la composante Y pour ne pas "voler" en regardant en haut/bas
    direction.y = 0.0;
    let direction = direction.normalize_or_zero();

    velocity.linear.x = direction.x * SPEED;
    velocity.linear.z = direction.z * SPEED;

    let delta = accumulated_mouse_motion.delta;

    if gamestate.mouse_grabbed {
        if delta != Vec2::ZERO {
            // Note that we are not multiplying by delta_time here.
            // The reason is that for mouse movement, we already get the full movement that happened since the last frame.
            // This means that if we multiply by delta_time, we will get a smaller rotation than intended by the user.
            // This situation is reversed when reading e.g. analog input from a gamepad however, where the same rules
            // as for keyboard input apply. Such an input should be multiplied by delta_time to get the intended rotation
            // independent of the framerate.
            let delta_yaw = -delta.x * playercamerasensitivity.x;
            let delta_pitch = -delta.y * playercamerasensitivity.y;

            let (yaw, pitch, roll) = transform.rotation.to_euler(EulerRot::YXZ);
            let yaw = yaw + delta_yaw;

            // If the pitch was ±¹⁄₂ π, the camera would look straight up or down.
            // When the user wants to move the camera back to the horizon, which way should the camera face?
            // The camera has no way of knowing what direction was "forward" before landing in that extreme position,
            // so the direction picked will for all intents and purposes be arbitrary.
            // Another issue is that for mathematical reasons, the yaw will effectively be flipped when the pitch is at the extremes.
            // To not run into these issues, we clamp the pitch to a safe range.
            const PITCH_LIMIT: f32 = FRAC_PI_2 - 0.01;
            let pitch = (pitch + delta_pitch).clamp(-PITCH_LIMIT, PITCH_LIMIT);

            transform.rotation = Quat::from_euler(EulerRot::YXZ, yaw, pitch, roll);
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
