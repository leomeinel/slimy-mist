/*
 * File: spawn.rs
 * Author: Leopold Johannes Meinel (leo@meinel.dev)
 * -----
 * Copyright (c) 2025 Leopold Johannes Meinel & contributors
 * SPDX ID: Apache-2.0
 * URL: https://www.apache.org/licenses/LICENSE-2.0
 */

use bevy::prelude::*;
use bevy_prng::WyRand;
use rand::{Rng as _, seq::IndexedRandom as _};

use crate::{
    characters::{
        Character, CollisionData, CollisionHandle, Shadow, VisualMap,
        animations::{ANIMATION_DELAY_RANGE, AnimationRng, Animations},
    },
    levels::Level,
    logging::error::{ERR_LOADING_COLLISION_DATA, ERR_LOADING_TILE_DATA},
    procgen::{
        CHUNK_SIZE, ProcGenController, ProcGenRng, ProcGenState, ProcGenerated, TileData,
        TileHandle,
    },
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
    mut rng: Single<&mut WyRand, (With<ProcGenRng>, Without<AnimationRng>)>,
    level: Single<Entity, With<B>>,
    mut commands: Commands,
    mut controller: ResMut<ProcGenController<T>>,
    mut procgen_state: ResMut<NextState<ProcGenState>>,
    mut visual_map: ResMut<VisualMap>,
    animations: Res<Animations<T>>,
    chunk_controller: Res<ProcGenController<A>>,
    collision_data: Res<Assets<CollisionData<T>>>,
    collision_handle: Res<CollisionHandle<T>>,
    shadow: Res<Shadow<T>>,
    tile_data: Res<Assets<TileData<A>>>,
    tile_handle: Res<TileHandle<A>>,
) where
    T: Character + ProcGenerated,
    A: ProcGenerated,
    B: Level,
{
    // Get data from `TileData` with `TileHandle`
    let data = tile_data
        .get(tile_handle.0.id())
        .expect(ERR_LOADING_TILE_DATA);
    let tile_size = Vec2::new(data.tile_height, data.tile_width);
    // Get data from `CollisionData` with `CollisionHandle`
    let data = collision_data
        .get(collision_handle.0.id())
        .expect(ERR_LOADING_COLLISION_DATA);
    let data = (data.shape.clone(), data.width, data.height);

    // FIXME: Use noise for spawning positions
    for (_, chunk_pos) in &chunk_controller.positions {
        // Continue if chunk has already been stored
        if controller.positions.values().any(|&v| v == *chunk_pos) {
            continue;
        }

        // Spawn character
        spawn_character::<T>(
            &mut animation_rng,
            &mut rng,
            &mut commands,
            &mut controller,
            &mut visual_map,
            level.entity(),
            &animations,
            &data,
            &shadow,
            chunk_pos,
            &tile_size,
        );
    }

    procgen_state.set(ProcGenState::RebuildNavGrid);
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
    rng: &mut WyRand,
    commands: &mut Commands,
    controller: &mut ResMut<ProcGenController<T>>,
    visual_map: &mut ResMut<VisualMap>,
    level: Entity,
    animations: &Res<Animations<T>>,
    data: &(Option<String>, Option<f32>, Option<f32>),
    shadow: &Res<Shadow<T>>,
    chunk_pos: &IVec2,
    tile_size: &Vec2,
) where
    T: Character + ProcGenerated,
{
    // Choose a number of target chunk tile origins to determine spawn positions
    let target_origins: Vec<(u32, u32)> = (0..CHUNK_SIZE.x)
        .flat_map(|x| (0..CHUNK_SIZE.y).map(move |y| (x, y)))
        .collect();
    let target_origins: Vec<&(u32, u32)> = target_origins
        .choose_multiple(rng, CHARACTERS_PER_CHUNK)
        .collect();

    for (x, y) in target_origins {
        let delay = animation_rng.random_range(ANIMATION_DELAY_RANGE);

        // Set target position in pixels
        let target_pos = Vec2::new(
            chunk_pos.x as f32 * CHUNK_SIZE.x as f32 * tile_size.x + *x as f32 * tile_size.x,
            chunk_pos.y as f32 * CHUNK_SIZE.y as f32 * tile_size.y + *y as f32 * tile_size.y,
        );

        // Spawn entity in chosen tile and store in controller
        let entity = T::spawn(
            commands, visual_map, data, target_pos, animations, shadow, delay,
        );
        controller.positions.insert(entity, *chunk_pos);

        // Add entity to level so that level handles despawning
        commands.entity(level).add_child(entity);
    }
}
