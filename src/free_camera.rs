use bevy::{
    camera::visibility::RenderLayers, camera_controller::free_camera::FreeCamera, prelude::*,
};

use crate::player::{Player, PlayerCamera, PlayerVar};

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
    mut player_var: ResMut<PlayerVar>,
) {
    let Ok(mut player_transform) = player_query.single_mut() else {
        return;
    };

    let Ok((mut player_camera, player_cam_transform, projection)) =
        player_camera_query.single_mut()
    else {
        return;
    };

    let player_var_dump = player_var.free_cam;

    if keyboard.just_pressed(KeyCode::KeyP) {
        player_var.free_cam = !player_var.free_cam;
    }

    if keyboard.just_pressed(KeyCode::Enter) && player_var.free_cam {
        player_var.free_cam = false;
        let Ok((entity, transform)) = free_cam_query.single() else {
            return;
        };
        player_transform.translation = transform.translation;
        player_camera.is_active = true;
        commands.entity(entity).despawn();
    }

    if player_var_dump != player_var.free_cam {
        if player_var.free_cam {
            commands.spawn((
                Camera3d::default(),
                FreeCam,
                FreeCamera::default(),
                Camera {
                    is_active: true,
                    ..default()
                },
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
}
