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

fn main() {
    App::new()
        .init_resource::<GameVar>()
        .add_plugins(DefaultPlugins)
        .add_plugins(PhysicsPlugins::default())
        .add_plugins(EditorPlugin)
        .add_plugins(FreeCameraPlugin)
        .add_plugins(TransformGizmoPlugin)
        .add_plugins(ScenePlugin)
        .add_plugins(CharacterControllerPlugin)
        //.add_plugins(ChunkPlugin)
        .add_plugins((PlayerPlugin, PlayerInputPlugin, UiPlugin, FreeCamPlugin))
        .run();
}
