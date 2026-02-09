/*
 * File: characters.rs
 * Author: Leopold Johannes Meinel (leo@meinel.dev)
 * -----
 * Copyright (c) 2026 Leopold Johannes Meinel & contributors
 * SPDX ID: Apache-2.0
 * URL: https://www.apache.org/licenses/LICENSE-2.0
 */

use bevy::prelude::*;
use bevy_prng::WyRand;
use rand::{Rng as _, seq::IndexedRandom as _};

use crate::{
    animations::{ANIMATION_DELAY_RANGE_SECS, AnimationRng, Animations},
    characters::{Character, CollisionDataCache},
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
    level: Single<Entity, With<B>>,
    mut animation_rng: Single<&mut WyRand, (With<AnimationRng>, Without<ProcGenRng>)>,
    mut procgen_rng: Single<&mut WyRand, (With<ProcGenRng>, Without<AnimationRng>)>,
    mut commands: Commands,
    mut object_cache: ResMut<ProcGenCache<T>>,
    animations: Res<Animations<T>>,
    collision_data: Res<CollisionDataCache<T>>,
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
        let animation_delay = animation_rng.random_range(ANIMATION_DELAY_RANGE_SECS);
        let target_pos = world_pos + origin * tile_data.tile_size;
        let collision_set = (
            collision_data.shape.clone(),
            collision_data.width,
            collision_data.height,
        );
        let entity = T::spawn(
            &mut commands,
            &collision_set,
            target_pos,
            &animations,
            animation_delay,
        );
        object_cache.chunk_positions.insert(entity, event.chunk_pos);

        // Add entity to level so that level handles despawning
        commands.entity(*level).add_child(entity);
    }
}
