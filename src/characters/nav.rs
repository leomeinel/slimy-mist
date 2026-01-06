/*
 * File: nav.rs
 * Author: Leopold Johannes Meinel (leo@meinel.dev)
 * -----
 * Copyright (c) 2026 Leopold Johannes Meinel & contributors
 * SPDX ID: Apache-2.0
 * URL: https://www.apache.org/licenses/LICENSE-2.0
 * -----
 * This is heavily inspired by: https://github.com/vleue/vleue_navigator
 */

use std::ops::Deref;

use bevy::{math::FloatPow, platform::collections::HashMap, prelude::*};
use bevy_rapier2d::prelude::*;
use vleue_navigator::prelude::*;

use crate::{
    AppSystems, PausableSystems,
    characters::{
        Movement, VisualMap,
        animations::{AnimationController, AnimationState},
    },
    levels::overworld::OverworldProcGen,
    logging::error::{
        ERR_INVALID_NAV_TARGET, ERR_INVALID_NAVMESH, ERR_INVALID_VISUAL_MAP, ERR_LOADING_TILE_DATA,
    },
    procgen::{CHUNK_SIZE, ProcGenController, ProcGenInit, ProcGenerated, TileData, TileHandle},
    screens::Screen,
};

pub(super) fn plugin(app: &mut App) {
    // Insert/Remove resources
    app.add_systems(OnEnter(Screen::Gameplay), insert_resources);
    app.add_systems(OnExit(Screen::Gameplay), remove_resources);

    // Update pathfinding
    app.add_systems(
        Update,
        (
            find_path::<OverworldProcGen>,
            refresh_path::<OverworldProcGen>,
            apply_path.in_set(PausableSystems),
        )
            .run_if(in_state(ProcGenInit(true)).and(in_state(Screen::Gameplay)))
            .in_set(AppSystems::Update),
    );
}

/// This contains a map of target entities mapped to their last updated position
#[derive(Resource, Default)]
pub(crate) struct NavTargetPosMap(HashMap<Entity, Vec2>);

/// Navigation target
///
/// The [`usize`] field is meant as a priority. The higher it is, the more preferred the target is.
#[derive(Component, Default)]
pub(crate) struct NavTarget(pub(crate) usize);

/// Navigator that will pathfind to [`NavTarget`]
#[derive(Component)]
pub(crate) struct Navigator(pub(crate) f32);

/// Path that is used for pathfinding to [`NavTarget`]
#[derive(Component)]
pub(crate) struct Path {
    pub(crate) current: Vec2,
    pub(crate) next: Vec<Vec2>,
    target: Entity,
}

