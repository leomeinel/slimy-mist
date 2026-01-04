/*
 * File: title.rs
 * Author: Leopold Johannes Meinel (leo@meinel.dev)
 * -----
 * Copyright (c) 2025 Leopold Johannes Meinel & contributors
 * SPDX ID: Apache-2.0
 * URL: https://www.apache.org/licenses/LICENSE-2.0
 * -----
 * Heavily inspired by: https://github.com/TheBevyFlock/bevy_new_2d
 */

//! The title screen that appears after the splash screen.

use bevy::prelude::*;

use crate::{menus::Menu, screens::Screen};

pub(super) fn plugin(app: &mut App) {
    // Open main menu
    app.add_systems(OnEnter(Screen::Title), open_main_menu);

    // Close main menu
    app.add_systems(OnExit(Screen::Title), close_menu);
}

/// Open main menu
fn open_main_menu(mut next_state: ResMut<NextState<Menu>>) {
    next_state.set(Menu::Main);
}

/// Close main menu
fn close_menu(mut next_state: ResMut<NextState<Menu>>) {
    next_state.set(Menu::None);
}
