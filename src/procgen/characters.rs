/*
 * File: characters.rs
 * Author: Leopold Johannes Meinel (leo@meinel.dev)
 * -----
 * Copyright (c) 2026 Leopold Johannes Meinel & contributors
 * SPDX ID: Apache-2.0
 * URL: https://www.apache.org/licenses/LICENSE-2.0
 */

use std::marker::PhantomData;

use bevy::prelude::*;
use bevy_prng::WyRand;
use rand::seq::IndexedRandom as _;

use crate::{
    characters::{Character, SpawnCharacter},
    levels::Level,
    procgen::{CHUNK_SIZE, ProcGen, ProcGenCache, ProcGenRng, ProcGenerated, TileDataCache},
};

/// Number of characters to spawn per chunk
const CHARACTERS_PER_CHUNK: usize = 1;

/// Spawn characters in a chunk
///
/// ## Traits
///
/// - `T` must implement [`Character`] + [`ProcGenerated`] and is used as the procedurally generated object associated with a [`ProcGenCache<T>`].
/// - `A` must implement [`ProcGenerated`] and is used as a level's procedurally generated item.
/// - `B` must implement [`Level`].
pub(crate) fn on_procgen_characters<T, A, B>(
    event: On<ProcGen<T>>,
    mut procgen_rng: Single<&mut WyRand, With<ProcGenRng>>,
    mut commands: Commands,
    mut object_cache: ResMut<ProcGenCache<T>>,
    tile_data: Res<TileDataCache<A>>,
) where
    T: Character + ProcGenerated,
    A: ProcGenerated,
    B: Level,
{
    let world_pos = event.chunk_pos.as_vec2() * CHUNK_SIZE.as_vec2() * tile_data.tile_size;

    // Choose a number of target chunk tile origins to determine spawn positions
    let target_origins: Vec<(u32, u32)> = (0..CHUNK_SIZE.x)
        .flat_map(|x| (0..CHUNK_SIZE.y).map(move |y| (x, y)))
        .collect();
    let target_origins: Vec<Vec2> = target_origins
        .choose_multiple(&mut procgen_rng, CHARACTERS_PER_CHUNK)
        .map(|&(x, y)| Vec2::new(x as f32, y as f32))
        .collect();

    for origin in target_origins {
        // Spawn entity in chosen tile and store in `object_cache`
        let entity = commands.spawn(T::default()).id();
        let pos = world_pos + origin * tile_data.tile_size;
        commands.trigger(SpawnCharacter::<T, B> {
            entity,
            pos,
            _phantom: PhantomData,
        });
        object_cache.chunk_positions.insert(entity, event.chunk_pos);
    }
}
