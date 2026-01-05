/*
 * File: navmesh.rs
 * Author: Leopold Johannes Meinel (leo@meinel.dev)
 * -----
 * Copyright (c) 2026 Leopold Johannes Meinel & contributors
 * SPDX ID: Apache-2.0
 * URL: https://www.apache.org/licenses/LICENSE-2.0
 * -----
 * Heavily inspired by: https://github.com/JtotheThree/bevy_northstar
 */

use bevy::prelude::*;
use polyanya::Triangulation;
use vleue_navigator::prelude::*;

use crate::{
    levels::Level,
    logging::error::ERR_LOADING_TILE_DATA,
    procgen::{CHUNK_SIZE, PROCGEN_DISTANCE, ProcGenerated, TileData, TileHandle},
};

pub(super) fn plugin(app: &mut App) {
    // Add north star plugin
    app.add_plugins((
        VleueNavigatorPlugin,
        NavmeshUpdaterPlugin::<PrimitiveObstacle>::default(),
    ));
}

/// Size of the [`ManagedNavMesh`]
const NAVMESH_SIZE: UVec2 = UVec2::new(
    CHUNK_SIZE.x * (PROCGEN_DISTANCE as u32 * 2 + 1),
    CHUNK_SIZE.y * (PROCGEN_DISTANCE as u32 * 2 + 1),
);

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

    let entity = commands
        .spawn((
            NavMeshSettings {
                simplify: 0.05,
                fixed: Triangulation::from_outer_edges(&[
                    Vec2::ZERO,
                    Vec2::new(NAVMESH_SIZE.x as f32, 0.),
                    NAVMESH_SIZE.as_vec2(),
                    Vec2::new(0., NAVMESH_SIZE.y as f32),
                ]),
                ..default()
            },
            NavMeshUpdateMode::Debounced(0.5),
            // NOTE: This seems to be anchored to the bottom left. This means that we have to offset it by:
            //       `NAVMESH_SIZE` / 2 + (one chunk - 1 tile) / 2 as world pos.
            //       This seemingly weird calculation is in part due to the chunk spawning at `0,0` having its
            //       minimum tile centered at `0,0`, not the chunk itself.
            //       Otherwise we would only have to offset it by: `NAVMESH_SIZE` / 2 as world pos.
            Transform::from_translation(
                ((-NAVMESH_SIZE.as_vec2() / 2. + (CHUNK_SIZE.as_vec2() - 1.) / 2.) * tile_size)
                    .extend(0.),
            )
            .with_scale(Vec3::splat(tile_size)),
        ))
        .id();

    commands.entity(level.entity()).add_child(entity);
}
