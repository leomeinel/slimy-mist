/*
 * File: levels.rs
 * Author: Leopold Johannes Meinel (leo@meinel.dev)
 * -----
 * Copyright (c) 2025 Leopold Johannes Meinel & contributors
 * SPDX ID: Apache-2.0
 * URL: https://www.apache.org/licenses/LICENSE-2.0
 */

//! Game worlds

pub(crate) mod arena;

use bevy::prelude::*;

pub(super) fn plugin(app: &mut App) {
    // Add child plugins
    app.add_plugins(arena::plugin);
}
