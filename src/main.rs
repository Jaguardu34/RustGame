
use std::f32::consts::FRAC_PI_2;

use bevy::{
    DefaultPlugins, asset::processor::InitializeError::FailedToReadSourcePaths, color::palettes::{basic::SILVER, css::WHITE}, dev_tools::fps_overlay::{FpsOverlayConfig, FpsOverlayPlugin, FrameTimeGraphConfig}, input::{common_conditions::input_just_pressed, keyboard::Key, mouse::AccumulatedMouseMotion}, prelude::*, scene, text::FontSmoothing, transform, window::{CursorGrabMode, CursorOptions, PresentMode},

};
use bevy::world_serialization::WorldInstanceReady;



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
    is_debug_texture_enabled: bool,
}

#[derive(Component)]
struct PendingTextureChange;

#[derive(Component)]
struct HasTexture;





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
        .add_systems(Update, toggle_texture_debug_state.run_if(input_just_pressed(KeyCode::AltLeft)))
        .add_systems(Update, toggle_texture_debug.run_if(input_just_pressed(KeyCode::AltLeft)))
        .add_observer(change_texture)
        .insert_resource(GameState{is_mouse_grabbed: false, is_debug_texture_enabled: false})
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

fn toggle_texture_debug_state(
    mut gamestate: ResMut<GameState>
){
    if gamestate.is_debug_texture_enabled {
        gamestate.is_debug_texture_enabled = false;
    } else {
        gamestate.is_debug_texture_enabled = true;
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

    commands.insert_resource(GlobalAmbientLight {
        color: WHITE.into(),
        brightness: 200.0,
        ..default()
    });


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

  

    let spider_model : Handle<WorldAsset> = asset_server
        .load(GltfAssetLabel::Scene(0).from_asset("models/model.gltf"));

    



    commands.spawn((
        WorldAssetRoot(spider_model.clone()),
        Transform::from_xyz(0.0, 1.0, 5.0),
        PendingTextureChange,
        HasTexture
    ));

    let mut iterator = 0;

    while iterator < 10{


        commands.spawn((
            Mesh3d(meshes.add(Cuboid::default().mesh())),
            MeshMaterial3d(grass_texture.clone()),
            Transform::from_xyz(iterator as f32, 0.5, 0.0).with_scale(Vec3::new(1., 1., 1.))
        ));
        iterator+=1;
    }


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


fn change_texture(
    trigger: On<WorldInstanceReady>,
    query: Query<Entity, With<PendingTextureChange>>,
    children_query: Query<&Children>,
    mesh_query: Query<&Mesh3d>,
    mut commands: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
) {
    let root = trigger.event_target();

    if query.get(root).is_err() {
        return;
    }

    let spider_material = materials.add(StandardMaterial {
        base_color_texture: Some(asset_server.load("minecraft/textures/entity/spider/spider.png")),
        ..default()
    });

    for descendant in children_query.iter_descendants(root) {
        if mesh_query.get(descendant).is_ok() {
            commands.entity(descendant).insert(MeshMaterial3d(spider_material.clone()));
        }
    }

    commands.entity(root).remove::<PendingTextureChange>();
}


fn toggle_texture_debug(
    mut commands: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
    query: Query<Entity, With<HasTexture>>,
    children_query: Query<&Children>,
    mesh_query: Query<&Mesh3d>,
    gamestate: Res<GameState>,
) {
    for root in query {
        for descendant in children_query.iter_descendants(root) {
            if mesh_query.get(descendant).is_err() {
                continue;
            }

            if gamestate.is_debug_texture_enabled {
                commands.entity(descendant).remove::<MeshMaterial3d<StandardMaterial>>();
                let spider_material = materials.add(StandardMaterial {
                    base_color_texture: Some(asset_server.load("models/spider.png")),
                    ..default()
                });
                commands.entity(descendant).insert(MeshMaterial3d(spider_material));
            } else {
                commands.entity(descendant).remove::<MeshMaterial3d<StandardMaterial>>();
                let spider_material = materials.add(StandardMaterial {
                    base_color_texture: Some(asset_server.load("minecraft/textures/entity/spider/spider.png")),
                    ..default()
                });
                commands.entity(descendant).insert(MeshMaterial3d(spider_material));
            }
        }
    }
}