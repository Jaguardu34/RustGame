use avian3d::prelude::*;
use bevy::prelude::*;
use bevy_inspector_egui::bevy_egui::EguiGlobalSettings;
use rand::random_range;

use crate::{pick_object::PlayerPickable, player::default_player};

pub struct ScenePlugin;

impl Plugin for ScenePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, (setup_meshes, setup_lights, setup_player))
            .add_systems(Update, spawn_object);
    }
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
    //commands.spawn(AmbientLight::default());
}

pub fn setup_player(
    mut commands: Commands,
    meshes: ResMut<Assets<Mesh>>,
    materials: ResMut<Assets<StandardMaterial>>,
    mut egui_global_settings: ResMut<EguiGlobalSettings>,
) {
    egui_global_settings.auto_create_primary_context = false;

    commands.spawn(default_player(
        meshes,
        materials,
        Vec3 {
            x: 0.0,
            y: 10.0,
            z: 0.0,
        },
    ));
}

fn setup_meshes(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(1.0, 1.0, 1.0))),
        MeshMaterial3d(materials.add(Color::srgb(0.8, 0.3, 0.3))),
        Transform::from_xyz(0.0, 2.0, 0.0),
        RigidBody::Dynamic,
        Collider::cuboid(1.0, 1.0, 1.0),
    ));

    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(100.0, 1.0, 100.0))),
        MeshMaterial3d(materials.add(Color::WHITE)),
        RigidBody::Static,
        Collider::cuboid(100.0, 1.0, 100.0),
    ));

    commands.spawn((
        Mesh3d(meshes.add(Capsule3d::new(0.2, 0.2))),
        MeshMaterial3d(materials.add(Color::srgb(0.0, 1.0, 0.0))),
        RigidBody::Dynamic,
        Collider::capsule(0.2, 0.2),
        Transform::from_xyz(2.0, 2.0, 2.0),
        PlayerPickable,
    ));
}

fn spawn_object(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    keyboard: Res<ButtonInput<KeyCode>>,
) {
    if keyboard.pressed(KeyCode::KeyB) {
        commands.spawn((
            WorldAssetRoot(
                asset_server.load(GltfAssetLabel::Scene(0).from_asset("fantasy_prop/Stool.gltf")),
            ),
            Transform::from_xyz(random_range(-10.0..-5.0), 4., random_range(-10.0..-5.0)),
            ColliderConstructorHierarchy::new(ColliderConstructor::ConvexHullFromMesh),
            RigidBody::Dynamic,
            ColliderDensity(0.8),
            PlayerPickable,
        ));
    }
}
