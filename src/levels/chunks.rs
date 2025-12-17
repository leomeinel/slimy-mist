/*
 * File: chunks.rs
 * Author: Leopold Johannes Meinel (leo@meinel.dev)
 * -----
 * Copyright (c) 2025 Leopold Johannes Meinel & contributors
 * SPDX ID: Apache-2.0
 * URL: https://www.apache.org/licenses/LICENSE-2.0
 * -----
 * This is heavily inspired by: https://github.com/StarArawn/bevy_ecs_tilemap
 */

use std::marker::PhantomData;

use bevy::{platform::collections::HashSet, prelude::*, reflect::Reflectable};
use bevy_ecs_tilemap::prelude::*;

use crate::{
    AppSystems, CanvasCamera,
    levels::{LEVEL_Z, Level, LevelAssets, RENDER_DISTANCE},
    logging::{error::ERR_LOADING_TILE_DATA, warn::WARN_INCOMPLETE_TILE_DATA},
    screens::Screen,
};

pub(super) fn plugin(app: &mut App) {
    // Setup timer
    app.insert_resource(ChunkTimer::default());
    app.add_systems(Update, tick_chunk_timer.in_set(AppSystems::TickTimers));
}

/// Size of a single chunk
pub(crate) const CHUNK_SIZE: UVec2 = UVec2 { x: 16, y: 16 };

/// Animation data deserialized from a ron file as a generic
#[derive(serde::Deserialize, Asset, TypePath, Default)]
pub(crate) struct TileData<T>
where
    T: Reflectable,
{
    pub(crate) tile_width: f32,
    pub(crate) tile_height: f32,
    #[serde(default)]
    full_dirt_tiles: Option<HashSet<UVec2>>,
    #[serde(default)]
    full_grass_tiles: Option<HashSet<UVec2>>,
    #[serde(default)]
    corner_outer_grass_to_dirt_tiles: Option<HashSet<UVec2>>,
    #[serde(default)]
    corner_outer_dirt_to_grass_tiles: Option<HashSet<UVec2>>,
    #[serde(default)]
    side_dirt_and_grass_tiles: Option<HashSet<UVec2>>,
    #[serde(default)]
    diag_stripe_grass_in_dirt_tiles: Option<HashSet<UVec2>>,
    #[serde(skip)]
    _phantom: PhantomData<T>,
}
impl<T> TileData<T>
where
    T: Reflectable,
{
    fn get_tiles(
        &self,
    ) -> Option<(
        HashSet<UVec2>,
        HashSet<UVec2>,
        HashSet<UVec2>,
        HashSet<UVec2>,
        HashSet<UVec2>,
        HashSet<UVec2>,
    )> {
        Some((
            self.full_dirt_tiles.as_ref().cloned()?,
            self.full_grass_tiles.as_ref().cloned()?,
            self.corner_outer_grass_to_dirt_tiles.as_ref().cloned()?,
            self.corner_outer_dirt_to_grass_tiles.as_ref().cloned()?,
            self.side_dirt_and_grass_tiles.as_ref().cloned()?,
            self.diag_stripe_grass_in_dirt_tiles.as_ref().cloned()?,
        ))
    }
}

/// Handle for [`TileData`] as a generic
#[derive(Resource)]
pub(crate) struct TileHandle<T>(pub(crate) Handle<TileData<T>>)
where
    T: Reflectable;

/// Chunk controller that stores spawned chunks
#[derive(Default, Debug, Resource)]
pub(crate) struct ChunkController<T> {
    pub(crate) positions: HashSet<IVec2>,
    _phantom: PhantomData<T>,
}

/// Chunk marker
#[derive(Component)]
pub(crate) struct Chunk;

/// Interval for generating chunks
const CHUNK_INTERVAL: f32 = 2.;

/// Timer that tracks chunk generation
#[derive(Resource, Debug, Clone, PartialEq, Reflect)]
#[reflect(Resource)]
pub(crate) struct ChunkTimer(Timer);
impl Default for ChunkTimer {
    fn default() -> Self {
        Self(Timer::from_seconds(CHUNK_INTERVAL, TimerMode::Repeating))
    }
}

