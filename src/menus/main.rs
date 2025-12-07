/*
 * File: main.rs
 * Author: Leopold Johannes Meinel (leo@meinel.dev)
 * -----
 * Copyright (c) 2025 Leopold Johannes Meinel & contributors
 * SPDX ID: Apache-2.0
 * URL: https://www.apache.org/licenses/LICENSE-2.0
 * -----
 * Heavily inspired by: https://github.com/TheBevyFlock/bevy_new_2d
 */

//! The main menu (seen on the title screen).

use bevy::prelude::*;

use crate::{menus::Menu, screens::Screen, theme::widgets};

pub(super) fn plugin(app: &mut App) {
    app.add_systems(OnEnter(Menu::Main), spawn_main_menu);
}

fn spawn_main_menu(mut commands: Commands) {
    commands.spawn((
        widgets::common::ui_root("Main Menu"),
        GlobalZIndex(2),
        DespawnOnExit(Menu::Main),
        #[cfg(not(target_family = "wasm"))]
        children![
            widgets::common::button("Play", enter_loading_or_gameplay_screen),
            widgets::common::button("Settings", open_settings_menu),
            widgets::common::button("Credits", open_credits_menu),
            widgets::common::button("Exit", exit_app),
        ],
        #[cfg(target_family = "wasm")]
        children![
            widgets::common::button("Play", enter_loading_or_gameplay_screen),
            widgets::common::button("Settings", open_settings_menu),
            widgets::common::button("Credits", open_credits_menu),
        ],
    ));
}

// FIXME: This is currently unsupported. After it is, we should implement this: https://github.com/NiklasEi/bevy_asset_loader/blob/main/bevy_asset_loader/examples/progress_tracking.rs
// See: https://github.com/NiklasEi/bevy_asset_loader/pull/259
/*
fn enter_loading_or_gameplay_screen(
    _: On<Pointer<Click>>,
    resource_handles: Res<ResourceStates>,
    mut next_screen: ResMut<NextState<Screen>>,
) {
    if resource_handles.is_all_done() {
        next_screen.set(Screen::Gameplay);
    } else {
        next_screen.set(Screen::Loading);
    }
}
*/

// FIXME: Remove this workaround after above has been adressed
fn enter_loading_or_gameplay_screen(
    _: On<Pointer<Click>>,
    mut next_screen: ResMut<NextState<Screen>>,
) {
    next_screen.set(Screen::Gameplay);
}

fn open_settings_menu(_: On<Pointer<Click>>, mut next_menu: ResMut<NextState<Menu>>) {
    next_menu.set(Menu::Settings);
}

fn open_credits_menu(_: On<Pointer<Click>>, mut next_menu: ResMut<NextState<Menu>>) {
    next_menu.set(Menu::Credits);
}

#[cfg(not(target_family = "wasm"))]
fn exit_app(_: On<Pointer<Click>>, mut app_exit: MessageWriter<AppExit>) {
    app_exit.write(AppExit::Success);
}
