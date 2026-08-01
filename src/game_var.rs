use bevy::prelude::*;

//Global GameVar
#[derive(Resource)]
pub struct GameVar {
    pub free_cam: bool,      //is freecam enabled ?
    pub mouse_grabbed: bool, //is the mouse grabbed by the game ?
}

impl Default for GameVar {
    fn default() -> Self {
        GameVar {
            free_cam: false,
            mouse_grabbed: false,
        }
    }
}
