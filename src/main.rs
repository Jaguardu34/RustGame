use avian3d::prelude::*;
use bevy::{camera_controller::free_camera::FreeCameraPlugin, prelude::*};

pub mod editor;
pub mod game_var;
pub mod input;
pub mod player;
use input::PlayerInputPlugin;

use crate::character_controller::CharacterControllerPlugin;
use crate::editor::EditorPlugin;
use crate::game_var::GameVar;

use crate::player::PlayerPlugin;
use crate::scene::ScenePlugin;

pub mod player_ui;

use player_ui::UiPlugin;

pub mod free_camera;
use free_camera::FreeCamPlugin;

pub mod scene;

pub mod character_controller;

pub mod pick_object;
use pick_object::PlayerPickUpPlugin;

pub mod water;

fn main() {
    App::new()
        .init_resource::<GameVar>()
        .add_plugins((DefaultPlugins, PhysicsPlugins::default()))
        //for debugging physics
        .add_plugins(PhysicsDebugPlugin)
        // Overwrite default debug rendering configuration (optional)
        .insert_gizmo_config(
            PhysicsGizmos {
                aabb_color: Some(Color::WHITE),
                ..default()
            },
            GizmoConfig::default(),
        )
        .add_plugins((EditorPlugin, FreeCameraPlugin, TransformGizmoPlugin))
        .add_plugins(ScenePlugin)
        .add_plugins(CharacterControllerPlugin)
        .add_plugins((
            PlayerPlugin,
            PlayerInputPlugin,
            PlayerPickUpPlugin,
            UiPlugin,
            FreeCamPlugin,
        ))
        .run();
}
