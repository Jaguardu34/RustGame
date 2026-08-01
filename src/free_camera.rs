use bevy::{
    camera::{Hdr, visibility::RenderLayers},
    camera_controller::free_camera::FreeCamera,
    prelude::*,
    window::{CursorGrabMode, CursorOptions},
    post_process::bloom::Bloom,
};


use crate::{
    game_var::GameVar,
    player::{Player, PlayerCamera},
};

#[derive(Component)]
pub struct FreeCam;

pub struct FreeCamPlugin;

impl Plugin for FreeCamPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, toggle_free_cam);
    }
}

pub fn toggle_free_cam(
    mut player_camera_query: Query<
        (&mut Camera, &GlobalTransform, &Projection),
        (With<PlayerCamera>, Without<FreeCam>),
    >,
    mut commands: Commands,
    mut player_query: Query<&mut Transform, (With<Player>, Without<FreeCam>)>,
    free_cam_query: Query<(Entity, &Transform), (With<FreeCam>, Without<Player>)>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut game_var: ResMut<GameVar>,
    mut cursor_options: Single<&mut CursorOptions>,
    mut last_game_var: Local<bool>,
) {
    let Ok(mut player_transform) = player_query.single_mut() else {
        return;
    };

    let Ok((mut player_camera, player_cam_transform, projection)) =
        player_camera_query.single_mut()
    else {
        return;
    };

    if keyboard.just_pressed(KeyCode::Enter) && game_var.free_cam {
        game_var.free_cam = false;
        let Ok((entity, transform)) = free_cam_query.single() else {
            return;
        };
        player_transform.translation = transform.translation;
        player_camera.is_active = true;
        commands.entity(entity).despawn();
    } else if *last_game_var != game_var.free_cam {
        if game_var.free_cam {
            game_var.mouse_grabbed = false;
            cursor_options.visible = true;
            cursor_options.grab_mode = CursorGrabMode::None;
            commands.spawn((
                Camera3d::default(),
                FreeCam,
                FreeCamera::default(),
                Camera {
                    is_active: true,
                    order: 0,
                    ..default()
                },
                Hdr,
                Bloom::NATURAL,
                projection.clone(),
                Transform::from_translation(player_cam_transform.translation())
                    .with_rotation(player_cam_transform.rotation()),
                RenderLayers::from_layers(&[0, 1]),
            ));
            player_camera.is_active = false;
        } else {
            let Ok((entity, _transform)) = free_cam_query.single() else {
                return;
            };
            player_camera.is_active = true;
            commands.entity(entity).despawn();
        }
    }
    *last_game_var = game_var.free_cam;
}
