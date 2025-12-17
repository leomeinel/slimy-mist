/*
 * File: procgen.rs
 * Author: Leopold Johannes Meinel (leo@meinel.dev)
 * -----
 * Copyright (c) 2025 Leopold Johannes Meinel & contributors
 * SPDX ID: Apache-2.0
 * URL: https://www.apache.org/licenses/LICENSE-2.0
 */

pub(crate) mod level;
pub(crate) mod spawn;

use bevy::prelude::*;
use bevy_prng::WyRand;
use bevy_rand::{global::GlobalRng, traits::ForkableSeed as _};

pub(super) fn plugin(app: &mut App) {
    // Add rng for chunks
    app.add_systems(Startup, setup_rng);

    // Add child plugins
    app.add_plugins((level::plugin, spawn::plugin));
}

/// Rng for animations
#[derive(Component)]
pub(crate) struct ChunkRng;

/// Spawn [`ChunkRng`] by forking [`GlobalRng`]
fn setup_rng(mut global: Single<&mut WyRand, With<GlobalRng>>, mut commands: Commands) {
    commands.spawn((ChunkRng, global.fork_seed()));
}
