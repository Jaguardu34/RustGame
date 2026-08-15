use bevy::prelude::*;

use crate::scene::spawn_grass_plane;
use rand::random_range;

pub struct GrassPlugin;

impl Plugin for GrassPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, build_grass.after(spawn_grass_plane));
    }
}

#[derive(Component)]
pub struct GrassPlane {
    pub size: Vec2,
    pub density: f32,
}

fn build_grass(
    mut commands: Commands,
    plane_query: Query<(&Transform, &GrassPlane)>,
    asset_server: Res<AssetServer>,
) {
    for (transform, plane) in plane_query.iter() {
        let nb_x = ((plane.size.x / plane.density) / 2.) as i32;
        let nb_y = ((plane.size.y / plane.density) / 2.) as i32;
        let asset: Handle<WorldAsset> = asset_server
            .load(GltfAssetLabel::Scene(0).from_asset("mega_nature/glTF/Grass_Common_Short.gltf"));
        for x in -nb_x..nb_x + 1 {
            for y in -nb_y..nb_y + 1 {
                let world_x = transform.translation.x + x as f32 * plane.density;
                let world_y = transform.translation.y;
                let world_z = transform.translation.z + y as f32 * plane.density;
                commands.spawn((
                    WorldAssetRoot(asset.clone()),
                    Transform::from_xyz(
                        world_x + random_range(-0.05..0.05),
                        world_y,
                        world_z + random_range(-0.05..0.05),
                    )
                    .with_rotation(Quat::from_rotation_y(random_range(-2.0..2.0)))
                    .with_scale(Vec3::splat(random_range(0.2..0.4))),
                ));
            }
        }
    }
}