/// Chunk size for [`TilemapRenderSettings`]
const RENDER_CHUNK_SIZE: UVec2 = UVec2 {
    x: CHUNK_SIZE.x * 2,
    y: CHUNK_SIZE.y * 2,
};

// TODO: Think about using generics to avoid code duplication
/// Spawn chunks around the [`CanvasCamera`]
pub(crate) fn spawn_chunks<T, A>(
    camera: Single<&Transform, (With<CanvasCamera>, Without<Chunk>)>,
    mut commands: Commands,
    mut controller: ResMut<ChunkController<T>>,
    data: Res<Assets<TileData<T>>>,
    handle: Res<TileHandle<T>>,
    assets: Res<A>,
    timer: Res<ChunkTimer>,
) where
    T: Level,
    A: LevelAssets,
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
    let pos = camera_pos / (chunk_size * tile_size);

    // Spawn chunk behind and in front of chunk position if it does not contain a chunk already
    for y in (pos.y - RENDER_DISTANCE)..(pos.y + RENDER_DISTANCE) {
        for x in (pos.x - RENDER_DISTANCE)..(pos.x + RENDER_DISTANCE) {
            if !controller.positions.contains(&IVec2::new(x, y)) {
                controller.positions.insert(IVec2::new(x, y));
                spawn_chunk::<A>(
                    &mut commands,
                    &assets,
                    IVec2::new(x, y),
                    tile_size.as_vec2(),
                    TileTextureIndex(8),
                );
            }
        }
    }
}

// TODO: Think about using generics to avoid code duplication
/// Despawn chunks
///
/// This removes the coordinates from [`ChunkController<T>`] and despawns the entity.
pub(crate) fn despawn_chunks<T>(
    camera: Single<&Transform, (With<CanvasCamera>, Without<Chunk>)>,
    query: Query<(Entity, &Transform), (With<Chunk>, Without<CanvasCamera>, Without<T>)>,
    mut commands: Commands,
    mut controller: ResMut<ChunkController<T>>,
    data: Res<Assets<TileData<T>>>,
    handle: Res<TileHandle<T>>,
    timer: Res<ChunkTimer>,
) where
    T: Level,
{
    // Return if timer has not finished
    if !timer.0.just_finished() {
        return;
    }

    // Get data from `TileData` with `TileHandle`
    let data = data.get(handle.0.id()).expect(ERR_LOADING_TILE_DATA);
    let tile_size = Vec2::new(data.tile_height, data.tile_width);

    // Despawn chunks outside of `DESPAWN_RANGE`
    for (entity, transform) in query.iter() {
        let pos = transform.translation.xy();
        let distance = camera.translation.xy().distance(pos);
        let despawn_range = RENDER_DISTANCE as f32 * CHUNK_SIZE.x as f32 * tile_size.x;

        if distance > despawn_range {
            let pos = &IVec2::new(
                (pos.x / (CHUNK_SIZE.x as f32 * tile_size.x)).floor() as i32,
                (pos.y / (CHUNK_SIZE.y as f32 * tile_size.y)).floor() as i32,
            );

            controller.positions.remove(pos);
            commands.entity(entity).despawn();
        }
    }
}

// TODO: Think about using generics to avoid code duplication
/// Clear [ChunkController]
pub(crate) fn clear_chunks<T>(mut controller: ResMut<ChunkController<T>>)
where
    T: Level,
{
    controller.positions.clear();
}

/// Spawn a single chunk
fn spawn_chunk<A>(
    commands: &mut Commands,
    assets: &Res<A>,
    chunk_pos: IVec2,
    tile_size: Vec2,
    texture_index: TileTextureIndex,
) where
    A: LevelAssets,
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
                    Chunk,
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

/// Tick chunk timer
fn tick_chunk_timer(mut timer: ResMut<ChunkTimer>, time: Res<Time>) {
    timer.0.tick(time.delta());
}
