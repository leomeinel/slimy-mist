/*
 * File: navigation.rs
 * Author: Leopold Johannes Meinel (leo@meinel.dev)
 * -----
 * Copyright (c) 2025 Leopold Johannes Meinel & contributors
 * SPDX ID: Apache-2.0
 * URL: https://www.apache.org/licenses/LICENSE-2.0
 * -----
 * Heavily inspired by: https://github.com/JtotheThree/bevy_northstar
 */

use bevy::prelude::*;
use bevy_northstar::prelude::*;

use crate::{
    characters::Character,
    levels::Level,
    logging::error::{ERR_INVALID_MINIMUM_CHUNK_POS, ERR_LOADING_TILE_DATA},
    procgen::{
        CHUNK_SIZE, PROCGEN_DISTANCE, ProcGenController, ProcGenState, ProcGenerated, TileData,
        TileHandle,
    },
};

pub(super) fn plugin(app: &mut App) {
    // Add north star plugin
    app.add_plugins(NorthstarPlugin::<OrdinalNeighborhood>::default());
}

/// Size of the [`Grid<OrdinalNeighborhood>`]
const GRID_SIZE: UVec2 = UVec2::new(
    CHUNK_SIZE.x * (PROCGEN_DISTANCE as u32 * 2 + 1),
    CHUNK_SIZE.y * (PROCGEN_DISTANCE as u32 * 2 + 1),
);

/// Replace [`Grid<OrdinalNeighborhood>`] with new grid at correct world position
///
/// ## Traits
///
/// - `T` must implement [`ProcGenerated`] and is used as a level's procedurally generated item.
pub(crate) fn spawn_nav_grid<T>(
    level: Single<Entity, (With<T>, Without<Grid<OrdinalNeighborhood>>)>,
    mut commands: Commands,
) where
    T: Level,
{
    let grid_settings = GridSettingsBuilder::new_2d(GRID_SIZE.x, GRID_SIZE.y)
        .chunk_size(CHUNK_SIZE.x)
        .default_impassable()
        .enable_collision()
        .build();
    let entity = commands
        .spawn(Grid::<OrdinalNeighborhood>::new(&grid_settings))
        .id();

    commands.entity(level.entity()).add_child(entity);
}

/// Rebuild the nav grid
///
/// Currently this sets every cell to [`Nav::Passable`], but this can in the future also include obstacle detection.
pub(crate) fn rebuild_nav_grid(
    mut grid: Single<&mut Grid<OrdinalNeighborhood>>,
    mut procgen_state: ResMut<NextState<ProcGenState>>,
    mut grid_pos: Local<UVec2>,
    mut rebuild: Local<bool>,
) {
    let range_limit = *grid_pos + CHUNK_SIZE;

    // Set every cell to passable
    for x in grid_pos.x..range_limit.x {
        for y in grid_pos.y..range_limit.y {
            let pos = UVec3::new(x, y, 0);
            // Continue if pos is already passable to avoid rebuilds
            if matches!(grid.nav(pos), Some(Nav::Passable(1))) {
                continue;
            }

            // Set `pos` to passable and set rebuild to true
            grid.set_nav(pos, Nav::Passable(1));
            *rebuild = true;
        }
    }
    grid_pos.x = range_limit.x;

    // Return if range limit x has not exceeded `GRID_SIZE`
    if range_limit.x < GRID_SIZE.x {
        return;
    }
    // Reset x, advance y and return if range limit y has not exceeded `GRID_SIZE`
    if range_limit.y < GRID_SIZE.y {
        grid_pos.x = 0;
        grid_pos.y += CHUNK_SIZE.y;
        return;
    }

    // Rebuild grid if rebuild is true
    if *rebuild {
        grid.build();
    }

    // Reset local variables
    *grid_pos = UVec2::ZERO;
    *rebuild = false;

    procgen_state.set(ProcGenState::Despawn);
}

/// Update nav grid position of [`Character`]
///
/// ## Traits
///
/// - `T` must implement [`Character`].
/// - `A` must implement [`ProcGenerated`] and is used as a level's procedurally generated item.
pub(crate) fn update_nav_grid_agent_pos<T, A>(
    grid: Single<Entity, With<Grid<OrdinalNeighborhood>>>,
    characters: Query<(Entity, &Transform, Option<&mut AgentPos>), With<T>>,
    mut commands: Commands,
    controller: Res<ProcGenController<A>>,
    data: Res<Assets<TileData<A>>>,
    handle: Res<TileHandle<A>>,
) where
    T: Character,
    A: ProcGenerated,
{
    // Get data from `TileData` with `TileHandle`
    let data = data.get(handle.0.id()).expect(ERR_LOADING_TILE_DATA);
    let tile_size = Vec2::new(data.tile_height, data.tile_width);

    // Determine minimum chunk position
    let min_chunk_pos = controller
        .positions
        .values()
        .min_by_key(|pos| (pos.x, pos.y))
        .expect(ERR_INVALID_MINIMUM_CHUNK_POS);

    // FIXME: Find a reliable way to avoid looping through all characters
    for (entity, transform, mut agent_pos) in characters {
        // Insert agent pos
        let pos = UVec2::new(
            (transform.translation.x / tile_size.x - min_chunk_pos.x as f32 * CHUNK_SIZE.x as f32)
                .floor() as u32,
            (transform.translation.y / tile_size.y - min_chunk_pos.y as f32 * CHUNK_SIZE.x as f32)
                .floor() as u32,
        );
        let Some(agent_pos) = agent_pos.as_mut() else {
            commands
                .entity(entity)
                .insert((AgentPos(pos.extend(0)), AgentOfGrid(grid.entity())));
            continue;
        };
        agent_pos.0 = pos.extend(0);
    }
}

/// Add pathfinding to [`Character`] that tracks another [`Character`]
///
/// ## Traits
///
/// - `T` must implement [`Character`] and is used as the origin entity that a path is given to.
/// - `A` must implement [`Character`] and is used as the target entity.
pub(crate) fn pathfind_to_character<T, A>(
    target: Single<&AgentPos, (With<A>, Without<T>)>,
    origins: Query<(Entity, Option<&mut Pathfind>), (With<T>, With<AgentPos>, Without<A>)>,
    mut commands: Commands,
) where
    T: Character,
    A: Character,
{
    for (entity, mut path_find) in origins {
        let Some(path_find) = path_find.as_mut() else {
            commands.entity(entity).insert(Pathfind::new(target.0));
            continue;
        };
        path_find.goal = target.0;
    }
}
