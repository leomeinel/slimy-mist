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
/// Error message if loading layer data failed
pub(crate) const ERR_LOADING_LAYER_DATA: &str =
    "Could not load layer data. The file is probably missing.";
/// Error message if loading tile data failed
pub(crate) const ERR_LOADING_TILE_DATA: &str =
    "Could not load tile data. The file is probably missing.";

/// Error message if attacker entity is invalid
pub(crate) const ERR_INVALID_ATTACKER: &str = "The attacker is invalid. This is a bug.";
/// Error message if an error has been encountered while processing [`bevy::prelude::Children`].
pub(crate) const ERR_INVALID_CHILDREN: &str = "The processed children are invalid. This is a bug.";
/// Error message if the domain for [`bevy::prelude::EasingCurve`] is invalid.
pub(crate) const ERR_INVALID_DOMAIN_EASING: &str =
    "The specified domain for an easing curve is invalid. This is a bug.";
/// Error message if a layer map is invalid.
pub(crate) const ERR_INVALID_IMAGE: &str = "The processed image is invalid. This might be due to an invalid image with no data having been referenced in any '*.ron'";
/// Error message if a layer map is invalid.
pub(crate) const ERR_INVALID_LAYER_MAP: &str = "The processed layer map is invalid. This might be due to fields in '*.layers.ron' not matching '*.layermap.ron'.";
/// Error message if an error has been encountered while calculating minimum chunk pos
pub(crate) const ERR_INVALID_MINIMUM_CHUNK_POS: &str =
    "Could not determine correct minimum chunk position. This is a bug.";
/// Error message if navmesh is invalid
pub(crate) const ERR_INVALID_NAVMESH: &str = "The navmesh is invalid. This is a bug.";
/// Error message if nav target is invalid
pub(crate) const ERR_INVALID_NAV_TARGET: &str = "The navigation target is invalid. This is a bug.";
/// Error message if [`crate::input::PointerInputCache`] is invalid
#[cfg(any(target_os = "android", target_os = "ios"))]
pub(crate) const ERR_INVALID_POINTER_CACHE: &str =
    "The pointer cache data is invalid. This is a bug.";
/// Error message if [`bevy_rapier2d::prelude::ReadRapierContext`] is invalid
pub(crate) const ERR_INVALID_RAPIER_CONTEXT: &str = "ReadRapierContext is invalid. This is a bug.";
/// Error message if animation data is invalid or incomplete
///
/// Since only the idle animation is required, the error message includes that.
pub(crate) const ERR_INVALID_REQUIRED_ANIMATION_DATA: &str =
    "The animation data for required idle animation is invalid or incomplete.";
/// Error message if an error has been encountered while processing texture atlas
pub(crate) const ERR_INVALID_TEXTURE_ATLAS: &str =
    "The loaded texture atlas is invalid. This is a bug.";

/// Error message if sprite image is not loaded
pub(crate) const ERR_NOT_LOADED_SPRITE_IMAGE: &str =
    "The given image for the sprite sheet is not loaded. This is a bug.";

/// Error message if animation has not been initialized
pub(crate) const ERR_UNINITIALIZED_REQUIRED_ANIMATION: &str =
    "The requested animation has not been initialized.";
