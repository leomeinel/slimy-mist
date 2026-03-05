/*
 * File: lighting.rs
 * Author: Leopold Johannes Meinel (leo@meinel.dev)
 * -----
 * Copyright (c) 2026 Leopold Johannes Meinel & contributors
 * SPDX ID: Apache-2.0
 * URL: https://www.apache.org/licenses/LICENSE-2.0
 */

use bevy::prelude::*;
use bevy_prng::WyRand;
use rand::seq::IndexedRandom as _;

use crate::{
    levels::Level,
    lighting::LightWrapper,
    procgen::{CHUNK_SIZE, ProcGen, ProcGenCache, ProcGenRng, ProcGenerated, TileDataCache},
    visual::Visible,
};

/// Number of lights to spawn per chunk
const LIGHTS_PER_CHUNK: usize = 4;

/// Spawn lights in a chunk
///
/// ## Traits
///
/// - `T` must implement [`LightWrapper`] + [`ProcGenerated`] + [`Visible`] and is used as the procedurally generated object associated with a [`ProcGenCache<T>`].
/// - `A` must implement [`ProcGenerated`] and is used as a level's procedurally generated item.
/// - `B` must implement [`Level`].
pub(crate) fn spawn_on_procgen_lights<T, A, B>(
    event: On<ProcGen<T>>,
    level: Single<Entity, With<B>>,
    mut procgen_rng: Single<&mut WyRand, With<ProcGenRng>>,
    mut commands: Commands,
    mut object_cache: ResMut<ProcGenCache<T>>,
    tile_data: Res<TileDataCache<A>>,
) where
    T: LightWrapper + ProcGenerated + Visible,
    A: ProcGenerated,
    B: Level,
{
    let world_pos = event.chunk_pos.as_vec2() * CHUNK_SIZE.as_vec2() * tile_data.tile_size;

    // Choose a number of target chunk tile origins to determine spawn positions
    let target_origins: Vec<(u32, u32)> = (0..CHUNK_SIZE.x)
        .flat_map(|x| (0..CHUNK_SIZE.y).map(move |y| (x, y)))
        .collect();
    let target_origins: Vec<Vec2> = target_origins
        .sample(&mut procgen_rng, LIGHTS_PER_CHUNK)
        .map(|&(x, y)| Vec2::new(x as f32, y as f32))
        .collect();

    for origin in target_origins {
        // Spawn entity in chosen tile and store in `object_cache`
        let target_pos = world_pos + origin * tile_data.tile_size;
        let entity = T::default().spawn(&mut commands, target_pos);
        object_cache.chunk_positions.insert(entity, event.chunk_pos);

        // Add entity to level so that level handles despawning
        commands.entity(*level).add_child(entity);
    }
}