/// Find [`Path`] to [`NavTarget`]
///
/// ## Traits
///
/// - `T` must implement [`ProcGenerated`]' and is used as the procedurally generated level.
fn find_path<T>(
    navmesh: Single<(&ManagedNavMesh, Ref<NavMeshStatus>)>,
    target_query: Query<(Entity, &Transform, &NavTarget), Without<Navigator>>,
    navigator_query: Query<
        (Entity, &Transform),
        (With<Navigator>, Without<Path>, Without<NavTarget>),
    >,
    mut commands: Commands,
    mut target_map: ResMut<NavTargetPosMap>,
    controller: Res<ProcGenController<T>>,
    data: Res<Assets<TileData<T>>>,
    handle: Res<TileHandle<T>>,
    navmeshes: Res<Assets<NavMesh>>,
    mut tile_size: Local<Option<f32>>,
) where
    T: ProcGenerated,
{
    let (navmesh_handle, status) = navmesh.deref();
    // Return if navmesh is not built
    if **status != NavMeshStatus::Built {
        return;
    }
    let navmesh = navmeshes.get(*navmesh_handle).expect(ERR_INVALID_NAVMESH);

    // Get target with maximum priority
    let Some((target, target_pos, _)) = target_query.iter().max_by_key(|q| q.2.0) else {
        return;
    };

    // Init local values
    let tile_size = tile_size.unwrap_or_else(|| {
        let data = data.get(handle.0.id()).expect(ERR_LOADING_TILE_DATA);
        let value = data.tile_size;
        *tile_size = Some(value);
        value
    });

    // Save and validate target pos in `NavTargetPosMap`
    let target_pos = target_pos.translation.xy();
    if let Some(pos) = target_map.0.get(&target)
        && target_pos.distance_squared(*pos) < tile_size.squared()
    {
        return;
    }

    let min_world_pos_scaled = controller.min_chunk_pos().as_vec2() * CHUNK_SIZE.as_vec2();
    // NOTE: We are subtracting `min_world_pos_scaled` to get the nav mesh pos
    let target_pos_scaled = (target_pos / tile_size - min_world_pos_scaled).floor();

    // Return if target pos is not in mesh
    if !navmesh.is_in_mesh(target_pos_scaled) {
        return;
    }

    let mut updated: HashMap<Entity, Vec2> = HashMap::new();
    for (entity, transform) in &navigator_query {
        // Find path to target
        let Some(path) = navmesh.transformed_path(
            transform.translation.xyz(),
            navmesh
                .transform()
                .transform_point(target_pos_scaled.extend(0.)),
        ) else {
            continue;
        };
        // Get current and first from path
        let Some((first, remaining)) = path.path.split_first() else {
            continue;
        };
        let mut next: Vec<Vec2> = remaining.iter().map(|p| p.xy()).collect();
        next.reverse();

        // Insert path
        // NOTE: We are using `try_insert` since it is possible that `entity` has been despawned by `procgen::despawn_procgen`
        //       Not calling `updated.insert` conditionally based on success is not desired, but preventing it would be quite ugly.
        //       If insertion fails, we will just not update a few paths in the following runs while the target has not moved.
        commands.entity(entity).try_insert(Path {
            current: first.xy(),
            next,
            target,
        });
        updated.insert(target, target_pos);
    }

    // Insert updated positions into target map
    if !updated.is_empty() {
        target_map.0.extend(updated);
    }
}

/// Refresh [`Path`]
fn refresh_path<T>(
    navmesh: Single<(&ManagedNavMesh, Ref<NavMeshStatus>)>,
    navigator_query: Query<(Entity, &Transform, &mut Path), With<Navigator>>,
    target_transforms: Query<&Transform, With<NavTarget>>,
    mut commands: Commands,
    mut target_map: ResMut<NavTargetPosMap>,
    mut navmeshes: ResMut<Assets<NavMesh>>,
    data: Res<Assets<TileData<T>>>,
    handle: Res<TileHandle<T>>,
    mut delta: Local<f32>,
    mut tile_size: Local<Option<f32>>,
) where
    T: ProcGenerated,
{
    // Return if target transforms is empty
    if target_transforms.is_empty() {
        return;
    }

    let (navmesh_handle, status) = navmesh.deref();
    // Return if navmesh is not built
    if **status != NavMeshStatus::Built && *delta == 0.0 {
        return;
    }
    let navmesh = navmeshes
        .get_mut(*navmesh_handle)
        .expect(ERR_INVALID_NAVMESH);

    // Init local values
    let tile_size = tile_size.unwrap_or_else(|| {
        let data = data.get(handle.0.id()).expect(ERR_LOADING_TILE_DATA);
        let value = data.tile_size;
        *tile_size = Some(value);
        value
    });

    let mut updated: HashMap<Entity, Vec2> = HashMap::new();
    for (entity, transform, mut path) in navigator_query {
        // Get transform for `path.target`
        let target_pos = target_transforms
            .get(path.target)
            .expect(ERR_INVALID_NAV_TARGET)
            .translation
            .xy();

        // Save and validate target pos in target map
        if let Some(pos) = target_map.0.get(&path.target)
            && target_pos.distance_squared(*pos) < tile_size.squared()
        {
            continue;
        }

        // Increase search delta each time the navigator is found to be outside of the navmesh
        if !navmesh.transformed_is_in_mesh(transform.translation) {
            *delta += 0.1;
            navmesh.set_search_delta(*delta);
            continue;
        }
        // Remove `Path` if target is outside of navmesh
        if !navmesh.transformed_is_in_mesh(target_pos.extend(0.0)) {
            // NOTE: We are using `try_remove` since it is possible that `entity` has been despawned by `procgen::despawn_procgen`.
            commands.entity(entity).try_remove::<Path>();
            continue;
        }

        // Find path to target or remove path
        let Some(new_path) =
            navmesh.transformed_path(transform.translation, target_pos.extend(0.0))
        else {
            // NOTE: We are using `try_remove` since it is possible that `entity` has been despawned by `procgen::despawn_procgen`.
            commands.entity(entity).try_remove::<Path>();
            continue;
        };
        // Get current and first from path
        let Some((first, remaining)) = new_path.path.split_first() else {
            continue;
        };
        let mut next = remaining.iter().map(|p| p.xy()).collect::<Vec<_>>();
        next.reverse();

        // Modify path
        path.current = first.xy();
        path.next = next;
        *delta = 0.0;
        updated.insert(path.target, target_pos);
    }

    // Insert updated positions into target map
    if !updated.is_empty() {
        target_map.0.extend(updated);
    }
}

