/*
 * File: nav_grid.rs
 * Author: Leopold Johannes Meinel (leo@meinel.dev)
 * -----
 * Copyright (c) 2026 Leopold Johannes Meinel & contributors
 * SPDX ID: Apache-2.0
 * URL: https://www.apache.org/licenses/LICENSE-2.0
 * -----
 * Heavily inspired by: https://github.com/JtotheThree/bevy_northstar
 */

use bevy::prelude::*;
use bevy_northstar::prelude::*;

use crate::{
    levels::Level,
    procgen::{CHUNK_SIZE, PROCGEN_DISTANCE, ProcGenInit, ProcGenState},
};

pub(super) fn plugin(app: &mut App) {
    // Add north star plugin
    app.add_plugins(NorthstarPlugin::<OrdinalNeighborhood>::default());
}

/// Size of the [`Grid<OrdinalNeighborhood>`]
const NAV_GRID_SIZE: UVec2 = UVec2::new(
    CHUNK_SIZE.x * (PROCGEN_DISTANCE as u32 * 2 + 1),
    CHUNK_SIZE.y * (PROCGEN_DISTANCE as u32 * 2 + 1),
);

/// Replace [`Grid<OrdinalNeighborhood>`] with new grid at correct world position
///
/// ## Traits
///
/// - `T` must implement [`Level`].
pub(crate) fn spawn_nav_grid<T>(
    level: Single<Entity, (With<T>, Without<Grid<OrdinalNeighborhood>>)>,
    mut commands: Commands,
) where
    T: Level,
{
    let grid_settings = GridSettingsBuilder::new_2d(NAV_GRID_SIZE.x, NAV_GRID_SIZE.y)
        .chunk_size(CHUNK_SIZE.x)
        .default_impassable()
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
    mut next_state: ResMut<NextState<ProcGenState>>,
    mut next_init_state: ResMut<NextState<ProcGenInit>>,
    init_state: Res<State<ProcGenInit>>,
    mut grid_pos: Local<UVec2>,
    mut rebuild: Local<bool>,
) {
    let range_limit = *grid_pos + CHUNK_SIZE;

    // Set every cell to passable
    for x in grid_pos.x..range_limit.x {
        for y in grid_pos.y..range_limit.y {
            let pos = UVec3::new(x, y, 0);

            if matches!(grid.nav(pos), Some(Nav::Passable(1))) {
                continue;
            }
            grid.set_nav(pos, Nav::Passable(1));
            *rebuild = true;
        }
    }
    grid_pos.x = range_limit.x;

    // Return if range limit x has not exceeded `GRID_SIZE`
    if range_limit.x < NAV_GRID_SIZE.x {
        return;
    }
    // Reset x, advance y and return if range limit y has not exceeded `GRID_SIZE`
    if range_limit.y < NAV_GRID_SIZE.y {
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

    // Skip following runs
    next_state.set(ProcGenState::None);
    if init_state.get() != &ProcGenInit(true) {
        next_init_state.set(ProcGenInit(true));
    }
}
