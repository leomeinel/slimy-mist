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
    levels::{DEFAULT_Z, Level},
    logging::error::{ERR_LOADING_COLLISION_DATA, ERR_LOADING_TILE_DATA},
    procgen::{
        Despawnable, ProcGenController, ProcGenRng, ProcGenTimer, TileData, TileHandle,
        chunks::CHUNK_SIZE,
    },
};

/// Spawn characters in every chunk contained in [`ProcGenController<A>`]
pub(crate) fn spawn_characters<T, A, B>(
    mut animation_rng: Single<&mut WyRand, (With<AnimationRng>, Without<ProcGenRng>)>,
    mut rng: Single<&mut WyRand, (With<ProcGenRng>, Without<AnimationRng>)>,
    level: Single<Entity, With<B>>,
    mut commands: Commands,
    mut controller: ResMut<ProcGenController<T>>,
    mut visual_map: ResMut<VisualMap>,
    animations: Res<Animations<T>>,
    chunk_controller: Res<ProcGenController<A>>,
    collision_data: Res<Assets<CollisionData<T>>>,
    collision_handle: Res<CollisionHandle<T>>,
    shadow: Res<Shadow<T>>,
    tile_data: Res<Assets<TileData<A>>>,
    tile_handle: Res<TileHandle<A>>,
    timer: Res<ProcGenTimer>,
) where
    T: Character + Despawnable, // Despawnable of the character that is also the character marker
    A: Despawnable,             // Despawnable of the level
    B: Level,                   // Container level
{
    // Return if timer has not finished
    if !timer.0.just_finished() {
        return;
    }

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
    for chunk_pos in &chunk_controller.positions {
        let target_pos = &Vec2::new(
            (chunk_pos.x as f32 * CHUNK_SIZE.x as f32 * tile_size.x).floor(),
            (chunk_pos.y as f32 * CHUNK_SIZE.y as f32 * tile_size.y).floor(),
        );
        // Continue if a character has already been stored in chunk or store pixel origin of tile
        if controller.positions.contains(chunk_pos) {
            continue;
        }
        controller.positions.insert(*chunk_pos);

        // Spawn character
        spawn_character::<T>(
            &mut animation_rng,
            &mut rng,
            &mut commands,
            &mut visual_map,
            level.entity(),
            &animations,
            &data,
            &shadow,
            target_pos,
            &tile_size,
        );
    }
}

/// Number of characters to spawn per chunk
const CHARACTERS_PER_CHUNK: usize = 4;

// FIXME: There has to be some logic error. Spawning seems to not actually use random positions.
//        Character spawn on top of each other or pile up.
/// Spawn characters in a chunk
fn spawn_character<T>(
    animation_rng: &mut WyRand,
    rng: &mut WyRand,
    commands: &mut Commands,
    visual_map: &mut ResMut<VisualMap>,
    container: Entity,
    animations: &Res<Animations<T>>,
    data: &(Option<String>, Option<f32>, Option<f32>),
    shadow: &Res<Shadow<T>>,
    chunk_pos: &Vec2,
    tile_size: &Vec2,
) where
    T: Character + Despawnable, // Despawnable of the character that is also the character marker
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

        // Multiply by tile size because chunk_pos only stores pixel origins of tiles
        let spawn_pos = Vec3::new(
            chunk_pos.x + *x as f32 * tile_size.x,
            chunk_pos.y + *y as f32 * tile_size.y,
            DEFAULT_Z,
        );

        // Spawn character in chosen tile
        let entity = T::spawn(
            commands, visual_map, data, spawn_pos, animations, shadow, delay,
        );
        commands.entity(container).add_child(entity);
    }
}
