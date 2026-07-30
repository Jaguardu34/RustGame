use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

use crate::MovingPlatform;
use crate::player::Player;

pub struct PlatformPhysicsPlugin;

impl Plugin for PlatformPhysicsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, is_player_on_platform);
    }
}

fn is_player_on_platform(
    player_query: Query<(Entity, &Collider, &Transform), With<Player>>,
    platform_query: Query<Entity, With<MovingPlatform>>,
    context_query: ReadRapierContext,
) {
    let Ok((player_entity, player_collider, player_transorm)) = player_query.single() else {
        return;
    };

    let Ok(context) = context_query.single() else {
        return;
    };

    let ray_origin = player_transorm.translation;
    let ray_dir = Vec3::NEG_Y;
    let max_toi = player_collider.scale().y / 2;
    let ray_exclude = QueryFilter::default().exclude_collider(player_entity);

    let hit = context.cast_ray(ray_origin, ray_dir, max_toi, true, ray_exclude);
    for platform_entity in platform_query.iter() {
        if hit == platform_entity {
            println!("Platform_touchée")
        }
    }
}
