/*
 * File: warn.rs
 * Author: Leopold Johannes Meinel (leo@meinel.dev)
 * -----
 * Copyright (c) 2026 Leopold Johannes Meinel & contributors
 * SPDX ID: Apache-2.0
 * URL: https://www.apache.org/licenses/LICENSE-2.0
 */

//! This stores warning messages

/// Warning on incomplete [`crate::animations::AnimationData`]
pub(crate) const WARN_INCOMPLETE_ANIMATION_DATA: &str = "The animation data is incomplete.";
/// Warning on incomplete asset data
pub(crate) const WARN_INCOMPLETE_ASSET_DATA: &str = "The asset data is incomplete.";
/// Warning on incomplete [`crate::characters::CollisionData`]
pub(crate) const WARN_INCOMPLETE_COLLISION_DATA: &str = "The collision data is incomplete.";
/// Warning on incomplete [`crate::characters::CollisionData`] if using a fallback ball collider.
pub(crate) const WARN_INCOMPLETE_COLLISION_DATA_FALLBACK: &str =
    "The collision data is incomplete. Using fallback ball collider.";

/// Warning on invalid [`crate::characters::attack::AttackData`]
pub(crate) const WARN_INVALID_ATTACK_DATA: &str =
    "The attack data for the fired attack is invalid.";
/// Warning on invalid [`crate::ui::directional_nav::DirectionalNavAction`]
pub(crate) const WARN_INVALID_UI_NAV: &str = "No next button found to navigate to.";
