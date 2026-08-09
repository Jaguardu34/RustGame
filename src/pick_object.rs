use avian3d::prelude::*;
use bevy::prelude::*;

use crate::player::{Player, PlayerCamera};

#[derive(Component)]
pub struct PlayerPickable;

pub struct PlayerPickUpPlugin;

#[derive(Resource, Default)]
pub struct CanPick(pub bool);

#[derive(Resource, Default)]
pub struct ObjectPicked(Option<Entity>);

impl Plugin for PlayerPickUpPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CanPick>()
            .init_resource::<ObjectPicked>()
            .add_systems(Update, (update_can_pick, pickup_object));
    }
}

const FORCE: f32 = 10.0;
const DISTANCE: f32 = 1.0;

fn update_can_pick(
    spatial_query: SpatialQuery,
    player_cam_query: Query<&GlobalTransform, With<PlayerCamera>>,
    player_query: Query<Entity, With<Player>>,
    pickable_query: Query<&PlayerPickable>,
    mut can_pick: ResMut<CanPick>,
    parents: Query<&ChildOf>,
    mut object_picked: ResMut<ObjectPicked>,
    mouse: Res<ButtonInput<MouseButton>>,
) {
    let Ok(cam_transform) = player_cam_query.single() else {
        return;
    };
    let Ok(player_entity) = player_query.single() else {
        return;
    };

    if mouse.just_released(MouseButton::Left) {
        object_picked.0 = None;
    }
    if object_picked.0.is_some() {
        return;
    }

    let origin = cam_transform.translation();
    let direction = cam_transform.forward();
    let filter = SpatialQueryFilter::default().with_excluded_entities([player_entity]);
    let hit = spatial_query.cast_ray_predicate(origin, direction, 4.0, true, &filter, &|entity| {
        find_pickable_ancestor(entity, &pickable_query, &parents).is_some()
    });

    can_pick.0 = hit.is_some();

    if let Some(hit_data) = hit {
        if mouse.just_pressed(MouseButton::Left) {
            object_picked.0 = find_pickable_ancestor(hit_data.entity, &pickable_query, &parents);
        }
    }
}

fn find_pickable_ancestor(
    entity: Entity,
    pickable_query: &Query<&PlayerPickable>,
    parents: &Query<&ChildOf>,
) -> Option<Entity> {
    let mut current = entity;
    loop {
        if pickable_query.contains(current) {
            return Some(current);
        }
        match parents.get(current) {
            Ok(child_of) => current = child_of.parent(),
            Err(_) => return None,
        }
    }
}

fn pickup_object(
    player_camera_query: Query<&GlobalTransform, With<PlayerCamera>>,
    object_picked: Res<ObjectPicked>,
    mut transform_query: Query<(&mut LinearVelocity, &mut AngularVelocity, &GlobalTransform)>,
) {
    let Ok(cam_transform) = player_camera_query.single() else {
        return;
    };
    let Some(entity) = object_picked.0 else {
        return;
    };
    let hand_pos = cam_transform.translation() + cam_transform.forward().as_vec3() * DISTANCE;
    let Ok((mut velocity, mut angular_velocity, transform)) = transform_query.get_mut(entity)
    else {
        return;
    };

    let diff = hand_pos - transform.translation();
    velocity.0 = diff * FORCE;
    angular_velocity.0 = Vec3::ZERO;
}
