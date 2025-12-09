/*
 * File: player.rs
 * Author: Leopold Johannes Meinel (leo@meinel.dev)
 * -----
 * Copyright (c) 2025 Leopold Johannes Meinel & contributors
 * SPDX ID: Apache-2.0
 * URL: https://www.apache.org/licenses/LICENSE-2.0
 */

//! Player sprite animation.
//! This is based on multiple examples and may be very different for your game.
//! - [Sprite flipping](https://github.com/bevyengine/bevy/blob/latest/examples/2d/sprite_flipping.rs)
//! - [Sprite animation](https://github.com/bevyengine/bevy/blob/latest/examples/2d/sprite_animation.rs)
//! - [Timers](https://github.com/bevyengine/bevy/blob/latest/examples/time/timers.rs)

use bevy::prelude::*;
use bevy_common_assets::ron::RonAssetPlugin;

use crate::{
    AppSystems, PausableSystems,
    characters::{
        animations::{self, AnimationData, AnimationHandle, Animations},
        player::{Player, PlayerAssetState, PlayerAssets},
    },
    impl_animation_data, impl_animation_handle,
};

pub(super) fn plugin(app: &mut App) {
    // Insert Animation resource
    app.insert_resource(Animations::<Player>::default());

    // Add plugin to load ron file
    app.add_plugins((RonAssetPlugin::<PlayerAnimationData>::new(&[
        "animation.ron",
    ]),));

    // Animation setup
    app.add_systems(Startup, setup_player);
    app.add_systems(
        OnEnter(PlayerAssetState::Next),
        animations::setup::<Player, PlayerAnimationHandle, PlayerAssets>.after(setup_player),
    );

    // Animation updates
    app.add_systems(
        Update,
        (
            animations::update::<Player>,
            animations::update_sound::<Player, PlayerAnimationHandle, PlayerAssets>,
        )
            .chain()
            .in_set(AppSystems::Update)
            .in_set(PausableSystems),
    );
}

/// Animation data that is serialized from a ron file
#[derive(serde::Deserialize, Asset, TypePath)]
struct PlayerAnimationData {
    atlas_columns: usize,
    atlas_rows: usize,
    idle_frames: usize,
    idle_interval_ms: u32,
    move_frames: usize,
    move_interval_ms: u32,
    step_sound_frames: Vec<usize>,
}
impl_animation_data![PlayerAnimationData];

/// Handle for [`PlayerAnimationData`]
#[derive(Resource)]
struct PlayerAnimationHandle(Handle<PlayerAnimationData>);
impl_animation_handle!(PlayerAnimationHandle, PlayerAnimationData);

/// Deserialize ron file for [`PlayerAnimationData`]
fn setup_player(mut commands: Commands, assets: Res<AssetServer>) {
    let animation_handle =
        PlayerAnimationHandle(assets.load("data/characters/player/male.animation.ron"));
    commands.insert_resource(animation_handle);
}
