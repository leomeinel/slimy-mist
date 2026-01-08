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

use crate::{
    characters::{
        Character, CollisionData, CollisionDataCache, CollisionHandle,
        animations::{AnimationData, AnimationDataCache, AnimationHandle},
        npc::{Slime, SlimeAssets},
        player::{Player, PlayerAssets},
    },
    levels::overworld::{OverworldAssets, OverworldProcGen},
    logging::error::{
        ERR_LOADING_ANIMATION_DATA, ERR_LOADING_COLLISION_DATA, ERR_LOADING_TILE_DATA,
    },
    menus::credits::CreditsAssets,
    procgen::{
        CHUNK_SIZE, PROCGEN_DISTANCE, ProcGenerated, TileData, TileDataCache, TileDataRelatedCache,
        TileHandle,
    },
    screens::{GameplayInsertResSystems, Screen, splash::SplashAssets},
    theme::{interaction::InteractionAssets, prelude::*},
};

pub(super) fn plugin(app: &mut App) {
    // Add library plugins
    app.add_plugins((
        // Progress tracking
        ProgressPlugin::<Screen>::new().with_state_transition(Screen::Loading, Screen::Splash),
        // Levels
        RonAssetPlugin::<TileData<OverworldProcGen>>::new(&["tiles.ron"]),
        // Characters
        RonAssetPlugin::<AnimationData<Player>>::new(&["animation.ron"]),
        RonAssetPlugin::<CollisionData<Player>>::new(&["collision.ron"]),
        RonAssetPlugin::<AnimationData<Slime>>::new(&["animation.ron"]),
        RonAssetPlugin::<CollisionData<Slime>>::new(&["collision.ron"]),
    ));

    // Add loading states via bevy_asset_loader
    app.add_loading_state(
        LoadingState::new(Screen::Loading)
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
            .load_collection::<SlimeAssets>(),
    );

    // Spawn loading screen and load custom resources
    app.add_systems(
        OnEnter(Screen::Loading),
        (
            spawn_loading_screen,
            // After initial `LoadingState<Screen::Loading>`, run other requirements before switching to `Screen::Splash`
            (setup_overworld, setup_player, setup_slime)
                .chain()
                .after(LoadingStateSet(Screen::Loading)),
        ),
    );
    app.add_systems(
        OnEnter(Screen::Gameplay),
        (
            cache_animation_data::<Player>,
            cache_animation_data::<Slime>,
            cache_collision_data::<Player>,
            cache_collision_data::<Slime>,
            cache_tile_data_and_related::<OverworldProcGen>,
        )
            .in_set(GameplayInsertResSystems),
    );
}

/// Display loading screen
fn spawn_loading_screen(mut commands: Commands) {
    commands.spawn((
        widgets::common::ui_root("Loading Screen"),
        DespawnOnExit(Screen::Loading),
        children![widgets::common::label("Loading...")],
    ));
}

/// Deserialize and insert [`TileData`] from ron file as [`Resource`]
fn setup_overworld(mut commands: Commands, assets: Res<AssetServer>) {
    let handle = TileHandle::<OverworldProcGen>(assets.load("data/levels/overworld.tiles.ron"));
    commands.insert_resource(handle);
}

/// Deserialize and insert multiple ron files as [`Resource`]s for [`Player`]
fn setup_player(mut commands: Commands, assets: Res<AssetServer>) {
    let handle =
        CollisionHandle::<Player>(assets.load("data/characters/player/male.collision.ron"));
    commands.insert_resource(handle);

    let handle =
        AnimationHandle::<Player>(assets.load("data/characters/player/male.animation.ron"));
    commands.insert_resource(handle);
}

/// Deserialize and insert multiple ron files as [`Resource`]s for [`Slime`]
fn setup_slime(mut commands: Commands, assets: Res<AssetServer>) {
    // Collisions
    let handle = CollisionHandle::<Slime>(assets.load("data/characters/npc/slime.collision.ron"));
    commands.insert_resource(handle);

    // Animations
    let handle = AnimationHandle::<Slime>(assets.load("data/characters/npc/slime.animation.ron"));
    commands.insert_resource(handle);
}

/// Cache data from [`TileData`] in [`TileDataCache`] and related data in [`TileDataRelatedCache`]
fn cache_tile_data_and_related<T>(
    mut commands: Commands,
    mut data: ResMut<Assets<TileData<T>>>,
    handle: Res<TileHandle<T>>,
) where
    T: ProcGenerated,
{
    let data = data.remove(handle.0.id()).expect(ERR_LOADING_TILE_DATA);
    // FIXME: Add missing fields from `TileData`
    let tile_size = data.tile_size;
    commands.insert_resource(TileDataCache::<T> {
        tile_size,
        ..default()
    });
    let chunk_size_px = CHUNK_SIZE.as_vec2() * tile_size;
    let world_height = PROCGEN_DISTANCE as f32 * 2. + 1. * chunk_size_px.y;
    commands.insert_resource(TileDataRelatedCache::<T> {
        chunk_size_px,
        world_height,
        ..default()
    });
}

/// Cache data from [`CollisionData`] in [`CollisionDataCache`]
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
}

/// Cache data from [`AnimationData`] in [`AnimationDataCache`]
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
        idle_row: data.idle_row,
        idle_frames: data.idle_frames,
        idle_interval_ms: data.idle_interval_ms,
        walk_row: data.walk_row,
        walk_frames: data.walk_frames,
        walk_interval_ms: data.walk_interval_ms,
        walk_sound_frames: data.walk_sound_frames,
        jump_row: data.jump_row,
        jump_frames: data.jump_frames,
        jump_sound_frames: data.jump_sound_frames,
        fall_row: data.fall_row,
        fall_frames: data.fall_frames,
        fall_sound_frames: data.fall_sound_frames,
        ..default()
    });
}
