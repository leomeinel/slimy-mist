/*
 * File: characters.rs
 * Author: Leopold Johannes Meinel (leo@meinel.dev)
 * -----
 * Copyright (c) 2025 Leopold Johannes Meinel & contributors
 * SPDX ID: Apache-2.0
 * URL: https://www.apache.org/licenses/LICENSE-2.0
 */

//! Characters

pub(crate) mod npc;

use bevy::prelude::*;

/// Plugin
pub(super) fn plugin(app: &mut App) {
    // Add plugins
    app.add_plugins(npc::plugin);
}
