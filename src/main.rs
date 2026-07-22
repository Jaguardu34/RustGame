
use std::f32::consts::FRAC_PI_2;

use bevy::{
    DefaultPlugins, color::palettes::basic::{SILVER}, dev_tools::fps_overlay::{FpsOverlayConfig, FpsOverlayPlugin, FrameTimeGraphConfig}, input::mouse::AccumulatedMouseMotion, prelude::*, text::FontSmoothing, window::{CursorGrabMode, CursorOptions, PresentMode},

};



struct OverlayColor;

impl OverlayColor {
        const GREEN: Color = Color::srgb(0.0, 1.0, 0.0);
}

#[derive(Debug, Component, Deref, DerefMut)]

struct CameraSensitivity(Vec2);

impl Default for CameraSensitivity {
    fn default() -> Self {
        Self(Vec2::new(0.003, 0.002))
    }
}

#[derive(Resource, Default, )]
struct GameState {
    is_mouse_grabbed: bool,
}





fn main() {
    App::new()
        .add_plugins(DefaultPlugins
            .set(WindowPlugin {
                primary_window: Some(Window {
                    title: "Learning Bevy".into(),
                    // 🚀 Désactive la V-Sync pour débloquer les FPS max
                    present_mode: PresentMode::AutoNoVsync, 
                    ..default()
                }),
                ..default()}
        )
            .set(ImagePlugin::default_nearest())
        )
        .add_plugins((
            FpsOverlayPlugin {
                config: FpsOverlayConfig {
                    text_config: TextFont {
                        // Here we define size of our overlay
                        font_size: FontSize::Px(42.0),
                        // If we want, we can use a custom font
                        font: default(),
                        // We could also disable font smoothing,
                        font_smoothing: FontSmoothing::default(),
                        ..default()
                    },
                    // We can also change color of the overlay
                    text_color: OverlayColor::GREEN,
                    // We can also set the refresh interval for the FPS counter
                    refresh_interval: core::time::Duration::from_millis(100),
                    enabled: true,
                    frame_time_graph_config: FrameTimeGraphConfig {
                        enabled: false,
                        // The minimum acceptable fps
                        min_fps: 30.0,
                        // The target fps
                        target_fps: 144.0,
                    },
                },
            },
        ))
        .add_systems(Startup, (setup_camera, setup_light, setup_shapes))
        .add_systems(Update, (move_camera, grab_mouse))
        .init_resource::<GameState>()
        .run();
}


fn grab_mouse(
    mut cursor_options: Single<&mut CursorOptions>,
    mouse: Res<ButtonInput<MouseButton>>,
    key: Res<ButtonInput<KeyCode>>,
    mut gamestate: ResMut<GameState>
) {
    if mouse.just_pressed(MouseButton::Left) {
        cursor_options.visible = false;
        cursor_options.grab_mode = CursorGrabMode::Locked;
        gamestate.is_mouse_grabbed = true;
    }

    if key.just_pressed(KeyCode::Escape) {
        cursor_options.visible = true;
        cursor_options.grab_mode = CursorGrabMode::None;
        gamestate.is_mouse_grabbed= false;
    }
}

fn setup_camera(mut commands: Commands) {
    commands.spawn((
        Camera3d::default(), 
        Transform::from_xyz(0., 7., 14.).looking_at(Vec3::new(0., 1., 0.), Vec3::Y),
        CameraSensitivity::default(),
    ));
}

fn setup_light(mut commands: Commands) {
    commands.spawn((
        PointLight{
            shadow_maps_enabled: true,
            intensity: 1_000_000.,
            color: Color::srgb(230. /255., 228. /255., 170. /255.),
            ..default()
        },
        Transform::from_xyz(5., 5., 5.)
    ));

}

fn setup_shapes(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server : Res<AssetServer>,
){


    let texture_grass_side: Handle<Image> = asset_server.load("minecraft/textures/block/grass_block_side.png");

    let grass_texture = materials.add(StandardMaterial {
        base_color_texture: Some(texture_grass_side.clone()),
        alpha_mode: AlphaMode::Blend,
        unlit: true,
        ..default()
    });

    commands.spawn((
        Mesh3d(meshes.add(Plane3d::default().mesh().size(50.0, 50.0).subdivisions(10))),
        MeshMaterial3d(materials.add(Color::from(SILVER))),
    ));

    commands.spawn((
        Mesh3d(meshes.add(Cuboid::default().mesh())),
        MeshMaterial3d(grass_texture),
        Transform::from_xyz(0.0, 1.0, 0.0).with_scale(Vec3::new(2.0, 2.0, 2.0)).with_rotation(Quat::from_rotation_y(30_f32.to_radians()))
    ));


}



fn move_camera(camera: Single<(&mut Transform, &CameraSensitivity)>, time: Res<Time>, keyboard: Res<ButtonInput<KeyCode>>, accumulated_mouse_motion: Res<AccumulatedMouseMotion>, gamestate: Res<GameState>){
        
        let (mut transform, camera_sensitivity) = camera.into_inner();
        
        let forward = transform.forward();
        let left = transform.left();


        let speed = if keyboard.pressed(KeyCode::ShiftLeft) {
            10.0
        } else {
            2.0
        };

        if keyboard.pressed(KeyCode::KeyA){
            transform.translation += left * time.delta_secs() * speed;
        };

        if keyboard.pressed(KeyCode::KeyD){
            transform.translation -= left * time.delta_secs() * speed;
        };

        if keyboard.pressed(KeyCode::KeyW){
            transform.translation += forward * time.delta_secs() * speed;
        };

        if keyboard.pressed(KeyCode::KeyS){
            transform.translation -= forward * time.delta_secs() * speed;
        };


        let delta = accumulated_mouse_motion.delta;

        if gamestate.is_mouse_grabbed {
            if delta != Vec2::ZERO {
                let delta_yaw = -delta.x * camera_sensitivity.x;
                let delta_pitch = -delta.y * camera_sensitivity.y;

                let (yaw, pitch, roll) = transform.rotation.to_euler(EulerRot::YXZ);
                let yaw = yaw + delta_yaw;

                const PITCH_LIMIT: f32 = FRAC_PI_2 - 0.01;
                let pitch = (pitch + delta_pitch).clamp(-PITCH_LIMIT, PITCH_LIMIT);

                transform.rotation = Quat::from_euler(EulerRot::YXZ, yaw, pitch, roll);
            }
        }

    
    
}


