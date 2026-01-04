/*
 * File: error.rs
 * Author: Leopold Johannes Meinel (leo@meinel.dev)
 * -----
 * Copyright (c) 2026 Leopold Johannes Meinel & contributors
 * SPDX ID: Apache-2.0
 * URL: https://www.apache.org/licenses/LICENSE-2.0
 */

/// Error message if loading animation data failed
pub(crate) const ERR_LOADING_ANIMATION_DATA: &str =
    "Could not load animation data. The file is probably missing.";
/// Error message if loading collision data failed
pub(crate) const ERR_LOADING_COLLISION_DATA: &str =
    "Could not load collision data. The file is probably missing.";
/// Error message if loading tile data failed
pub(crate) const ERR_LOADING_TILE_DATA: &str =
    "Could not load tile data. The file is probably missing.";

/// Error message if an error has been encountered while calculating minimum chunk pos
pub(crate) const ERR_INVALID_MINIMUM_CHUNK_POS: &str =
    "Could not determine correct minimum chunk position. This is a bug.";
/// Error message if animation data is invalid or incomplete
///
/// Since only the idle animation is required, the error message includes that.
pub(crate) const ERR_INVALID_REQUIRED_ANIMATION_DATA: &str =
    "The animation data for required idle animation is invalid or incomplete.";
/// Error message if an error has been encountered while processing visual_map
pub(crate) const ERR_INVALID_VISUAL_MAP: &str = "The loaded visual map is invalid. This is a bug.";

/// Error message if sprite image is not loaded
pub(crate) const ERR_NOT_LOADED_SPRITE_IMAGE: &str =
    "The given image for the sprite sheet is not loaded. This is a bug.";

/// Error message if animation has not been initialized
pub(crate) const ERR_UNINITIALIZED_REQUIRED_ANIMATION: &str =
    "The requested animation has not been initialized.";
