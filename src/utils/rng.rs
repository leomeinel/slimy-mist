/*
 * File: rng.rs
 * Author: Leopold Johannes Meinel (leo@meinel.dev)
 * -----
 * Copyright (c) 2026 Leopold Johannes Meinel & contributors
 * SPDX ID: Apache-2.0
 * URL: https://www.apache.org/licenses/LICENSE-2.0
 */

use bevy::prelude::*;
use bevy_prng::WyRand;
use bevy_rand::{global::GlobalRng, traits::ForkableSeed as _};

/// Applies to any rng that is forked from [`GlobalRng`]
pub(crate) trait ForkedRng
where
    Self: Component + Default,
{
}

/// Spawn [`ForkedRng`] by forking [`GlobalRng`]
pub(crate) fn setup_rng<T>(mut global: Single<&mut WyRand, With<GlobalRng>>, mut commands: Commands)
where
    T: ForkedRng,
{
    commands.spawn((T::default(), global.fork_seed()));
}
