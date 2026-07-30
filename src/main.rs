use bevy::camera::RenderTarget;
use bevy::camera::visibility::RenderLayers;
use bevy::camera_controller::free_camera::FreeCameraPlugin;
use bevy::prelude::*;
use bevy::render::mesh::{Mesh, VertexAttributeValues};
use bevy::render::render_resource::TextureFormat;
use bevy_rapier3d::prelude::*;

pub mod player;
use player::PlayerVar;
use player::default_player;

pub mod player_input;
use player_input::PlayerInputPlugin;

use crate::player::PlayerPlugin;

pub mod spawn_objects;

pub mod ui;
use ui::UiPlugin;

pub mod free_camera;
use free_camera::FreeCamPlugin;

#[derive(Component)]
pub struct Mirror;

//pub mod platform_physics;

#[derive(Resource, Default)]
pub struct GameState {
    pub mouse_grabbed: bool,
}

#[derive(Component)]
pub struct MovingPlatform;

#[derive(Component)]
pub struct FloatingPlatform;

fn main() {
    App::new()
        .init_resource::<GameState>()
        .init_resource::<PlayerVar>()
        .add_plugins(DefaultPlugins)
        .add_plugins(RapierPhysicsPlugin::<NoUserData>::default())
        .add_plugins(FreeCameraPlugin)
        .add_plugins((PlayerPlugin, PlayerInputPlugin, UiPlugin, FreeCamPlugin))
        .add_systems(Startup, (setup_lights, setup_scene, setup_player))
        .add_systems(Update, update_scene)
        .run();
}

fn setup_lights(mut commands: Commands) {
    commands.spawn((
        DirectionalLight {
            shadow_maps_enabled: true,
            illuminance: 600.0,
            ..default()
        },
        Transform::from_xyz(3.0, 3.0, 3.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));
    commands.spawn(AmbientLight::default());
}

fn setup_scene(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut images: ResMut<Assets<Image>>,
) {
    let image = Image::new_target_texture(
        512,
        512,
        TextureFormat::Rgba8Unorm,
        Some(TextureFormat::Rgba8UnormSrgb),
    );

    let image_handle = images.add(image);

    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(20.0, 1.0, 20.0))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::WHITE,
            perceptual_roughness: 1.0,
            ..default()
        })),
        Transform::from_xyz(0.0, -0.5, 0.0),
        RigidBody::Fixed,
        Collider::cuboid(10.0, 0.5, 10.0),
    ));

    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(0.2, 2.0, 10.0))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.7, 0.7, 0.8),
            perceptual_roughness: 0.05,
            metallic: 1.0,
            ..default()
        })),
        Transform::from_xyz(5.0, 1.0, -5.0),
        RigidBody::Fixed,
        Collider::cuboid(0.1, 1.0, 5.0),
    ));

    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(1.0, 1.0, 1.0))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgba(1.0, 0.0, 0.0, 0.1),
            emissive: LinearRgba {
                red: 2.0,
                green: 0.0,
                blue: 0.0,
                alpha: 1.0,
            },
            ..default()
        })),
        Transform::from_xyz(1.0, 0.5, -2.0),
        RigidBody::Fixed,
        Collider::cuboid(0.5, 0.5, 0.5),
    ));

    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(5.0, 0.5, 5.0))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.0, 1.0, 0.0),
            ..default()
        })),
        Transform::from_xyz(15.0, 0.0, 15.0),
        RigidBody::Fixed,
        Collider::cuboid(2.5, 0.25, 2.5),
        MovingPlatform,
    ));

    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(2.0, 0.5, 2.0))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.0, 0.0, 1.0),
            ..default()
        })),
        Transform::from_xyz(2.0, 1.0, 2.0),
        RigidBody::Dynamic,
        Collider::cuboid(1.0, 0.25, 1.0),
        ExternalForce::default(),
        FloatingPlatform,
    ));

    commands.spawn((
        Camera3d::default(),
        Camera {
            // render before the "main pass" camera
            order: -1,
            clear_color: Color::WHITE.into(),

            ..default()
        },
        RenderTarget::Image(image_handle.clone().into()),
        RenderLayers::from_layers(&[0, 1]),
        Transform::from_xyz(-2.0, 0.7, 0.0)
            .with_rotation(Quat::from_rotation_y(-90.0_f32.to_radians())),
    ));

    let material_handle = materials.add(StandardMaterial {
        base_color_texture: Some(image_handle),
        reflectance: 0.02,
        unlit: false,
        ..default()
    });

    let mut mirror_mesh = Mesh::from(Plane3d::default().mesh().size(1.6, 0.9).subdivisions(10));

    flip_uvs_horizontal(&mut mirror_mesh);

    commands.spawn((
        Mesh3d(meshes.add(mirror_mesh)),
        MeshMaterial3d(material_handle),
        Transform::from_xyz(-2.0, 0.7, 0.0).with_rotation(
            Quat::from_rotation_arc(Vec3::Y, Vec3::X)
                * Quat::from_rotation_y(std::f32::consts::FRAC_PI_2),
        ),
    ));
}

fn flip_uvs_horizontal(mesh: &mut Mesh) {
    if let Some(VertexAttributeValues::Float32x2(uvs)) = mesh.attribute_mut(Mesh::ATTRIBUTE_UV_0) {
        for uv in uvs.iter_mut() {
            uv[0] = 1.0 - uv[0]; // Inversion horizontale
        }
    }
}

fn setup_player(
    mut commands: Commands,
    meshes: ResMut<Assets<Mesh>>,
    materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.spawn(default_player(meshes, materials));
}

fn update_scene(
    mut platform_query: Query<&mut Transform, With<MovingPlatform>>,
    time: Res<Time>,
    mut forward: Local<bool>,
) {
    let Ok(mut transform) = platform_query.single_mut() else {
        return;
    };

    const SPEED: f32 = 3.0; // unités par seconde
    const MIN_X: f32 = 0.0;
    const MAX_X: f32 = 15.0;

    let translation_goal = if *forward { MAX_X } else { MIN_X };
    let translation_pos = transform.translation.x;

    let step = SPEED * time.delta_secs();
    let new_pos = if *forward {
        (translation_pos + step).min(translation_goal)
    } else {
        (translation_pos - step).max(translation_goal)
    };

    transform.translation.x = new_pos;

    // Une fois la cible atteinte exactement, on inverse le sens
    if new_pos == translation_goal {
        *forward = !*forward;
    }
}