/// Number used as divisor for path overshoot threshold
const PATH_OVERSHOOT_THRESHOLD_DIVISOR: f32 = 50.;

/// Apply [`Path`]
fn apply_path(
    mut child_query: Query<&mut AnimationController, Without<Navigator>>,
    navigator_query: Query<(
        Entity,
        &Transform,
        &mut KinematicCharacterController,
        Option<&KinematicCharacterControllerOutput>,
        &mut Movement,
        &mut Path,
        &Navigator,
    )>,
    mut commands: Commands,
    time: Res<Time>,
    visual_map: Res<VisualMap>,
) {
    for (entity, transform, mut controller, controller_output, mut movement, mut path, navigator) in
        navigator_query
    {
        // Set movement direction to normalized vector and apply translation
        let navigator_pos = transform.translation.xy();
        let direction = path.current - navigator_pos;
        movement.direction = direction.normalize() * navigator.0 * time.delta_secs();
        controller.translation = Some(movement.direction);

        // Extract `animation_controller` from `child_query`
        let visual = visual_map.0.get(&entity).expect(ERR_INVALID_VISUAL_MAP);
        let mut animation_controller = child_query.get_mut(*visual).expect(ERR_INVALID_VISUAL_MAP);

        // If `entity` collided with `path.target` stop applying path and return.
        // NOTE: This does not reliably determine whether the `entity` can not advance, just if it has collided with their target.
        //       For now this should be enough since not switching to `Idle` for these entities might cause the illusion of them
        //       still trying to wiggle their way around obstacles.
        if let Some(output) = controller_output
            && output.collisions.iter().any(|c| c.entity == path.target)
        {
            stop_apply_path(&mut commands, entity, &mut animation_controller);
            return;
        }

        // Set animation state if we are `Idle`
        if animation_controller.state == AnimationState::Idle {
            animation_controller.set_new_state(AnimationState::Walk);
        }

        // Loop while distance to `path.current` is smaller than threshold to allow multiple next
        while navigator_pos.distance_squared(path.current)
            < (navigator.0 / PATH_OVERSHOOT_THRESHOLD_DIVISOR).squared()
        {
            // Set `path.current` to `path.next` if it exists or stop applying path and break from loop.
            if let Some(next) = path.next.pop() {
                path.current = next;
            } else {
                stop_apply_path(&mut commands, entity, &mut animation_controller);
                break;
            }
        }
    }
}

/// Remove [`Path`] and set [`AnimationController`] state to [`AnimationState::Idle`]
fn stop_apply_path(commands: &mut Commands, entity: Entity, controller: &mut AnimationController) {
    commands.entity(entity).remove::<Path>();
    controller.set_new_state(AnimationState::Idle);
}

/// Insert resources
fn insert_resources(mut commands: Commands) {
    commands.init_resource::<NavTargetPosMap>();
}

/// Remove resources
fn remove_resources(mut commands: Commands) {
    commands.remove_resource::<NavTargetPosMap>();
}
