use bevy::prelude::*;
use bevy_inspector_egui::{InspectorOptions, inspector_options::ReflectInspectorOptions};

//Global GameVar
#[derive(Resource, Reflect, InspectorOptions)]
#[reflect(Resource, InspectorOptions)]
pub struct GameVar {
    //is freecam enabled ?
    pub mouse_grabbed: bool, //is the mouse grabbed by the game ?
    pub render_distance: u32,
    pub in_editor: bool,
    pub hitbox_shown: bool,
}

impl Default for GameVar {
    fn default() -> Self {
        GameVar {
            mouse_grabbed: false,
            render_distance: 40,
            in_editor: false,
            hitbox_shown: false,
        }
    }
}
