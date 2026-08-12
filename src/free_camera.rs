use bevy::{
    camera::{Hdr, visibility::RenderLayers},
    camera_controller::free_camera::FreeCamera,
    post_process::bloom::Bloom,
    prelude::*,
    window::{CursorGrabMode, CursorOptions},
};

use crate::{
    editor::{EditorState, GameViewCam},
    game_var::GameVar,
    player::{Player, PlayerCamera},
    scene::setup_player,
};

#[derive(Component)]
pub struct FreeCam;

pub struct FreeCamPlugin;

impl Plugin for FreeCamPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_free_cam.after(setup_player))
            .add_systems(Update, toggle_free_cam);
    }
}

pub fn spawn_free_cam(
    mut commands: Commands,
    camera_query: Query<(&GlobalTransform, &Projection), With<PlayerCamera>>,
) {
    let Ok((player_cam_transform, projection)) = camera_query.single() else {
        println!("Player pas spawn");
        return;
    };

    commands.spawn((
        Camera3d::default(),
        Name::new("FreeCam"),
        FreeCam,
        FreeCamera::default(),
        Camera {
            is_active: false,
            order: 0,
            ..default()
        },
        Hdr,
        GameViewCam,
        TransformGizmoCamera,
        MeshPickingCamera,
        Bloom::NATURAL,
        projection.clone(),
        Transform::from_translation(player_cam_transform.translation())
            .with_rotation(player_cam_transform.rotation()),
        RenderLayers::from_layers(&[0, 1]),
        Msaa::Off,
    ));
}

pub fn toggle_free_cam(
    mut player_camera_query: Query<
        (&mut Camera, &GlobalTransform),
        (With<PlayerCamera>, Without<FreeCam>),
    >,
    mut free_cam_query: Query<
        (&mut Camera, &mut Transform),
        (With<FreeCam>, Without<Player>, Without<PlayerCamera>),
    >,
    editor_state: Res<EditorState>,
    mut cursor_options: Single<&mut CursorOptions>,
    mut last_game_var: Local<bool>,
    mut game_var: ResMut<GameVar>,
) {
    let Ok((mut player_camera, player_camera_transform)) = player_camera_query.single_mut() else {
        return;
    };

    let Ok((mut free_cam_camera, mut transform)) = free_cam_query.single_mut() else {
        return;
    };

    if !editor_state.game_playing {
        free_cam_camera.is_active = true;
        player_camera.is_active = false;
    } else {
        player_camera.is_active = true;
        free_cam_camera.is_active = false;
    }

    if *last_game_var != editor_state.game_playing {
        if !editor_state.game_playing {
            game_var.mouse_grabbed = false;
            cursor_options.visible = true;
            cursor_options.grab_mode = CursorGrabMode::None;
            transform.translation = player_camera_transform.translation();
            transform.rotation = player_camera_transform.rotation();
        }
    }
    *last_game_var = editor_state.game_playing;
}
