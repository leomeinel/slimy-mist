/*
 * File: navmesh.rs
 * Author: Leopold Johannes Meinel (leo@meinel.dev)
 * -----
 * Copyright (c) 2026 Leopold Johannes Meinel & contributors
 * SPDX ID: Apache-2.0
 * URL: https://www.apache.org/licenses/LICENSE-2.0
 * -----
 * Heavily inspired by: https://github.com/vleue/vleue_navigator
 */

use bevy::prelude::*;
use polyanya::Triangulation;
use vleue_navigator::prelude::*;

use crate::{
    levels::Level,
    logging::error::ERR_LOADING_TILE_DATA,
    procgen::{
        CHUNK_SIZE, PROCGEN_DISTANCE, ProcGenController, ProcGenInit, ProcGenState, ProcGenerated,
        TileData, TileHandle,
    },
};

pub(super) fn plugin(app: &mut App) {
    // Add north star plugin
    app.add_plugins((
        VleueNavigatorPlugin,
        NavmeshUpdaterPlugin::<PrimitiveObstacle>::default(),
    ));
}

/// Number of horizontal/vertical chunks in a straight line
const NUM_CHUNKS: u32 = PROCGEN_DISTANCE as u32 * 2 + 1;
/// Size of the [`ManagedNavMesh`]
const NAVMESH_SIZE: UVec2 = UVec2::new(CHUNK_SIZE.x * NUM_CHUNKS, CHUNK_SIZE.y * NUM_CHUNKS);

/// Spawn [`ManagedNavMesh`] from [`NavMeshSettings`]
///
/// ## Traits
///
/// - `T` must implement [`ProcGenerated`]' and is used as the procedurally generated level.
/// - `A` must implement [`Level`].
pub(crate) fn spawn_navmesh<T, A>(
    level: Single<Entity, With<A>>,
    mut commands: Commands,
    data: Res<Assets<TileData<T>>>,
    handle: Res<TileHandle<T>>,
    mut tile_size: Local<Option<f32>>,
) where
    T: ProcGenerated,
    A: Level,
{
    // Init local values
    let tile_size = tile_size.unwrap_or_else(|| {
        let data = data.get(handle.0.id()).expect(ERR_LOADING_TILE_DATA);
        let value = data.tile_size;
        *tile_size = Some(value);
        value
    });

    // NOTE: This is anchored to the bottom left. This means that we have to offset it by:
    //       (`NAVMESH_SIZE` + (one chunk - 1 tile)) / 2 as world pos.
    //       This seemingly weird calculation is in part due to the chunk spawning at `0,0` having its
    //       minimum tile centered at `0,0`, not the chunk itself.
    //       Otherwise we would only have to offset it by: `NAVMESH_SIZE` / 2 as world pos.
    let target_pos = (-NAVMESH_SIZE.as_vec2() + (CHUNK_SIZE.as_vec2() - 1.)) * tile_size / 2.;

    let entity = commands
        .spawn((
            NavMeshSettings {
                simplify: 0.1,
                merge_steps: 1,
                fixed: Triangulation::from_outer_edges(&[
                    Vec2::ZERO,
                    Vec2::new(NAVMESH_SIZE.x as f32, 0.),
                    NAVMESH_SIZE.as_vec2(),
                    Vec2::new(0., NAVMESH_SIZE.y as f32),
                ]),
                ..default()
            },
            // FIXME: Investigate why `NavMeshUpdateMode::Direct` does not update correctly and prevents pathfinding
            NavMeshUpdateMode::Debounced(0.1),
            Transform::from_translation(target_pos.extend(0.)).with_scale(Vec3::splat(tile_size)),
        ))
        .id();

    commands.entity(level.entity()).add_child(entity);
}

/// Move [`ManagedNavMesh`] with generated chunks
///
/// ## Traits
///
/// - `T` must implement [`ProcGenerated`]' and is used as the procedurally generated level.
pub(crate) fn move_navmesh<T>(
    mut navmesh: Single<&mut Transform, With<ManagedNavMesh>>,
    controller: Res<ProcGenController<T>>,
    mut next_init_state: ResMut<NextState<ProcGenInit>>,
    mut next_state: ResMut<NextState<ProcGenState>>,
    init_state: Res<State<ProcGenInit>>,
    data: Res<Assets<TileData<T>>>,
    handle: Res<TileHandle<T>>,
    mut tile_size: Local<Option<f32>>,
) where
    T: ProcGenerated,
{
    // Init local values
    let tile_size = tile_size.unwrap_or_else(|| {
        let data = data.get(handle.0.id()).expect(ERR_LOADING_TILE_DATA);
        let value = data.tile_size;
        *tile_size = Some(value);
        value
    });

    // Change navmesh translation
    let min_world_pos = controller.min_chunk_pos().as_vec2() * CHUNK_SIZE.as_vec2() * tile_size;
    // NOTE: This is anchored to the bottom left. Instead of min world pos, we actually need the minimum tile of the center chunk.
    //       Therefore we are adding `CHUNK_SIZE` * `PROCGEN_DISTANCE` to the calculation from `spawn_navmesh`
    //       and then adding everything to world pos to get the correct offset as world pos.
    let target_pos = (min_world_pos
        + ((-NAVMESH_SIZE.as_vec2() + (CHUNK_SIZE.as_vec2() - 1.)) / 2.
            + CHUNK_SIZE.as_vec2() * PROCGEN_DISTANCE as f32)
            * tile_size)
        .floor();
    navmesh.translation = target_pos.extend(0.);

    // Proceed to next state
    next_state.set(ProcGenState::Despawn);
    if init_state.get() != &ProcGenInit(true) {
        next_init_state.set(ProcGenInit(true));
    }
}
