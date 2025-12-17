/*
 * File: spawn.rs
 * Author: Leopold Johannes Meinel (leo@meinel.dev)
 * -----
 * Copyright (c) 2025 Leopold Johannes Meinel & contributors
 * SPDX ID: Apache-2.0
 * URL: https://www.apache.org/licenses/LICENSE-2.0
 */

use std::marker::PhantomData;

use bevy::{platform::collections::HashSet, prelude::*};
use bevy_prng::WyRand;
use rand::{Rng as _, seq::IndexedRandom as _};

use crate::{
    AppSystems, CanvasCamera,
    characters::{
        Character, CharacterRng, CollisionData, CollisionHandle, Shadow, VisualMap,
        animations::{ANIMATION_DELAY_RANGE, AnimationRng, Animations},
    },
    levels::{
        DEFAULT_Z, Level, RENDER_DISTANCE,
        chunks::{CHUNK_SIZE, ChunkController, TileData, TileHandle},
    },
    logging::error::{ERR_LOADING_COLLISION_DATA, ERR_LOADING_TILE_DATA},
};

pub(super) fn plugin(app: &mut App) {
    // Setup timer
    app.insert_resource(SpawnTimer::default());
    app.add_systems(Update, tick_spawn_timer.in_set(AppSystems::TickTimers));
}

/// Spawn controller that stores positions of spawned entities
#[derive(Default, Debug, Resource)]
pub(crate) struct SpawnController<T> {
    pub(crate) positions: HashSet<IVec2>,
    _phantom: PhantomData<T>,
}

/// Interval for generating chunks
const SPAWN_INTERVAL: f32 = 2.;

/// Timer that tracks chunk generation
#[derive(Resource, Debug, Clone, PartialEq, Reflect)]
#[reflect(Resource)]
pub(crate) struct SpawnTimer(Timer);
impl Default for SpawnTimer {
    fn default() -> Self {
        Self(Timer::from_seconds(SPAWN_INTERVAL, TimerMode::Repeating))
    }
}

// TODO: Think about using generics to avoid code duplication
/// Spawn chunks around the [`CanvasCamera`]
pub(crate) fn spawn_characters<T, A>(
    mut animation_rng: Single<&mut WyRand, (With<AnimationRng>, Without<CharacterRng>)>,
    mut rng: Single<&mut WyRand, (With<CharacterRng>, Without<AnimationRng>)>,
    level: Single<Entity, With<A>>,
    mut commands: Commands,
    mut controller: ResMut<SpawnController<T>>,
    mut visual_map: ResMut<VisualMap>,
    animations: Res<Animations<T>>,
    chunk_controller: Res<ChunkController<A>>,
    collision_data: Res<Assets<CollisionData<T>>>,
    collision_handle: Res<CollisionHandle<T>>,
    shadow: Res<Shadow<T>>,
    tile_data: Res<Assets<TileData<A>>>,
    tile_handle: Res<TileHandle<A>>,
    timer: Res<SpawnTimer>,
) where
    T: Character,
    A: Level,
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
            &mut rng,
            &mut commands,
            &mut visual_map,
            level.entity(),
            &animations,
            &data,
            &shadow,
            target_pos,
            &tile_size,
            animation_rng.random_range(ANIMATION_DELAY_RANGE),
        );
    }
}

// TODO: Think about using generics to avoid code duplication
//       It is quite hard to implement a system where we just use contained values in chunk controller
//       to determine despawning since we also need to despawn the correct entity and check moving position.
/// Despawn characters
///
/// This removes the coordinates from [`SpawnController`] and despawns the entity.
pub(crate) fn despawn_characters<T, A>(
    camera: Single<&Transform, (With<CanvasCamera>, Without<T>)>,
    query: Query<(Entity, &Transform), (With<T>, Without<CanvasCamera>)>,
    mut commands: Commands,
    mut controller: ResMut<SpawnController<T>>,
    data: Res<Assets<TileData<A>>>,
    handle: Res<TileHandle<A>>,
    timer: Res<SpawnTimer>,
) where
    T: Character,
    A: Level,
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
/// Clear [SpawnController]
pub(crate) fn clear_spawn_points<T>(mut controller: ResMut<SpawnController<T>>)
where
    T: Character,
{
    controller.positions.clear();
}

/// Number of characters to spawn per chunk
const CHARACTERS_PER_CHUNK: usize = 4;

/// Spawn characters in a chunk
fn spawn_character<T>(
    rng: &mut WyRand,
    commands: &mut Commands,
    visual_map: &mut ResMut<VisualMap>,
    container: Entity,
    animations: &Res<Animations<T>>,
    data: &(Option<String>, Option<f32>, Option<f32>),
    shadow: &Res<Shadow<T>>,
    chunk_pos: &Vec2,
    tile_size: &Vec2,
    animation_delay: f32,
) where
    T: Character,
{
    // Choose a number of target chunk tile origins to determine spawn positions
    let target_origins: Vec<(u32, u32)> = (0..CHUNK_SIZE.x)
        .flat_map(|x| (0..CHUNK_SIZE.y).map(move |y| (x, y)))
        .collect();
    let target_origins: Vec<&(u32, u32)> = target_origins
        .choose_multiple(rng, CHARACTERS_PER_CHUNK)
        .collect();

    for (x, y) in target_origins {
        // Multiply by tile size because chunk_pos only stores pixel origins of tiles
        let spawn_pos = Vec3::new(
            chunk_pos.x + *x as f32 * tile_size.x,
            chunk_pos.y + *y as f32 * tile_size.y,
            DEFAULT_Z,
        );

        // Spawn character in chosen tile
        let entity = T::spawn(
            commands,
            visual_map,
            data,
            spawn_pos,
            animations,
            shadow,
            animation_delay,
        );
        commands.entity(container).add_child(entity);
    }
}

/// Tick spawn timer
fn tick_spawn_timer(mut timer: ResMut<SpawnTimer>, time: Res<Time>) {
    timer.0.tick(time.delta());
}
