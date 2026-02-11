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
    animations::{AnimationCache, AnimationState},
    characters::Movement,
    levels::overworld::OverworldProcGen,
    logging::error::{ERR_INVALID_NAV_TARGET, ERR_INVALID_NAVMESH},
    procgen::{ProcGenDespawning, ProcGenInit, ProcGenerated, TileDataCache},
    screens::Screen,
};

pub(super) fn plugin(app: &mut App) {
    // Update pathfinding
    app.add_systems(
        Update,
        (
            (
                find_path::<OverworldProcGen>,
                refresh_path::<OverworldProcGen>,
            )
                .run_if(in_state(ProcGenDespawning(false))),
            apply_path.in_set(PausableSystems),
        )
            .run_if(in_state(ProcGenInit(true)).and(in_state(Screen::Gameplay)))
            .in_set(AppSystems::Update),
    );
}

/// Map of target entities mapped to their last updated position.
#[derive(Resource, Default)]
pub(crate) struct NavTargetPosMap(HashMap<Entity, Vec2>);

/// Navigation target
///
/// [`NavTarget::0`] is meant as a priority. The higher it is, the more preferred the target is.
#[derive(Component, Default)]
pub(crate) struct NavTarget(pub(crate) u8);

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
    mut navmeshes: ResMut<Assets<NavMesh>>,
    mut target_map: ResMut<NavTargetPosMap>,
    tile_data: Res<TileDataCache<T>>,
    mut delta: Local<f32>,
) where
    T: ProcGenerated,
{
    let (navmesh_handle, status) = navmesh.deref();
    // Return if navmesh is not built
    if **status != NavMeshStatus::Built && *delta == 0. {
        return;
    }
    let navmesh = navmeshes
        .get_mut(*navmesh_handle)
        .expect(ERR_INVALID_NAVMESH);

    // Get target with maximum priority
    let Some((target, target_pos, _)) = target_query.iter().max_by_key(|q| q.2.0) else {
        return;
    };
    // Validate target pos in `NavTargetPosMap`
    let target_pos = target_pos.translation.xy();
    if let Some(pos) = target_map.0.get(&target)
        && target_pos.distance_squared(*pos) < tile_data.tile_size.squared()
    {
        return;
    }
    let target_pos_vec3 = target_pos.extend(0.);
    // Return if target pos is not in mesh
    if !navmesh.transformed_is_in_mesh(target_pos_vec3) {
        return;
    }

    let mut updated: HashMap<Entity, Vec2> = HashMap::new();
    for (entity, transform) in &navigator_query {
        let origin_pos = transform.translation;

        // Increase search delta each time the navigator is found to be outside of the navmesh
        if !navmesh.transformed_is_in_mesh(origin_pos) {
            *delta += 0.1;
            navmesh.set_search_delta(*delta);
            continue;
        }
        // Find path to target
        let Some((current, next)) = next_path_step(navmesh, origin_pos, target_pos_vec3) else {
            continue;
        };

        // Insert path
        commands.entity(entity).insert(Path {
            current,
            next,
            target,
        });
        *delta = 0.;

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
    tile_data: Res<TileDataCache<T>>,
    mut delta: Local<f32>,
) where
    T: ProcGenerated,
{
    // Return if target transforms is empty
    if target_transforms.is_empty() {
        return;
    }

    let (navmesh_handle, status) = navmesh.deref();
    // Return if navmesh is not built
    if **status != NavMeshStatus::Built && *delta == 0. {
        return;
    }
    let navmesh = navmeshes
        .get_mut(*navmesh_handle)
        .expect(ERR_INVALID_NAVMESH);

    let mut updated: HashMap<Entity, Vec2> = HashMap::new();
    for (entity, transform, mut path) in navigator_query {
        // Get transform for `path.target`
        let target_pos = target_transforms
            .get(path.target)
            .expect(ERR_INVALID_NAV_TARGET)
            .translation
            .xy();
        // Validate target pos in target map
        if let Some(pos) = target_map.0.get(&path.target)
            && target_pos.distance_squared(*pos) < tile_data.tile_size.squared()
        {
            continue;
        }
        let target_pos_vec3 = target_pos.extend(0.);
        let origin_pos = transform.translation;

        // Increase search delta each time the navigator is found to be outside of the navmesh
        if !navmesh.transformed_is_in_mesh(origin_pos) {
            *delta += 0.1;
            navmesh.set_search_delta(*delta);
            continue;
        }
        // Remove `Path` if target is outside of navmesh
        if !navmesh.transformed_is_in_mesh(target_pos_vec3) {
            commands.entity(entity).remove::<Path>();
            continue;
        }

        // Find path to target or remove path
        let Some((current, next)) = next_path_step(navmesh, origin_pos, target_pos_vec3) else {
            commands.entity(entity).remove::<Path>();
            continue;
        };

        // Modify path
        path.current = current;
        path.next = next;
        *delta = 0.0;

        updated.insert(path.target, target_pos);
    }

    // Insert updated positions into target map
    if !updated.is_empty() {
        target_map.0.extend(updated);
    }
}

/// Next step for the [`Path`]
fn next_path_step(navmesh: &mut NavMesh, start: Vec3, end: Vec3) -> Option<(Vec2, Vec<Vec2>)> {
    let path = navmesh.transformed_path(start, end)?;
    let (first, remaining) = path.path.split_first()?;
    let mut next: Vec<_> = remaining.iter().map(|p| p.xy()).collect();
    next.reverse();

    Some((first.xy(), next))
}

/// Number used as divisor for path overshoot threshold
const PATH_OVERSHOOT_THRESHOLD_DIVISOR: f32 = 50.;

/// Apply [`Path`]
fn apply_path(
    navigator_query: Query<(
        Entity,
        &Transform,
        &mut AnimationCache,
        &mut KinematicCharacterController,
        Option<&KinematicCharacterControllerOutput>,
        &mut Movement,
        &mut Path,
        &Navigator,
    )>,
    mut commands: Commands,
    time: Res<Time>,
) {
    for (
        entity,
        transform,
        mut cache,
        mut controller,
        controller_output,
        mut movement,
        mut path,
        navigator,
    ) in navigator_query
    {
        // Set movement direction to normalized vector and apply translation
        let navigator_pos = transform.translation.xy();
        let direction = path.current - navigator_pos;
        movement.direction = direction.normalize_or_zero() * navigator.0 * time.delta_secs();
        controller.translation = Some(movement.direction);

        // If `entity` collided with `path.target` stop applying path and return.
        // NOTE: This does not reliably determine whether the `entity` can not advance, just if it has collided with their target.
        //       For now this should be enough since not switching to `Idle` for these entities might cause the illusion of them
        //       still trying to wiggle their way around obstacles.
        if let Some(output) = controller_output
            && output.collisions.iter().any(|c| c.entity == path.target)
        {
            stop_apply_path(&mut commands, entity, &mut cache);
            return;
        }

        // Set animation state if we are `Idle`
        if cache.state == AnimationState::Idle {
            cache.set_new_state(AnimationState::Walk);
        }

        // Loop while distance to `path.current` is smaller than threshold to allow multiple next
        while navigator_pos.distance_squared(path.current)
            < (navigator.0 / PATH_OVERSHOOT_THRESHOLD_DIVISOR).squared()
        {
            // Set `path.current` to `path.next` if it exists or stop applying path and break from loop.
            if let Some(next) = path.next.pop() {
                path.current = next;
            } else {
                stop_apply_path(&mut commands, entity, &mut cache);
                break;
            }
        }
    }
}

/// Remove [`Path`] and set [`AnimationCache`] state to [`AnimationState::Idle`]
fn stop_apply_path(commands: &mut Commands, entity: Entity, cache: &mut AnimationCache) {
    // NOTE: We are using `try_remove` to avoid use after despawn because of `procgen::despawn`.
    commands.entity(entity).try_remove::<Path>();
    cache.set_new_state(AnimationState::Idle);
}
