/*
 * File: chunks.rs
 * Author: Leopold Johannes Meinel (leo@meinel.dev)
 * -----
 * Copyright (c) 2025 Leopold Johannes Meinel & contributors
 * SPDX ID: Apache-2.0
 * URL: https://www.apache.org/licenses/LICENSE-2.0
 */

use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;

use crate::{
    CanvasCamera,
    levels::{LEVEL_Z, LevelAssets, RENDER_DISTANCE},
    logging::{error::ERR_LOADING_TILE_DATA, warn::WARN_INCOMPLETE_TILE_DATA},
    procgen::{ProcGenController, ProcGenTimer, ProcGenerated, TileData, TileHandle},
    screens::Screen,
};

/// Size of a single chunk
pub(crate) const CHUNK_SIZE: UVec2 = UVec2 { x: 16, y: 16 };

/// Chunk size for [`TilemapRenderSettings`]
const RENDER_CHUNK_SIZE: UVec2 = UVec2 {
    x: CHUNK_SIZE.x * 2,
    y: CHUNK_SIZE.y * 2,
};

/// Spawn chunks around the [`CanvasCamera`]
pub(crate) fn spawn_chunks<T, A>(
    camera: Single<&Transform, With<CanvasCamera>>,
    mut commands: Commands,
    mut controller: ResMut<ProcGenController<T>>,
    data: Res<Assets<TileData<T>>>,
    handle: Res<TileHandle<T>>,
    assets: Res<A>,
    timer: Res<ProcGenTimer>,
) where
    T: ProcGenerated, // Procedurally generated level
    A: LevelAssets,   // Level assets
{
    // Return if timer has not finished
    if !timer.0.just_finished() {
        return;
    }

    // Get data from `TileData` with `TileHandle`
    let data = data.get(handle.0.id()).expect(ERR_LOADING_TILE_DATA);
    let tile_size = IVec2::new(data.tile_height as i32, data.tile_width as i32);
    // FIXME: Use this for conditional spawning/arranging
    let Some(_tiles) = data.get_tiles() else {
        // Return and do not spawn chunks if tiles are not configured correctly
        warn_once!("{}", WARN_INCOMPLETE_TILE_DATA);
        return;
    };

    // Get target translation for new chunk from camera translation
    let camera_pos = &camera.translation.xy().as_ivec2();
    let chunk_size = IVec2::new(CHUNK_SIZE.x as i32, CHUNK_SIZE.y as i32);
    let chunk_pos = camera_pos / (chunk_size * tile_size);

    // Spawn chunk behind and in front of chunk position if it does not contain a chunk already
    for y in (chunk_pos.y - RENDER_DISTANCE)..(chunk_pos.y + RENDER_DISTANCE) {
        for x in (chunk_pos.x - RENDER_DISTANCE)..(chunk_pos.x + RENDER_DISTANCE) {
            // Continue if a character has already been stored in chunk or store pixel origin of tile
            if controller.positions.contains(&IVec2::new(x, y)) {
                continue;
            }
            controller.positions.insert(IVec2::new(x, y));

            // Spawn chunk
            spawn_chunk::<T, A>(
                &mut commands,
                &assets,
                IVec2::new(x, y),
                tile_size.as_vec2(),
                TileTextureIndex(8),
            );
        }
    }
}

/// Spawn a single chunk
fn spawn_chunk<T, A>(
    commands: &mut Commands,
    assets: &Res<A>,
    chunk_pos: IVec2,
    tile_size: Vec2,
    texture_index: TileTextureIndex,
) where
    T: ProcGenerated, // Procedurally generated level
    A: LevelAssets,   // Level assets
{
    // Create empty entity and storage dedicated to this chunk
    let container = commands.spawn(DespawnOnExit(Screen::Gameplay)).id();
    let mut storage = TileStorage::empty(CHUNK_SIZE.into());

    // Spawn a `TileBundle` mapped to the container entity for each x/y in `CHUNK_SIZE`,
    // add as child to container entity and add to storage.
    for x in 0..CHUNK_SIZE.x {
        for y in 0..CHUNK_SIZE.y {
            let tile_pos = TilePos { x, y };
            let entity = commands
                .spawn((
                    T::default(),
                    TileBundle {
                        position: tile_pos,
                        texture_index,
                        tilemap_id: TilemapId(container),
                        ..default()
                    },
                ))
                .id();
            commands.entity(container).add_child(entity);
            storage.set(&tile_pos, entity);
        }
    }

    let transform = Transform::from_xyz(
        chunk_pos.x as f32 * CHUNK_SIZE.x as f32 * tile_size.x,
        chunk_pos.y as f32 * CHUNK_SIZE.y as f32 * tile_size.y,
        LEVEL_Z,
    );
    let handle = assets.get_tile_set().clone();

    // Insert TileMapBundle with storage, transform and texture from handle to container entity
    commands.entity(container).insert(TilemapBundle {
        grid_size: tile_size.into(),
        size: CHUNK_SIZE.into(),
        storage,
        texture: TilemapTexture::Single(handle),
        tile_size: tile_size.into(),
        transform,
        render_settings: TilemapRenderSettings {
            render_chunk_size: RENDER_CHUNK_SIZE,
            y_sort: false,
        },
        ..default()
    });
}
