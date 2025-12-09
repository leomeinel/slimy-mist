/*
 * File: palette.rs
 * Author: Leopold Johannes Meinel (leo@meinel.dev)
 * -----
 * Copyright (c) 2025 Leopold Johannes Meinel & contributors
 * SPDX ID: Apache-2.0
 * URL: https://www.apache.org/licenses/LICENSE-2.0
 * -----
 * Heavily inspired by: https://github.com/TheBevyFlock/bevy_new_2d
 */

use bevy::{color::palettes::tailwind, prelude::*};

/// rgb(74, 222, 128)
pub(crate) const LABEL_TEXT: Srgba = tailwind::GREEN_400;

/// rgb(187, 247, 208)
pub(crate) const HEADER_TEXT: Srgba = tailwind::GREEN_200;

/// rgb(229, 229, 229)
pub(crate) const BUTTON_TEXT: Srgba = tailwind::NEUTRAL_200;
/// rgb(6, 182, 212)
pub(crate) const BUTTON_BACKGROUND: Srgba = tailwind::CYAN_500;
/// rgb(103, 232, 249)
pub(crate) const BUTTON_HOVERED_BACKGROUND: Srgba = tailwind::CYAN_300;
/// rgb(14, 116, 144)
pub(crate) const BUTTON_PRESSED_BACKGROUND: Srgba = tailwind::CYAN_700;
