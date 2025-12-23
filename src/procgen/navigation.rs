/*
 * File: navigation.rs
 * Author: Leopold Johannes Meinel (leo@meinel.dev)
 * -----
 * Copyright (c) 2025 Leopold Johannes Meinel & contributors
 * SPDX ID: Apache-2.0
 * URL: https://www.apache.org/licenses/LICENSE-2.0
 * -----
 * Heavily inspired by: https://github.com/vleue/vleue_navigator
 */

use bevy::prelude::*;
use polyanya::Triangulation;
use vleue_navigator::prelude::*;

use crate::procgen::CHUNK_SIZE;

pub(super) fn plugin(app: &mut App) {
    // Add libraries
    app.add_plugins((
        VleueNavigatorPlugin,
        NavmeshUpdaterPlugin::<PrimitiveObstacle>::default(),
    ));
}

/// Simplification used in [`NavMeshSettings`]
const OBSTACLE_SIMPLIFICATION: f32 = 0.05;
/// Update interval for [`NavMeshUpdateMode::Debounced`] that is applied to [`chunk_mesh`]
const DEBOUNCED_UPDATE_INTERVAL: f32 = 0.5;

/// Mesh for a single chunk
pub(crate) fn chunk_mesh(world_pos: Vec2, scale_factor: f32) -> impl Bundle {
    (
        NavMeshSettings {
            simplify: OBSTACLE_SIMPLIFICATION,
            fixed: Triangulation::from_outer_edges(&[
                Vec2::new(0., 0.),
                Vec2::new(CHUNK_SIZE.x as f32, 0.),
                Vec2::new(CHUNK_SIZE.x as f32, CHUNK_SIZE.y as f32),
                Vec2::new(0., CHUNK_SIZE.y as f32),
            ]),
            ..default()
        },
        NavMeshUpdateMode::Debounced(DEBOUNCED_UPDATE_INTERVAL),
        Transform::from_xyz(
            world_pos.x - scale_factor / 2.,
            world_pos.y - scale_factor / 2.,
            0.,
        )
        .with_scale(Vec3::splat(scale_factor)),
    )
}
