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
    impl_animation_data, impl_animation_handle,
};

pub(super) fn plugin(app: &mut App) {
    // Insert Animation resource
    app.insert_resource(Animations::<Slime>::default());

    // Add plugin to load ron file
    app.add_plugins((RonAssetPlugin::<SlimeAnimationData>::new(&[
        "animation.ron",
    ]),));

    // Animation setup
    app.add_systems(Startup, setup_slime);
    app.add_systems(
        OnEnter(NpcAssetState::Next),
        animations::setup::<Slime, SlimeAnimationHandle, SlimeAssets>.after(setup_slime),
    );

    // Animation updates
    app.add_systems(
        Update,
        (
            animations::update::<Slime>,
            animations::update_sound::<Slime, SlimeAnimationHandle, SlimeAssets>,
        )
            .chain()
            .in_set(AppSystems::Update)
            .in_set(PausableSystems),
    );
}

/// Animation data that is serialized from a ron file
#[derive(serde::Deserialize, Asset, TypePath)]
struct SlimeAnimationData {
    atlas_columns: usize,
    atlas_rows: usize,
    idle_frames: usize,
    idle_interval_ms: u32,
    move_frames: usize,
    move_interval_ms: u32,
    step_sound_frames: Vec<usize>,
}
impl_animation_data![SlimeAnimationData];

/// Handle for [`SlimeAnimationData`]
#[derive(Resource)]
struct SlimeAnimationHandle(Handle<SlimeAnimationData>);
impl_animation_handle!(SlimeAnimationHandle, SlimeAnimationData);

/// Deserialize ron file for [`SlimeAnimationData`]
fn setup_slime(mut commands: Commands, assets: Res<AssetServer>) {
    let animation_handle =
        SlimeAnimationHandle(assets.load("data/characters/npc/slime.animation.ron"));
    commands.insert_resource(animation_handle);
}
