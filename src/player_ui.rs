use avian3d::spatial_query::{SpatialQuery, SpatialQueryFilter};
use bevy::prelude::*;

use crate::{
    player::{Player, PlayerCamera, PlayerVar, TickTimer},
    scene::setup_player,
};

#[derive(Component)]
pub struct DefaultUi;

pub struct UiPlugin;

#[derive(Component)]
pub struct PlayerCrosshair;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_ui.after(setup_player))
            .add_systems(Update, (update_ui, can_place));
    }
}

#[derive(Component)]
pub struct UICamera;

fn spawn_ui(mut commands: Commands, player_camera_query: Query<Entity, With<PlayerCamera>>) {
    let Ok(player_camera_entity) = player_camera_query.single() else {
        return;
    };

    commands
        .spawn((
            Name::new("Player_UI"),
            Node {
                width: Val::Percent(100.0),
                height: Val::Px(50.0),
                position_type: PositionType::Absolute,
                top: Val::Px(10.0),
                ..Default::default()
            },
            UiTargetCamera(player_camera_entity),
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new(String::new()),
                TextFont::default(),
                Node {
                    margin: UiRect::bottom(px(10)),
                    ..Default::default()
                },
                DefaultUi,
            ));
        });
    commands
        .spawn((
            Name::new("Player_Crosshair"),
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                position_type: PositionType::Absolute,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            UiTargetCamera(player_camera_entity),
        ))
        .with_children(|parent| {
            parent.spawn((
                Node {
                    width: Val::Px(4.0),
                    height: Val::Px(4.0),
                    ..default()
                },
                BackgroundColor(Color::WHITE),
                PlayerCrosshair,
            ));
        });
}

fn update_ui(
    mut text_query: Query<&mut Text, With<DefaultUi>>,
    player_var: Res<PlayerVar>,
    mut timer: ResMut<TickTimer>,
    time: Res<Time>,
) {
    timer.tick(time.delta());

    if !timer.just_finished() {
        return;
    }

    let Ok(mut text) = text_query.single_mut() else {
        return;
    };

    let text_to_show: &str = &String::from(format!(
        "Sprinting : {}, Crouching : {}\nx: {:.1} y: {:.1} z : {:.1}\nSpeed : {:.1}",
        player_var.sprinting,
        player_var.crouching,
        player_var.coord.x,
        player_var.coord.y,
        player_var.coord.z,
        player_var.speed
    ));

    text.clear();
    text.insert_str(0, text_to_show);
}

fn can_place(
    spatial_query: SpatialQuery,
    player_query: Query<Entity, With<Player>>,
    camera_query: Query<&GlobalTransform, With<PlayerCamera>>,
    mut crosshair_query: Query<&mut BackgroundColor, With<PlayerCrosshair>>,
) {
    let Ok(player_entity) = player_query.single() else {
        return;
    };

    let Ok(camera_transform) = camera_query.single() else {
        return;
    };

    let Ok(mut background_color) = crosshair_query.single_mut() else {
        return;
    };

    let ray_origin = camera_transform.translation();
    let ray_direction = camera_transform.forward();
    let ray_max_distance = 2.0;
    let filter = SpatialQueryFilter::default().with_excluded_entities([player_entity]);

    let can_pose = spatial_query
        .cast_ray(ray_origin, ray_direction, ray_max_distance, true, &filter)
        .is_some();

    if can_pose {
        background_color.0 = Color::srgb(1.0, 0.0, 0.0);
    } else {
        background_color.0 = Color::WHITE;
    }
}
