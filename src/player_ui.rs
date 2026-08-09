use avian3d::collision::collider::Collider;
use bevy::{
    diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin},
    prelude::*,
};

use crate::{
    pick_object::CanPick,
    player::{PlayerCamera, PlayerVar, TickTimer},
    scene::setup_player,
};

#[derive(Component)]
pub struct DefaultUi;
#[derive(Component)]
pub struct UICamera;
#[derive(Component)]
pub struct PlayerCrosshair;

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_ui.after(setup_player))
            .add_systems(Update, update_ui);
    }
}

fn spawn_ui(
    mut commands: Commands,
    player_camera_query: Query<Entity, With<PlayerCamera>>,
    asset_server: Res<AssetServer>,
) {
    let Ok(player_camera_entity) = player_camera_query.single() else {
        return;
    };

    commands
        .spawn((
            Name::new("Player_UI"),
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
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
                ImageNode::new(asset_server.load("cursor_pack/PNG/Basic/Default/line_cross.png")),
                // Child Node control `ImageNode` size
                Node {
                    width: px(11.),
                    height: px(11.),
                    ..default()
                },
                PlayerCrosshair,
            ));
        });
}

fn update_ui(
    mut text_query: Query<&mut Text, With<DefaultUi>>,
    player_var: Res<PlayerVar>,
    mut timer: ResMut<TickTimer>,
    time: Res<Time>,
    diagnostics: Res<DiagnosticsStore>,
    physic_object_query: Query<&Collider>,
    mut crosshair_query: Query<&mut ImageNode, With<PlayerCrosshair>>,
    can_pickup: Res<CanPick>,
    asset_server: Res<AssetServer>,
) {
    timer.tick(time.delta());

    if !timer.just_finished() {
        return;
    }

    let Ok(mut text) = text_query.single_mut() else {
        return;
    };

    let Ok(mut crosshair) = crosshair_query.single_mut() else {
        return;
    };

    let fps_value = if let Some(value) = diagnostics
        .get(&FrameTimeDiagnosticsPlugin::FPS)
        .and_then(|fps| fps.smoothed())
    {
        format!("{:.1}", value)
    } else {
        String::from("Error fetching FPS")
    };

    let count = physic_object_query.iter().len();

    let text_to_show: &str = &String::from(format!(
        "Sprinting : {}, Crouching : {}\nx: {:.1} y: {:.1} z : {:.1}\nSpeed : {:.1}\nFps : {}\nPhysics objects : {}",
        player_var.sprinting,
        player_var.crouching,
        player_var.coord.x,
        player_var.coord.y,
        player_var.coord.z,
        player_var.speed,
        fps_value,
        count
    ));

    text.clear();
    text.insert_str(0, text_to_show);

    if can_pickup.is_changed() {
        if can_pickup.0 {
            let crosshair_image: Handle<Image> =
                asset_server.load("cursor_pack/PNG/Basic/Default/hand_open.png");
            crosshair.image = crosshair_image;
        } else {
            let crosshair_image: Handle<Image> =
                asset_server.load("cursor_pack/PNG/Basic/Default/line_cross.png");
            crosshair.image = crosshair_image;
        }
    }
}
