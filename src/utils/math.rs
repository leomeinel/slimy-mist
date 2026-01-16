/*
 * File: math.rs
 * Author: Leopold Johannes Meinel (leo@meinel.dev)
 * -----
 * Copyright (c) 2026 Leopold Johannes Meinel & contributors
 * SPDX ID: Apache-2.0
 * URL: https://www.apache.org/licenses/LICENSE-2.0
 */

use bevy::math::*;

/// Trait for comparisons corresponding to approximate equivalence relations.
pub(crate) trait ApproxEq {
    fn is_near_zero(&self, epsilon: f32) -> bool;
}

impl ApproxEq for f32 {
    fn is_near_zero(&self, epsilon: f32) -> bool {
        self.abs() < epsilon
    }
}

impl ApproxEq for Vec2 {
    fn is_near_zero(&self, epsilon: f32) -> bool {
        self.length_squared() < epsilon.squared()
    }
}
