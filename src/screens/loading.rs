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
        CollisionData, CollisionHandle,
        animations::{AnimationData, AnimationHandle},
        npc::{Slime, SlimeAssets},
        player::{Player, PlayerAssets},
    },
    levels::overworld::{OverworldAssets, OverworldProcGen},
    menus::credits::CreditsAssets,
    procgen::{TileData, TileHandle},
    screens::{Screen, splash::SplashAssets},
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

    // Spawn loading screen and load custom
    app.add_systems(
        OnEnter(Screen::Loading),
        (
            spawn_loading_screen,
            // After initial `LoadingState<Screen::Loading>`, run other requirements before switching to `Screen::Splash`
            (setup_overworld, setup_player, setup_slime).after(LoadingStateSet(Screen::Loading)),
        ),
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

/// Deserialize ron file for [`TileData`]
fn setup_overworld(mut commands: Commands, assets: Res<AssetServer>) {
    let handle = TileHandle::<OverworldProcGen>(assets.load("data/levels/overworld.tiles.ron"));
    commands.insert_resource(handle);
}

/// Deserialize ron file for [`CollisionData`]
fn setup_player(mut commands: Commands, assets: Res<AssetServer>) {
    let handle =
        CollisionHandle::<Player>(assets.load("data/characters/player/male.collision.ron"));
    commands.insert_resource(handle);

    let handle =
        AnimationHandle::<Player>(assets.load("data/characters/player/male.animation.ron"));
    commands.insert_resource(handle);
}

/// Deserialize ron file for [`CollisionData`]
fn setup_slime(mut commands: Commands, assets: Res<AssetServer>) {
    // Collisions
    let handle = CollisionHandle::<Slime>(assets.load("data/characters/npc/slime.collision.ron"));
    commands.insert_resource(handle);

    // Animations
    let handle = AnimationHandle::<Slime>(assets.load("data/characters/npc/slime.animation.ron"));
    commands.insert_resource(handle);
}
