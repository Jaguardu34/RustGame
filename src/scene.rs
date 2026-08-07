use bevy::prelude::*;
use bevy_inspector_egui::bevy_egui::EguiGlobalSettings;

use crate::player::default_player;

pub struct ScenePlugin;

impl Plugin for ScenePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, (setup_meshes, setup_lights, setup_player));
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
        Transform::from_xyz(0.0, 1.0, 0.0),
        TransformGizmoFocus,
    ));
}
