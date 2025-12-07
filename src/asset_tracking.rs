/*
 * File: asset_tracking.rs
 * Author: Leopold Johannes Meinel (leo@meinel.dev)
 * -----
 * Copyright (c) 2025 Leopold Johannes Meinel & contributors
 * SPDX ID: Apache-2.0
 * URL: https://www.apache.org/licenses/LICENSE-2.0
 * -----
 * Heavily inspired by: https://github.com/NiklasEi/bevy_asset_loader
 */

//! A high-level way to load collections of asset handles as resources.

use bevy::prelude::*;

pub(super) fn plugin(app: &mut App) {
    app.init_state::<AssetStates>();
}

#[derive(Clone, Eq, PartialEq, Debug, Hash, Default, States)]
pub(crate) enum AssetStates {
    #[default]
    AssetLoading,
    Next,
}
