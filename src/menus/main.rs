/*
 * File: main.rs
 * Author: Leopold Johannes Meinel (leo@meinel.dev)
 * -----
 * Copyright (c) 2026 Leopold Johannes Meinel & contributors
 * SPDX ID: Apache-2.0
 * URL: https://www.apache.org/licenses/LICENSE-2.0
 * -----
 * Heavily inspired by: https://github.com/TheBevyFlock/bevy_new_2d
 */

//! The main menu (seen on the title screen).

use bevy::prelude::*;

use crate::{menus::Menu, screens::Screen, ui::widgets};

pub(super) fn plugin(app: &mut App) {
    // Open main menu
    app.add_systems(OnEnter(Menu::Main), spawn_main_menu);
}

fn spawn_main_menu(mut commands: Commands) {
    // Spawn Main menu with state changing buttons
    commands.spawn((
        widgets::ui_root("Main Menu"),
        GlobalZIndex(2),
        DespawnOnExit(Menu::Main),
        #[cfg(not(any(target_family = "wasm", target_os = "android", target_os = "ios")))]
        children![
            widgets::button("Play", enter_gameplay_screen),
            widgets::button("Settings", open_settings_menu),
            widgets::button("Credits", open_credits_menu),
            widgets::button("Exit", exit_app),
        ],
        // Do not add exit button for wasm, android and ios
        #[cfg(any(target_family = "wasm", target_os = "android", target_os = "ios"))]
        children![
            widgets::button("Play", enter_gameplay_screen),
            widgets::button("Settings", open_settings_menu),
            widgets::button("Credits", open_credits_menu),
        ],
    ));
}

/// Enter the gameplay screen
fn enter_gameplay_screen(_: On<Pointer<Click>>, mut next_state: ResMut<NextState<Screen>>) {
    next_state.set(Screen::Gameplay);
}

/// Open settings
fn open_settings_menu(_: On<Pointer<Click>>, mut next_state: ResMut<NextState<Menu>>) {
    next_state.set(Menu::Settings);
}

/// Open credits
fn open_credits_menu(_: On<Pointer<Click>>, mut next_state: ResMut<NextState<Menu>>) {
    next_state.set(Menu::Credits);
}

/// Exit the app
#[cfg(not(any(target_family = "wasm", target_os = "android", target_os = "ios")))]
fn exit_app(_: On<Pointer<Click>>, mut app_exit_msg: MessageWriter<AppExit>) {
    app_exit_msg.write(AppExit::Success);
}
