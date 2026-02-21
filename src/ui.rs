/*
 * File: ui.rs
 * Author: Leopold Johannes Meinel (leo@meinel.dev)
 * -----
 * Copyright (c) 2026 Leopold Johannes Meinel & contributors
 * SPDX ID: Apache-2.0
 * URL: https://www.apache.org/licenses/LICENSE-2.0
 * -----
 * Heavily inspired by: https://github.com/TheBevyFlock/bevy_new_2d
 */

//! Reusable UI widgets & theming.

pub(crate) mod directional_nav;
pub(crate) mod interaction;
pub(crate) mod palette;
pub(crate) mod scroll;
pub(crate) mod widgets;

#[allow(unused_imports)]
pub(crate) mod prelude {
    pub(crate) use super::{
        BODY_FONT_SIZE, HEADER_FONT_SIZE, NodeOffset, UiFontHandle,
        interaction::{InteractionOverride, InteractionPalette},
        palette::*,
        widgets,
    };
}

use bevy::prelude::*;

pub(super) fn plugin(app: &mut App) {
    // Add child plugins
    app.add_plugins((directional_nav::plugin, interaction::plugin, scroll::plugin));
}

/// Font size for any header.
pub(crate) const HEADER_FONT_SIZE: f32 = 36.;
/// Font size for any body.
pub(crate) const BODY_FONT_SIZE: f32 = 18.;

/// Wrapper for [`Handle<Font>`] for the ui.
#[derive(Resource, Default)]
pub(crate) struct UiFontHandle(pub(crate) Handle<Font>);

/// Offset that stores the offset for a [`Node`].
///
/// Can apply to [`Node::left`] and [`Node::bottom`] according to [`Self::0`].
#[derive(Component, Default)]
pub(crate) struct NodeOffset(pub(crate) IVec2);
