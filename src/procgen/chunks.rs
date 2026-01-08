/*
 * File: chunks.rs
 * Author: Leopold Johannes Meinel (leo@meinel.dev)
 * -----
 * Copyright (c) 2026 Leopold Johannes Meinel & contributors
 * SPDX ID: Apache-2.0
 * URL: https://www.apache.org/licenses/LICENSE-2.0
 */

use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;
use bevy_prng::WyRand;
use rand::Rng as _;

use crate::{
    camera::LEVEL_Z,
    levels::{Level, LevelAssets},
    procgen::{
        CHUNK_SIZE, PROCGEN_DISTANCE, ProcGenController, ProcGenRng, ProcGenState, ProcGenerated,
        TileDataCache,
    },
};

/// Spawn chunks around the camera
///
/// ## Traits
///
/// - `T` must implement [`ProcGenerated`]' and is used as the procedurally generated level associated with a [`ProcGenController<T>`].
/// - `A` must implement [`LevelAssets`] and is used as a level's assets.
/// - `B` must implement [`Level`].
pub(crate) fn spawn_chunks<T, A, B>(
    level: Single<Entity, With<B>>,
    mut rng: Single<&mut WyRand, With<ProcGenRng>>,
    mut commands: Commands,
    mut controller: ResMut<ProcGenController<T>>,
    mut next_state: ResMut<NextState<ProcGenState>>,
    tile_data: Res<TileDataCache<T>>,
    assets: Res<A>,
) where
    T: ProcGenerated,
    A: LevelAssets,
    B: Level,
{
    // Spawn chunk behind and in front of camera chunk position if it does not contain a chunk already
    // NOTE: We are using inclusive range because we might have a movement offset of 1 chunk.
    // NOTE: We are spawning in a square. Since that has only minimal performance overhead.
    //       I deem this a cleaner solution and if spawning in a circle, distance calculations
    //       would be more expensive.
    let chunk_pos = controller.camera_chunk_pos;
    for y in (chunk_pos.y - PROCGEN_DISTANCE)..=(chunk_pos.y + PROCGEN_DISTANCE) {
        for x in (chunk_pos.x - PROCGEN_DISTANCE)..=(chunk_pos.x + PROCGEN_DISTANCE) {
            // Continue if a chunk has already been stored
            if controller
                .chunk_positions
                .values()
                .any(|&v| v == IVec2::new(x, y))
            {
                continue;
            }

            // FIXME: Currently this just chooses from a range of random numbers.
            //        Make this choose from _tiles in a way that makes sense with noise.
            let rand_index = rng.random_range(0_u32..15_u32);

            // Spawn chunk
            spawn_chunk::<T, A>(
                &mut commands,
                &mut controller,
                level.entity(),
                &assets,
                IVec2::new(x, y),
                tile_data.tile_size,
                TileTextureIndex(rand_index),
            );
        }
    }

    next_state.set(ProcGenState::MoveNavMesh);
}

/// Spawn a single chunk
///
/// ## Traits
///
/// - `T` must implement [`ProcGenerated`]' and is used as a level's procedurally generated item.
/// - `A` must implement [`LevelAssets`] and is used as a level's assets.
fn spawn_chunk<T, A>(
    commands: &mut Commands,
    controller: &mut ResMut<ProcGenController<T>>,
    level: Entity,
    assets: &Res<A>,
    chunk_pos: IVec2,
    tile_size: f32,
    texture_index: TileTextureIndex,
) where
    T: ProcGenerated,
    A: LevelAssets,
{
    // Create empty container and store in controller
    let container = commands.spawn(T::default()).id();
    controller.chunk_positions.insert(container, chunk_pos);
    let mut storage = TileStorage::empty(CHUNK_SIZE.into());

    // Spawn a `TileBundle` mapped to the container entity for each x/y in `CHUNK_SIZE`,
    // add as child to container entity and add to storage.
    for x in 0..CHUNK_SIZE.x {
        for y in 0..CHUNK_SIZE.y {
            let tile_pos = TilePos { x, y };
            let entity = commands
                .spawn((TileBundle {
                    position: tile_pos,
                    texture_index,
                    tilemap_id: TilemapId(container),
                    ..default()
                },))
                .id();
            commands.entity(container).add_child(entity);
            storage.set(&tile_pos, entity);
        }
    }

    let world_pos = chunk_pos.as_vec2() * CHUNK_SIZE.as_vec2() * tile_size;
    let handle = assets.get_tile_set().clone();

    // Insert TileMapBundle with storage, transform and texture from handle to container entity
    let tile_size_vec = Vec2::splat(tile_size);
    commands.entity(container).insert(TilemapBundle {
        grid_size: tile_size_vec.into(),
        size: CHUNK_SIZE.into(),
        storage,
        texture: TilemapTexture::Single(handle),
        tile_size: tile_size_vec.into(),
        transform: Transform::from_translation(world_pos.extend(LEVEL_Z)),
        render_settings: TilemapRenderSettings {
            render_chunk_size: CHUNK_SIZE,
            y_sort: false,
        },
        ..default()
    });

    // Add chunk container to level so that level handles despawning
    commands.entity(level).add_child(container);
}
