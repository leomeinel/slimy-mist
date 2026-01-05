/*
 * File: spawn.rs
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
    characters::{
        Character, CollisionData, CollisionHandle, VisualMap,
        animations::{ANIMATION_DELAY_RANGE, AnimationRng, Animations},
    },
    levels::Level,
    logging::error::{ERR_LOADING_COLLISION_DATA, ERR_LOADING_TILE_DATA},
    procgen::{CHUNK_SIZE, ProcGenController, ProcGenRng, ProcGenerated, TileData, TileHandle},
};

/// Spawn characters in every chunk contained in [`ProcGenController<A>`]
///
/// ## Traits
///
/// - `T` must implement [`Character`] and [`ProcGenerated`] and is used as the procedurally generated character associated with a [`ProcGenController<T>`].
/// - `A` must implement [`ProcGenerated`] and is used as a level's procedurally generated item.
/// - `B` must implement [`Level`].
pub(crate) fn spawn_characters<T, A, B>(
    mut animation_rng: Single<&mut WyRand, (With<AnimationRng>, Without<ProcGenRng>)>,
    mut procgen_rng: Single<&mut WyRand, (With<ProcGenRng>, Without<AnimationRng>)>,
    level: Single<Entity, With<B>>,
    mut commands: Commands,
    mut controller: ResMut<ProcGenController<T>>,
    mut visual_map: ResMut<VisualMap>,
    animations: Res<Animations<T>>,
    chunk_controller: Res<ProcGenController<A>>,
    collision_data: Res<Assets<CollisionData<T>>>,
    collision_handle: Res<CollisionHandle<T>>,
    tile_data: Res<Assets<TileData<A>>>,
    tile_handle: Res<TileHandle<A>>,
    mut collision_set: Local<Option<(Option<String>, Option<f32>, Option<f32>)>>,
    mut tile_size: Local<Option<f32>>,
) where
    T: Character + ProcGenerated,
    A: ProcGenerated,
    B: Level,
{
    // Init local values
    let tile_size = tile_size.unwrap_or_else(|| {
        let data = tile_data
            .get(tile_handle.0.id())
            .expect(ERR_LOADING_TILE_DATA);
        let value = data.tile_size;
        *tile_size = Some(value);
        value
    });
    if collision_set.is_none() {
        let data = collision_data
            .get(collision_handle.0.id())
            .expect(ERR_LOADING_COLLISION_DATA);
        let value = (data.shape.clone(), data.width, data.height);
        *collision_set = Some(value);
    }
    let collision_set = collision_set.as_ref().unwrap();

    for (_, chunk_pos) in &chunk_controller.chunk_positions {
        // Continue if chunk has already been stored
        if controller
            .chunk_positions
            .values()
            .any(|&v| v == *chunk_pos)
        {
            continue;
        }

        // Spawn character
        spawn_character::<T>(
            &mut animation_rng,
            &mut procgen_rng,
            &mut commands,
            &mut controller,
            &mut visual_map,
            level.entity(),
            &animations,
            collision_set,
            chunk_pos,
            tile_size,
        );
    }
}

/// Number of characters to spawn per chunk
const CHARACTERS_PER_CHUNK: usize = 4;

/// Spawn characters in a chunk
///
/// ## Traits
///
/// - `T` must implement [`Character`] + [`ProcGenerated`] and is used as the procedurally generated character.
fn spawn_character<T>(
    animation_rng: &mut WyRand,
    procgen_rng: &mut WyRand,
    commands: &mut Commands,
    controller: &mut ResMut<ProcGenController<T>>,
    visual_map: &mut ResMut<VisualMap>,
    level: Entity,
    animations: &Res<Animations<T>>,
    collision_set: &(Option<String>, Option<f32>, Option<f32>),
    chunk_pos: &IVec2,
    tile_size: f32,
) where
    T: Character + ProcGenerated,
{
    // Choose a number of target chunk tile origins to determine spawn positions
    let target_origins: Vec<(u32, u32)> = (0..CHUNK_SIZE.x)
        .flat_map(|x| (0..CHUNK_SIZE.y).map(move |y| (x, y)))
        .collect();
    let target_origins: Vec<Vec2> = target_origins
        .choose_multiple(procgen_rng, CHARACTERS_PER_CHUNK)
        .map(|&(x, y)| Vec2::new(x as f32, y as f32))
        .collect();

    for origin in target_origins {
        let animation_delay = animation_rng.random_range(ANIMATION_DELAY_RANGE);

        // Set target position in pixels
        let world_pos = chunk_pos.as_vec2() * CHUNK_SIZE.as_vec2() * tile_size;
        let target_pos = world_pos + origin * tile_size;

        // Spawn entity in chosen tile and store in controller
        let entity = T::spawn(
            commands,
            visual_map,
            collision_set,
            target_pos,
            animations,
            animation_delay,
        );
        controller.chunk_positions.insert(entity, *chunk_pos);

        // Add entity to level so that level handles despawning
        commands.entity(level).add_child(entity);
    }
}
