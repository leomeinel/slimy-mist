/*
 * File: loading.rs
 * Author: Leopold Johannes Meinel (leo@meinel.dev)
 * -----
 * Copyright (c) 2026 Leopold Johannes Meinel & contributors
 * SPDX ID: Apache-2.0
 * URL: https://www.apache.org/licenses/LICENSE-2.0
 * -----
 * Heavily inspired by:
 * - https://github.com/TheBevyFlock/bevy_new_2d/tree/main
 * - https://github.com/NiklasEi/bevy_asset_loader
 */

//! A loading screen during which game assets are loaded if necessary.
//! This reduces stuttering, especially for audio on Wasm.

use bevy::prelude::*;
use bevy_asset_loader::prelude::*;
use bevy_common_assets::ron::RonAssetPlugin;
use iyes_progress::ProgressPlugin;

#[cfg(any(target_os = "android", target_os = "ios"))]
use crate::mobile::JoystickAssets;
use crate::{
    animations::{AnimationData, AnimationDataCache, AnimationHandle},
    characters::{
        Character, CollisionData, CollisionDataCache, CollisionHandle,
        npc::{Slime, SlimeAssets},
        player::{Player, PlayerAssets},
    },
    levels::overworld::{OverworldAssets, OverworldProcGen},
    logging::error::{
        ERR_LOADING_ANIMATION_DATA, ERR_LOADING_COLLISION_DATA, ERR_LOADING_LAYER_DATA,
        ERR_LOADING_TILE_DATA,
    },
    menus::credits::CreditsAssets,
    procgen::{
        CHUNK_SIZE, PROCGEN_DISTANCE, ProcGenerated, TileData, TileDataCache, TileDataRelatedCache,
        TileHandle,
    },
    screens::{Screen, splash::SplashAssets},
    ui::{interaction::InteractionAssets, prelude::*},
    visual::{
        Visible,
        layers::{LayerData, LayerDataRelatedCache, LayerHandle},
        particles::{ParticleHandle, ParticleMeleeAttack, ParticleWalkingDust},
    },
};

pub(super) fn plugin(app: &mut App) {
    // Add library plugins
    app.add_plugins((
        // Progress tracking
        // NOTE: This advances to `Screen::LoadingCache` to cache data
        ProgressPlugin::<Screen>::new()
            .with_state_transition(Screen::Loading, Screen::LoadingCache),
        // Levels
        RonAssetPlugin::<TileData<OverworldProcGen>>::new(&["tiles.ron"]),
        // Characters
        RonAssetPlugin::<AnimationData<Player>>::new(&["animation.ron"]),
        RonAssetPlugin::<CollisionData<Player>>::new(&["collision.ron"]),
        RonAssetPlugin::<LayerData<Player>>::new(&["layers.ron"]),
        RonAssetPlugin::<AnimationData<Slime>>::new(&["animation.ron"]),
        RonAssetPlugin::<CollisionData<Slime>>::new(&["collision.ron"]),
        RonAssetPlugin::<LayerData<Slime>>::new(&["layers.ron"]),
    ));

    // Add loading states via bevy_asset_loader
    let loading_state = LoadingState::new(Screen::Loading)
        .load_collection::<InteractionAssets>()
        .load_collection::<SplashAssets>()
        .load_collection::<CreditsAssets>()
        .with_dynamic_assets_file::<StandardDynamicAssetCollection>(
            "data/levels/overworld.assets.ron",
        )
        .load_collection::<OverworldAssets>()
        .with_dynamic_assets_file::<StandardDynamicAssetCollection>(
            "data/characters/player/male.assets.ron",
        )
        .load_collection::<PlayerAssets>()
        .with_dynamic_assets_file::<StandardDynamicAssetCollection>(
            "data/characters/npc/slime.assets.ron",
        )
        .load_collection::<SlimeAssets>();
    #[cfg(any(target_os = "android", target_os = "ios"))]
    let loading_state = loading_state.load_collection::<JoystickAssets>();
    app.add_loading_state(loading_state);

    // Spawn loading screen and load custom resources
    app.add_systems(
        OnEnter(Screen::Loading),
        (
            spawn_loading_screen,
            // After initial `LoadingState<Screen::Loading>` insert resources with handles for data
            insert_handle_resources.after(LoadingStateSet(Screen::Loading)),
        ),
    );
    app.add_systems(
        OnEnter(Screen::LoadingCache),
        (
            (
                cache_animation_data::<Player>,
                cache_animation_data::<Slime>,
                cache_collision_data::<Player>,
                cache_collision_data::<Slime>,
                cache_layer_data_related::<Player>,
                cache_layer_data_related::<Slime>,
                cache_tile_data_and_related::<OverworldProcGen>,
            ),
            enter_splash_screen,
        )
            .chain(),
    );
}

/// Display loading screen
fn spawn_loading_screen(mut commands: Commands) {
    commands.spawn((
        widgets::ui_root("Loading Screen"),
        DespawnOnExit(Screen::Loading),
        children![widgets::label("Loading...")],
    ));
}

