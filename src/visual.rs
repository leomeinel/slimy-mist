/*
 * File: visual.rs
 * Author: Leopold Johannes Meinel (leo@meinel.dev)
 * -----
 * Copyright (c) 2026 Leopold Johannes Meinel & contributors
 * SPDX ID: Apache-2.0
 * URL: https://www.apache.org/licenses/LICENSE-2.0
 */

// FIXME: I'd like to have an easy way to use shaders for my sprites. This does currently not seem possible.
//        Also see:
//        - https://github.com/bevyengine/bevy/issues/1738
//        - https://github.com/bevyengine/bevy/issues/7131
//        - https://github.com/bevyengine/bevy/pull/10845

pub(crate) mod particles;

use std::marker::PhantomData;

use bevy::{prelude::*, reflect::Reflectable};

pub(super) fn plugin(app: &mut App) {
    app.add_plugins(particles::plugin);
}

/// Can apply to anything that is visible
pub(crate) trait Visible
where
    Self: Component + Default + Reflectable,
{
}

/// Cache for texture info
///
/// ## Traits
///
/// - `T` must implement [`Visible`].
#[derive(Resource, Default)]
pub(crate) struct TextureInfoCache<T>
where
    T: Visible,
{
    pub(crate) size: UVec2,
    pub(crate) _phantom: PhantomData<T>,
}
