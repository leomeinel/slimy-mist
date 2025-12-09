/*
 * File: npc.rs
 * Author: Leopold Johannes Meinel (leo@meinel.dev)
 * -----
 * Copyright (c) 2025 Leopold Johannes Meinel & contributors
 * SPDX ID: Apache-2.0
 * URL: https://www.apache.org/licenses/LICENSE-2.0
 * -----
 * Heavily inspired by: https://github.com/NiklasEi/bevy_common_assets
 */

//! Animation for npc characters

use bevy::prelude::*;
use bevy_common_assets::ron::RonAssetPlugin;

use crate::{
    AppSystems, PausableSystems,
    characters::{
        animations::{self, AnimationData, AnimationHandle, Animations},
        npc::{NpcAssetState, Slime, SlimeAssets},
    },
};

pub(super) fn plugin(app: &mut App) {
    // Insert Animation resource
    app.insert_resource(Animations::<Slime>::default());

    // Add plugin to load ron file
    app.add_plugins((RonAssetPlugin::<AnimationData<Slime>>::new(&[
        "animation.ron",
    ]),));

    // Animation setup
    app.add_systems(Startup, setup_slime);
    app.add_systems(
        OnEnter(NpcAssetState::Next),
        animations::setup::<Slime, SlimeAssets>.after(setup_slime),
    );

    // Animation updates
    app.add_systems(
        Update,
        (
            animations::update::<Slime>,
            animations::update_sound::<Slime, SlimeAssets>,
        )
            .chain()
            .in_set(AppSystems::Update)
            .in_set(PausableSystems),
    );
}

/// Deserialize ron file for [`SlimeAnimationData`]
fn setup_slime(mut commands: Commands, assets: Res<AssetServer>) {
    let animation_handle =
        AnimationHandle::<Slime>(assets.load("data/characters/npc/slime.animation.ron"));
    commands.insert_resource(animation_handle);
}