/// Insert handle [`Resource`]s for deserialized data.
///
/// These serve as handles for the actual data.
fn insert_handle_resources(mut commands: Commands, assets: Res<AssetServer>) {
    // `AnimationData`
    commands.insert_resource(AnimationHandle::<Player>(
        assets.load("data/characters/human/male.animation.ron"),
    ));
    commands.insert_resource(AnimationHandle::<Slime>(
        assets.load("data/characters/npc/slime.animation.ron"),
    ));

    // `CollisionData`
    commands.insert_resource(CollisionHandle::<Player>(
        assets.load("data/characters/human/male.collision.ron"),
    ));
    commands.insert_resource(CollisionHandle::<Slime>(
        assets.load("data/characters/npc/slime.collision.ron"),
    ));

    commands.insert_resource(LayerHandle::<Player>(
        assets.load("data/characters/player/male.layers.ron"),
    ));
    commands.insert_resource(LayerHandle::<Slime>(
        assets.load("data/characters/npc/slime.layers.ron"),
    ));

    // `TileData`
    commands.insert_resource(TileHandle::<OverworldProcGen>(
        assets.load("data/levels/overworld.tiles.ron"),
    ));

    // `ParticleHandle` not needing a custom data struct
    commands.insert_resource(ParticleHandle::<ParticleWalkingDust> {
        handle: assets.load("data/particles/walking-dust.particle.ron"),
        ..default()
    });
    commands.insert_resource(ParticleHandle::<ParticleMeleeAttack> {
        handle: assets.load("data/particles/melee-attack.particle.ron"),
        ..default()
    });
}

/// Cache data from [`AnimationData`] in [`AnimationDataCache`]
///
/// ## Traits
///
/// - `T` must implement [`Character`].
fn cache_animation_data<T>(
    mut commands: Commands,
    mut data: ResMut<Assets<AnimationData<T>>>,
    handle: Res<AnimationHandle<T>>,
) where
    T: Character,
{
    let data = data
        .remove(handle.0.id())
        .expect(ERR_LOADING_ANIMATION_DATA);
    commands.insert_resource(AnimationDataCache::<T> {
        atlas_columns: data.atlas_columns,
        atlas_rows: data.atlas_rows,
        idle_frames: data.idle_frames,
        idle_interval_ms: data.idle_interval_ms,
        walk_frames: data.walk_frames,
        walk_interval_ms: data.walk_interval_ms,
        walk_sound_frames: data.walk_sound_frames,
        _run_frames: data.run_frames,
        _run_interval_ms: data.run_interval_ms,
        _run_sound_frames: data.run_sound_frames,
        jump_frames: data.jump_frames,
        jump_sound_frames: data.jump_sound_frames,
        fall_frames: data.fall_frames,
        fall_sound_frames: data.fall_sound_frames,
        ..default()
    });

    // Remove handle after caching since it is no longer needed
    commands.remove_resource::<AnimationHandle<T>>();
}

/// Cache data from [`CollisionData`] in [`CollisionDataCache`]
///
/// ## Traits
///
/// - `T` must implement [`Character`].
fn cache_collision_data<T>(
    mut commands: Commands,
    mut data: ResMut<Assets<CollisionData<T>>>,
    handle: Res<CollisionHandle<T>>,
) where
    T: Character,
{
    let data = data
        .remove(handle.0.id())
        .expect(ERR_LOADING_COLLISION_DATA);
    commands.insert_resource(CollisionDataCache::<T> {
        shape: data.shape,
        width: data.width,
        height: data.height,
        ..default()
    });

    // Remove handle after caching since it is no longer needed
    commands.remove_resource::<CollisionHandle<T>>();
}

/// Cache data from [`LayerData`] in [`LayerDataCache`]
///
/// ## Traits
///
/// - `T` must implement [`Visible`].
fn cache_layer_data_related<T>(
    mut commands: Commands,
    mut data: ResMut<Assets<LayerData<T>>>,
    assets: Res<AssetServer>,
    handle: Res<LayerHandle<T>>,
) where
    T: Visible,
{
    let data = data.remove(handle.0.id()).expect(ERR_LOADING_LAYER_DATA);
    let images = data.layers.iter().map(|l| assets.load(l)).collect();
    commands.insert_resource(LayerDataRelatedCache::<T> {
        images,
        ..default()
    });

    // Remove handle after caching since it is no longer needed
    commands.remove_resource::<LayerHandle<T>>();
}

/// Cache data from [`TileData`] in [`TileDataCache`] and related data in [`TileDataRelatedCache`]
///
/// ## Traits
///
/// - `T` must implement [`ProcGenerated`].
fn cache_tile_data_and_related<T>(
    mut commands: Commands,
    mut data: ResMut<Assets<TileData<T>>>,
    handle: Res<TileHandle<T>>,
) where
    T: ProcGenerated,
{
    let data = data.remove(handle.0.id()).expect(ERR_LOADING_TILE_DATA);
    commands.insert_resource(TileDataCache::<T> {
        tile_size: data.tile_size,
        _full_dirt_tiles: data.full_dirt_tiles,
        _full_grass_tiles: data.full_grass_tiles,
        _corner_outer_grass_to_dirt_tiles: data.corner_outer_grass_to_dirt_tiles,
        _corner_outer_dirt_to_grass_tiles: data.corner_outer_dirt_to_grass_tiles,
        _side_dirt_and_grass_tiles: data.side_dirt_and_grass_tiles,
        _diag_stripe_grass_in_dirt_tiles: data.diag_stripe_grass_in_dirt_tiles,
        ..default()
    });
    let chunk_size_px = CHUNK_SIZE.as_vec2() * data.tile_size;
    let world_height = PROCGEN_DISTANCE as f32 * 2. + 1. * chunk_size_px.y;
    commands.insert_resource(TileDataRelatedCache::<T> {
        chunk_size_px,
        world_height,
        ..default()
    });

    // Remove handle after caching since it is no longer needed
    commands.remove_resource::<TileHandle<T>>();
}

/// Enter splash screen
fn enter_splash_screen(mut next_state: ResMut<NextState<Screen>>) {
    next_state.set(Screen::Splash);
}
