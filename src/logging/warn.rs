/*
 * File: warn.rs
 * Author: Leopold Johannes Meinel (leo@meinel.dev)
 * -----
 * Copyright (c) 2026 Leopold Johannes Meinel & contributors
 * SPDX ID: Apache-2.0
 * URL: https://www.apache.org/licenses/LICENSE-2.0
 */

//! This stores warning messages

/// Warning on incomplete animation data
pub(crate) const WARN_INCOMPLETE_ANIMATION_DATA: &str = "The animation data is incomplete.";
/// Warning on incomplete asset data
pub(crate) const WARN_INCOMPLETE_ASSET_DATA: &str = "The asset data is incomplete.";
/// Warning on incomplete collision data
pub(crate) const WARN_INCOMPLETE_COLLISION_DATA_FALLBACK: &str =
    "The collision data is incomplete. Using fallback ball collider.";
/// Warning on incomplete tile data
pub(crate) const WARN_INCOMPLETE_TILE_DATA: &str = "The tile data is incomplete.";
