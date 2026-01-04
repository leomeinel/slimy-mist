/*
 * File: chunks.rs
 * Author: Leopold Johannes Meinel (leo@meinel.dev)
 * -----
 * Copyright (c) 2026 Leopold Johannes Meinel & contributors
 * SPDX ID: Apache-2.0
 * URL: https://www.apache.org/licenses/LICENSE-2.0
 */

use bevy::{platform::collections::HashSet, prelude::*};
use bevy_ecs_tilemap::prelude::*;
use bevy_prng::WyRand;
use rand::Rng as _;

use crate::{
    levels::{LEVEL_Z, Level, LevelAssets},
    logging::{error::ERR_LOADING_TILE_DATA, warn::WARN_INCOMPLETE_TILE_DATA},
    procgen::{
        CHUNK_SIZE, PROCGEN_DISTANCE, ProcGenController, ProcGenRng, ProcGenState, ProcGenerated,
        TileData, TileHandle,
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
    data: Res<Assets<TileData<T>>>,
    handle: Res<TileHandle<T>>,
    assets: Res<A>,
    mut tile_size: Local<Option<Vec2>>,
    mut tiles: Local<
        Option<(
            HashSet<UVec2>,
            HashSet<UVec2>,
            HashSet<UVec2>,
            HashSet<UVec2>,
            HashSet<UVec2>,
            HashSet<UVec2>,
        )>,
    >,
) where
    T: ProcGenerated,
    A: LevelAssets,
    B: Level,
{
    // Init local values
    let tile_size = tile_size.unwrap_or_else(|| {
        let data = data.get(handle.0.id()).expect(ERR_LOADING_TILE_DATA);
        let value = Vec2::new(data.tile_height, data.tile_width);
        *tile_size = Some(value);
        value
    });
    if tiles.is_none() {
        let data = data.get(handle.0.id()).expect(ERR_LOADING_TILE_DATA);
        let Some(value) = data.get_tiles() else {
            // Return and do not spawn chunks if tiles are not configured correctly
            warn_once!("{}", WARN_INCOMPLETE_TILE_DATA);
            return;
        };
        *tiles = Some(value);
    }
    let _tiles = tiles.as_ref().unwrap();

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
                tile_size,
                TileTextureIndex(rand_index),
            );
        }
    }

    next_state.set(ProcGenState::UpdateNav);
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
    tile_size: Vec2,
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
    commands.entity(container).insert(TilemapBundle {
        grid_size: tile_size.into(),
        size: CHUNK_SIZE.into(),
        storage,
        texture: TilemapTexture::Single(handle),
        tile_size: tile_size.into(),
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
