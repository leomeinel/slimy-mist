/*
 * File: theme.rs
 * Author: Leopold Johannes Meinel (leo@meinel.dev)
 * -----
 * Copyright (c) 2025 Leopold Johannes Meinel & contributors
 * SPDX ID: Apache-2.0
 * URL: https://www.apache.org/licenses/LICENSE-2.0
 * -----
 * Heavily inspired by: https://github.com/TheBevyFlock/bevy_new_2d
 */

//! Reusable UI widgets & theming.

// Unused utilities may trigger this lints undesirably.
#![allow(dead_code)]

pub(crate) mod interaction;
pub(crate) mod palette;
pub(crate) mod widgets;

#[allow(unused_imports)]
pub(crate) mod prelude {
    pub(crate) use super::{interaction::InteractionPalette, palette as ui_palette, widgets};
}

use bevy::prelude::*;

pub(super) fn plugin(app: &mut App) {
    // Add child plugins
    app.add_plugins(interaction::plugin);
}
