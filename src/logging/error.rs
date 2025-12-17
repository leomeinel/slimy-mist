/*
 * File: error.rs
 * Author: Leopold Johannes Meinel (leo@meinel.dev)
 * -----
 * Copyright (c) 2025 Leopold Johannes Meinel & contributors
 * SPDX ID: Apache-2.0
 * URL: https://www.apache.org/licenses/LICENSE-2.0
 */

/// Error message if loading collision data failed
pub(crate) const ERR_LOADING_COLLISION_DATA: &str =
    "Could not load collision data. The file is probably missing.";
/// Error message if loading animation data failed
pub(crate) const ERR_LOADING_ANIMATION_DATA: &str =
    "Could not load animation data. The file is probably missing.";
/// Error message if loading tile data failed
pub(crate) const ERR_LOADING_TILE_DATA: &str =
    "Could not load tile data. The file is probably missing.";

pub(crate) const ERR_SPRITE_IMAGE_NOT_LOADED: &str =
    "The given image for the sprite sheet has not been loaded successfully. This is a bug.";
pub(crate) const ERR_INVALID_REQUIRED_ANIMATION_DATA: &str =
    "The loaded animation data for required idle animation is invalid or incomplete.";
pub(crate) const ERR_UNINITIALIZED_REQUIRED_ANIMATION: &str =
    "The loaded animation data for required idle animation is invalid or incomplete.";
