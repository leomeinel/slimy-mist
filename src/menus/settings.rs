/*
 * File: settings.rs
 * Author: Leopold Johannes Meinel (leo@meinel.dev)
 * -----
 * Copyright (c) 2025 Leopold Johannes Meinel & contributors
 * SPDX ID: Apache-2.0
 * URL: https://www.apache.org/licenses/LICENSE-2.0
 * -----
 * Heavily inspired by: https://github.com/TheBevyFlock/bevy_new_2d
 */

//! The settings menu.
//!
//! Additional settings and accessibility options should go here.

use bevy::{audio::Volume, input::common_conditions::input_just_pressed, prelude::*};

use crate::{menus::Menu, screens::Screen, theme::prelude::*};

pub(super) fn plugin(app: &mut App) {
    // Open settings menu on state
    app.add_systems(OnEnter(Menu::Settings), spawn_settings_menu);

    // Exit settings menu on pressing Escape
    app.add_systems(
        Update,
        go_back.run_if(in_state(Menu::Settings).and(input_just_pressed(KeyCode::Escape))),
    );
    // Handle changes to global volume from settings menu
    app.add_systems(
        Update,
        update_global_volume_label.run_if(in_state(Menu::Settings)),
    );
}

/// Global volume label marker
#[derive(Component, Reflect)]
#[reflect(Component)]
struct GlobalVolumeLabel;

/// Spawn settings menu
fn spawn_settings_menu(mut commands: Commands) {
    commands.spawn((
        widgets::common::ui_root("Settings Menu"),
        GlobalZIndex(2),
        DespawnOnExit(Menu::Settings),
        children![
            widgets::common::header("Settings"),
            grid(),
            widgets::common::button("Back", go_back_on_click),
        ],
    ));
}

/// Grid with custom settings that fit the settings screen
fn grid() -> impl Bundle {
    (
        Name::new("Settings Grid"),
        Node {
            display: Display::Grid,
            row_gap: px(10),
            column_gap: px(30),
            grid_template_columns: RepeatedGridTrack::px(2, 400.0),
            ..default()
        },
        children![
            (
                widgets::common::label("Master Volume"),
                Node {
                    justify_self: JustifySelf::End,
                    ..default()
                }
            ),
            global_volume_widget(),
        ],
    )
}

/// Widget to adjust global volume
fn global_volume_widget() -> impl Bundle {
    (
        Name::new("Global Volume Widget"),
        Node {
            justify_self: JustifySelf::Start,
            ..default()
        },
        children![
            widgets::common::button_small("-", lower_global_volume),
            (
                Name::new("Current Volume"),
                Node {
                    padding: UiRect::horizontal(px(10)),
                    justify_content: JustifyContent::Center,
                    ..default()
                },
                children![(widgets::common::label(""), GlobalVolumeLabel)],
            ),
            widgets::common::button_small("+", raise_global_volume),
        ],
    )
}

/// Minimum global volume
const MIN_VOLUME: f32 = 0.0;
/// Maximum global volume
const MAX_VOLUME: f32 = 3.0;

/// Lower global volume
fn lower_global_volume(_: On<Pointer<Click>>, mut global_volume: ResMut<GlobalVolume>) {
    let linear = (global_volume.volume.to_linear() - 0.1).max(MIN_VOLUME);
    global_volume.volume = Volume::Linear(linear);
}

/// Raise global volume
fn raise_global_volume(_: On<Pointer<Click>>, mut global_volume: ResMut<GlobalVolume>) {
    let linear = (global_volume.volume.to_linear() + 0.1).min(MAX_VOLUME);
    global_volume.volume = Volume::Linear(linear);
}

/// Update global volume label that displays volume
fn update_global_volume_label(
    mut label: Single<&mut Text, With<GlobalVolumeLabel>>,
    global_volume: Res<GlobalVolume>,
) {
    let percent = 100.0 * global_volume.volume.to_linear();
    label.0 = format!("{percent:3.0}%");
}

/// Go back on pointer click
fn go_back_on_click(
    _: On<Pointer<Click>>,
    screen: Res<State<Screen>>,
    mut next_menu: ResMut<NextState<Menu>>,
) {
    next_menu.set(if screen.get() == &Screen::Title {
        Menu::Main
    } else {
        Menu::Pause
    });
}

/// Go back manually
fn go_back(screen: Res<State<Screen>>, mut next_menu: ResMut<NextState<Menu>>) {
    next_menu.set(if screen.get() == &Screen::Title {
        Menu::Main
    } else {
        Menu::Pause
    });
}
