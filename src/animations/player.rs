/*
 * File: player.rs
 * Author: Leopold Johannes Meinel (leo@meinel.dev)
 * -----
 * Copyright (c) 2025 Leopold Johannes Meinel & contributors
 * SPDX ID: Apache-2.0
 * URL: https://www.apache.org/licenses/LICENSE-2.0
 */

//! Player sprite animation.

use bevy::prelude::*;
use bevy_common_assets::ron::RonAssetPlugin;

use crate::{
    AppSystems, PausableSystems,
    animations::{self, AnimationData, AnimationHandle, Animations, tick_animation_timer},
    characters::player::{Player, PlayerAssetState, PlayerAssets},
};

pub(super) fn plugin(app: &mut App) {
    // Insert Animation resource
    app.insert_resource(Animations::<Player>::default());

    // Add plugin to load ron file
    app.add_plugins((RonAssetPlugin::<AnimationData<Player>>::new(&[
        "animation.ron",
    ]),));

    // Animation setup
    app.add_systems(Startup, setup_player);
    app.add_systems(
        OnEnter(PlayerAssetState::Next),
        animations::setup::<Player, PlayerAssets>.after(setup_player),
    );

    // Animation updates
    app.add_systems(
        Update,
        (
            animations::update::<Player>.after(tick_animation_timer),
            animations::update_sound::<Player, PlayerAssets>,
        )
            .chain()
            .in_set(AppSystems::Update)
            .in_set(PausableSystems),
    );
}

/// Deserialize ron file for [`AnimationData`]
fn setup_player(mut commands: Commands, assets: Res<AssetServer>) {
    let handle =
        AnimationHandle::<Player>(assets.load("data/characters/player/male.animation.ron"));
    commands.insert_resource(handle);
}
