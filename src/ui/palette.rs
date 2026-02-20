/*
 * File: palette.rs
 * Author: Leopold Johannes Meinel (leo@meinel.dev)
 * -----
 * Copyright (c) 2026 Leopold Johannes Meinel & contributors
 * SPDX ID: Apache-2.0
 * URL: https://www.apache.org/licenses/LICENSE-2.0
 * -----
 * Heavily inspired by: https://github.com/TheBevyFlock/bevy_new_2d
 */

use bevy::{color::palettes::tailwind, prelude::*};

/// Color for text label
pub(crate) const LABEL_TEXT: Srgba = tailwind::NEUTRAL_50;
/// Color for header text
pub(crate) const HEADER_TEXT: Srgba = tailwind::NEUTRAL_50;

/// Color for button text
pub(crate) const BUTTON_TEXT: Srgba = tailwind::NEUTRAL_50;
/// Background color for button base
pub(crate) const BUTTON_BASE_BACKGROUND: Srgba = tailwind::SKY_700;
/// Background color for button
pub(crate) const BUTTON_BACKGROUND: Srgba = tailwind::SKY_500;
/// Background color for button if hovered
pub(crate) const BUTTON_HOVERED_BACKGROUND: Srgba = tailwind::SKY_300;
/// Background color for button if pressed
pub(crate) const BUTTON_PRESSED_BACKGROUND: Color = /*Color::NONE*/
    Color::srgba(0., 0., 0., 0.8);

/// Background color for the pause screen.
pub(crate) const PAUSE_BACKGROUND: Color = Color::srgba(0., 0., 0., 0.8);
/// Background color for the splash screen.
pub(crate) const CLEAR_BACKGROUND: Srgba = tailwind::SLATE_500;

/// Color for the debug navmesh
#[cfg(feature = "dev")]
pub(crate) const DEBUG_NAVMESH: Srgba = tailwind::AMBER_500;
/// Color for the debug obstacle used in the debug navmesh
#[cfg(feature = "dev")]
pub(crate) const DEBUG_OBSTACLE: Srgba = tailwind::VIOLET_800;
/// Color for the debug path used in the debug navmesh
#[cfg(feature = "dev")]
pub(crate) const DEBUG_PATH: Srgba = tailwind::FUCHSIA_500;

/// Color for the joystick image
#[cfg(any(target_os = "android", target_os = "ios"))]
pub(crate) const JOYSTICK_IMAGE: Srgba = tailwind::NEUTRAL_50;
