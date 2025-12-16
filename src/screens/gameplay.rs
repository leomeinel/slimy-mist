/*
 * File: gameplay.rs
 * Author: Leopold Johannes Meinel (leo@meinel.dev)
 * -----
 * Copyright (c) 2025 Leopold Johannes Meinel & contributors
 * SPDX ID: Apache-2.0
 * URL: https://www.apache.org/licenses/LICENSE-2.0
 * -----
 * Heavily inspired by: https://github.com/TheBevyFlock/bevy_new_2d
 */

//! The screen state for the main gameplay.

use bevy::{input::common_conditions::input_just_pressed, prelude::*};

use crate::{
    Pause,
    levels::{
        despawn_chunks,
        overworld::{Overworld, OverworldAssets, spawn_overworld},
        spawn_chunks,
    },
    menus::Menu,
    screens::Screen,
};

pub(super) fn plugin(app: &mut App) {
    // Spawn overworld
    app.add_systems(OnEnter(Screen::Gameplay), spawn_overworld);

    // Start spawning/despawning chunks
    app.add_systems(
        Update,
        (
            spawn_chunks::<Overworld, OverworldAssets>,
            despawn_chunks::<Overworld>,
        )
            .run_if(in_state(Screen::Gameplay)),
    );

    // Open pause on pressing P or Escape and pause game
    app.add_systems(
        Update,
        (
            (pause, spawn_pause_overlay, open_pause_menu).run_if(
                in_state(Screen::Gameplay)
                    .and(in_state(Menu::None))
                    .and(input_just_pressed(KeyCode::KeyP).or(input_just_pressed(KeyCode::Escape))),
            ),
            close_menu.run_if(
                in_state(Screen::Gameplay)
                    .and(not(in_state(Menu::None)))
                    .and(input_just_pressed(KeyCode::KeyP)),
            ),
        ),
    );
    // Exit pause menu on pressing Escape and unpause game
    app.add_systems(OnExit(Screen::Gameplay), (close_menu, unpause));

    // Unpause if in no menu and in gameplay screen
    app.add_systems(
        OnEnter(Menu::None),
        unpause.run_if(in_state(Screen::Gameplay)),
    );
}

/// Unpause the game
fn unpause(mut next_pause: ResMut<NextState<Pause>>) {
    next_pause.set(Pause(false));
}

/// Pause the game
fn pause(mut next_pause: ResMut<NextState<Pause>>) {
    next_pause.set(Pause(true));
}

/// rgba(0, 0, 0, 204)
const BACKGROUND_COLOR: Color = Color::srgba(0.0, 0.0, 0.0, 0.8);

/// Spawn pause overlay
fn spawn_pause_overlay(mut commands: Commands) {
    commands.spawn((
        Name::new("Pause Overlay"),
        Node {
            width: percent(100),
            height: percent(100),
            ..default()
        },
        GlobalZIndex(1),
        BackgroundColor(BACKGROUND_COLOR),
        DespawnOnExit(Pause(true)),
    ));
}

/// Open pause menu
fn open_pause_menu(mut next_menu: ResMut<NextState<Menu>>) {
    next_menu.set(Menu::Pause);
}

/// Close pause menu
fn close_menu(mut next_menu: ResMut<NextState<Menu>>) {
    next_menu.set(Menu::None);
}
