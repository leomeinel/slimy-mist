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
    levels::{LEVEL_Z, Level, LevelAssets},
    logging::{error::ERR_LOADING_TILE_DATA, warn::WARN_INCOMPLETE_TILE_DATA},
    procgen::{
        CHUNK_SIZE, PROCGEN_DISTANCE, ProcGenController, ProcGenTimer, ProcGenerated, TileData,
        TileHandle, navigation::chunk_mesh,
    },
};

/// Spawn chunks around the [`CanvasCamera`]
///
/// ## Traits
///
/// - `T` must implement [`ProcGenerated`]' and is used as the procedurally generated level associated with a [`ProcGenController<T>`].
/// - `A` must implement [`LevelAssets`] and is used as a level's assets.
/// - `B` must implement [`Level`].
pub(crate) fn spawn_chunks<T, A, B>(
    camera: Single<(&Transform, Ref<Transform>), With<CanvasCamera>>,
    level: Single<Entity, With<B>>,
    mut commands: Commands,
    mut controller: ResMut<ProcGenController<T>>,
    data: Res<Assets<TileData<T>>>,
    handle: Res<TileHandle<T>>,
    assets: Res<A>,
    timer: Res<ProcGenTimer>,
) where
    T: ProcGenerated,
    A: LevelAssets,
    B: Level,
{
    // Return if timer has not finished
    if !timer.0.just_finished() {
        return;
    }

    let (camera, ref_camera) = camera.into_inner();

    // Return if camera transform has not changed
    if !ref_camera.is_changed() {
        return;
    }

    // Get data from `TileData` with `TileHandle`
    let data = data.get(handle.0.id()).expect(ERR_LOADING_TILE_DATA);
    let tile_size = Vec2::new(data.tile_height, data.tile_width);
    // FIXME: Use this for conditional spawning/arranging
    let Some(_tiles) = data.get_tiles() else {
        // Return and do not spawn chunks if tiles are not configured correctly
        warn_once!("{}", WARN_INCOMPLETE_TILE_DATA);
        return;
    };

    // Get target translation for new chunk from camera translation
    let camera_pos = camera.translation.xy();
    let chunk_size_px = Vec2::new(CHUNK_SIZE.x as f32, CHUNK_SIZE.y as f32) * tile_size;
    let chunk_pos = (camera_pos / chunk_size_px).floor().as_ivec2();

    // Spawn chunk behind and in front of chunk position if it does not contain a chunk already
    // NOTE: We are using inclusive range because we might have a movement offset of 1 chunk.
    // NOTE: We are spawning in a square. Since that has only minimal performance overhead.
    //       I deem this a cleaner solution and if spawning in a circle, distance calculations
    //       would be more expensive.
    for y in (chunk_pos.y - PROCGEN_DISTANCE)..=(chunk_pos.y + PROCGEN_DISTANCE) {
        for x in (chunk_pos.x - PROCGEN_DISTANCE)..=(chunk_pos.x + PROCGEN_DISTANCE) {
            // Continue if a chunk has already been stored
            if controller
                .positions
                .values()
                .any(|&v| v == IVec2::new(x, y))
            {
                continue;
            }

            // Spawn chunk
            spawn_chunk::<T, A>(
                &mut commands,
                &mut controller,
                level.entity(),
                &assets,
                IVec2::new(x, y),
                tile_size,
                TileTextureIndex(8),
            );
        }
    }
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
    controller.positions.insert(container, chunk_pos);
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

    let world_pos = Vec2::new(
        chunk_pos.x as f32 * CHUNK_SIZE.x as f32 * tile_size.x,
        chunk_pos.y as f32 * CHUNK_SIZE.y as f32 * tile_size.y,
    );
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

    // Add chunk container and nav mesh to level so that level handles despawning
    let nav_mesh = commands.spawn(chunk_mesh(world_pos, tile_size.x)).id();
    commands.entity(level).add_children(&[container, nav_mesh]);
}
