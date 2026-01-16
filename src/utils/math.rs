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
pub(crate) trait NearEq {
    fn is_near_zero(&self, epsilon: f32) -> bool;
    fn is_near(&self, rhs: Self, epsilon: f32) -> bool;
}

impl NearEq for f32 {
    fn is_near_zero(&self, epsilon: f32) -> bool {
        self.abs() < epsilon
    }
    fn is_near(&self, rhs: Self, epsilon: f32) -> bool {
        (self - rhs).abs() < epsilon
    }
}

impl NearEq for Vec2 {
    fn is_near_zero(&self, epsilon: f32) -> bool {
        self.length_squared() < epsilon.squared()
    }
    fn is_near(&self, rhs: Self, epsilon: f32) -> bool {
        self.distance_squared(rhs) < epsilon.squared()
    }
}
