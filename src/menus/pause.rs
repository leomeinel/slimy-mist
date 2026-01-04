/*
 * File: pause.rs
 * Author: Leopold Johannes Meinel (leo@meinel.dev)
 * -----
 * Copyright (c) 2025 Leopold Johannes Meinel & contributors
 * SPDX ID: Apache-2.0
 * URL: https://www.apache.org/licenses/LICENSE-2.0
 * -----
 * Heavily inspired by: https://github.com/TheBevyFlock/bevy_new_2d
 */

//! The pause menu.

use bevy::{input::common_conditions::input_just_pressed, prelude::*};

use crate::{menus::Menu, screens::Screen, theme::widgets};

pub(super) fn plugin(app: &mut App) {
    // Open pause menu
    app.add_systems(OnEnter(Menu::Pause), spawn_pause_menu);

    // Exit pause menu on pressing Escape
    app.add_systems(
        Update,
        go_back.run_if(in_state(Menu::Pause).and(input_just_pressed(KeyCode::Escape))),
    );
}

/// Spawn pause menu
fn spawn_pause_menu(mut commands: Commands) {
    commands.spawn((
        widgets::common::ui_root("Pause Menu"),
        GlobalZIndex(2),
        DespawnOnExit(Menu::Pause),
        children![
            widgets::common::header("Game paused"),
            widgets::common::button("Continue", close_menu),
            widgets::common::button("Settings", open_settings_menu),
            widgets::common::button("Quit to title", quit_to_title),
        ],
    ));
}

/// Open settings
fn open_settings_menu(_: On<Pointer<Click>>, mut next_state: ResMut<NextState<Menu>>) {
    next_state.set(Menu::Settings);
}

/// Close menu via on click
fn close_menu(_: On<Pointer<Click>>, mut next_state: ResMut<NextState<Menu>>) {
    next_state.set(Menu::None);
}

/// Close menu manually
fn go_back(mut next_state: ResMut<NextState<Menu>>) {
    next_state.set(Menu::None);
}

/// Quit to title
fn quit_to_title(_: On<Pointer<Click>>, mut next_screen: ResMut<NextState<Screen>>) {
    next_screen.set(Screen::Title);
}
