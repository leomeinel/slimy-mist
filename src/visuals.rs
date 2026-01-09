/*
 * File: visuals.rs
 * Author: Leopold Johannes Meinel (leo@meinel.dev)
 * -----
 * Copyright (c) 2026 Leopold Johannes Meinel & contributors
 * SPDX ID: Apache-2.0
 * URL: https://www.apache.org/licenses/LICENSE-2.0
 */

use std::marker::PhantomData;

use bevy::{prelude::*, reflect::Reflectable};

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
