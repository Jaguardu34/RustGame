use crate::editor::EditorState;
use crate::game_var::GameVar;
use crate::player::{Player, PlayerCamera};

use avian3d::debug_render::PhysicsGizmos;
use bevy::{
    input::mouse::AccumulatedMouseMotion,
    prelude::*,
    window::{CursorGrabMode, CursorOptions},
};

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
                toggle_physics_gizmos,
            ),
        );
    }
}

//bric broc fn to redo
pub fn handle_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut player_query: Query<&mut Transform, With<Player>>,
    mut player_var: ResMut<PlayerVar>,
    editor_state: Res<EditorState>,
    mut game_var: ResMut<GameVar>,
) {
    if keyboard.just_pressed(KeyCode::F12) {
        game_var.in_editor = !game_var.in_editor;
    }

    if keyboard.just_pressed(KeyCode::KeyG) && keyboard.pressed(KeyCode::F3) {
        game_var.hitbox_shown = !game_var.hitbox_shown
    }

    if !editor_state.game_playing {
        return;
    }

    let Ok(mut transform) = player_query.single_mut() else {
        return;
    };

    if keyboard.just_pressed(KeyCode::KeyR) {
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
    editor_state: Res<EditorState>,
    buttons: Res<ButtonInput<MouseButton>>,
) {
    if !editor_state.game_playing {
        return;
    }

    if game_var.mouse_grabbed {
        cursor_options.visible = false;
        cursor_options.grab_mode = CursorGrabMode::Locked;
    } else {
        cursor_options.visible = true;
        cursor_options.grab_mode = CursorGrabMode::None;
    }
    if buttons.just_pressed(MouseButton::Left) && editor_state.pointer_in_viewport {
        game_var.mouse_grabbed = true;
    }

    if key.just_pressed(KeyCode::Escape) && game_var.mouse_grabbed {
        game_var.mouse_grabbed = false;
    }
}

fn toggle_physics_gizmos(mut store: ResMut<GizmoConfigStore>, game_var: Res<GameVar>) {
    if game_var.is_changed() {
        let (config, _) = store.config_mut::<PhysicsGizmos>();
        config.enabled = game_var.hitbox_shown;
    }
}
