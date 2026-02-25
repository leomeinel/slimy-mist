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
pub(crate) const LABEL_TEXT: Color = Color::WHITE;
/// Color for header text
pub(crate) const HEADER_TEXT: Color = Color::WHITE;

/// Color for button text
pub(crate) const BUTTON_TEXT: Color = Color::WHITE;
/// Background color for button base
pub(crate) const BUTTON_BASE_BACKGROUND: Srgba = tailwind::SKY_700;
/// Background color for button
pub(crate) const BUTTON_BACKGROUND: Srgba = tailwind::SKY_600;
/// Background color for button if hovered
pub(crate) const BUTTON_HOVERED_BACKGROUND: Srgba = tailwind::SKY_500;
/// Background color for button if pressed
pub(crate) const BUTTON_PRESSED_BACKGROUND: Color = Color::NONE;

/// Background color for switch button base if toggled on
pub(crate) const SWITCH_BASE_ON_BACKGROUND: Srgba = tailwind::GREEN_700;
/// Background color for switch button if toggled on
pub(crate) const SWITCH_ON_BACKGROUND: Srgba = tailwind::GREEN_600;
/// Background color for switch button if toggled on and hovered
pub(crate) const SWITCH_ON_HOVERED_BACKGROUND: Srgba = tailwind::GREEN_500;
/// Background color for switch button base if toggled off
pub(crate) const SWITCH_BASE_OFF_BACKGROUND: Srgba = tailwind::RED_700;
/// Background color for switch button if toggled off
pub(crate) const SWITCH_OFF_BACKGROUND: Srgba = tailwind::RED_600;
/// Background color for switch button if toggled off and hovered
pub(crate) const SWITCH_OFF_HOVERED_BACKGROUND: Srgba = tailwind::RED_500;

/// Background color for pause screen.
pub(crate) const PAUSE_BACKGROUND: Color = Color::srgba(0., 0., 0., 0.8);
/// Background color for splash screen.
pub(crate) const CLEAR_BACKGROUND: Srgba = tailwind::SLATE_700;

/// Color for joystick image
pub(crate) const JOYSTICK_IMAGE: Color = Color::WHITE;

/// Color for debug navmesh
#[cfg(feature = "dev")]
pub(crate) const DEBUG_NAVMESH: Srgba = tailwind::AMBER_500;
/// Color for debug obstacle used in the debug navmesh
#[cfg(feature = "dev")]
pub(crate) const DEBUG_OBSTACLE: Srgba = tailwind::VIOLET_800;
/// Color for debug path used in the debug navmesh
#[cfg(feature = "dev")]
pub(crate) const DEBUG_PATH: Srgba = tailwind::FUCHSIA_500;
