use avian3d::{collider_tree, prelude::*};
use bevy::{ecs::entity::MapEntities, prelude::*};

use crate::player::{Player, PlayerCamera};

#[derive(Component)]
pub struct PlayerPickable;

pub struct PlayerPickUpPlugin;

#[derive(Resource, Default)]
pub struct CanPick(pub bool);

impl Plugin for PlayerPickUpPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CanPick>()
            .add_systems(Update, update_can_pick);
    }
}

fn update_can_pick(
    spatial_query: SpatialQuery,
    player_cam_query: Query<&GlobalTransform, With<PlayerCamera>>,
    player_query: Query<Entity, With<Player>>,
    pickable_query: Query<&PlayerPickable>,
    mut can_pick: ResMut<CanPick>,
    parents: Query<&ChildOf>,
) {
    let Ok(cam_transform) = player_cam_query.single() else {
        return;
    };
    let Ok(player_entity) = player_query.single() else {
        return;
    };

    let origin = cam_transform.translation();
    let direction = cam_transform.forward();
    let filter = SpatialQueryFilter::default().with_excluded_entities([player_entity]);

    let hit = spatial_query.cast_ray_predicate(origin, direction, 4.0, true, &filter, &|entity| {
        let mut current = entity;
        loop {
            if pickable_query.contains(current) {
                return true;
            }
            match parents.get(current) {
                Ok(child_of) => current = child_of.parent(),
                Err(_) => return false,
            }
        }
    });

    can_pick.0 = hit.is_some();
}
