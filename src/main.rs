use bevy::prelude::*;

pub fn greeting() {
    println!("Hello World this is RustCraft starting")

}

fn main() {
    App::new()
        .add_systems(Startup, greeting)
        .add_plugins(DefaultPlugins)
        .run();
}



