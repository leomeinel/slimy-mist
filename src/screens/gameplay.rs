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
    characters::{npc::Slime, player::Player, setup_shadow},
    levels::overworld::{Overworld, OverworldAssets, OverworldProcGen, spawn_overworld},
    menus::Menu,
    procgen::{
        ProcGenState,
        chunks::spawn_chunks,
        clear_procgen_controller, despawn_procgen,
        navigation::{
            follow_character, rebuild_nav_grid, spawn_nav_grid, update_nav_grid_agent_pos,
        },
        spawn::spawn_characters,
    },
    screens::Screen,
};

pub(super) fn plugin(app: &mut App) {
    // Spawn overworld
    app.add_systems(
        OnEnter(Screen::Gameplay),
        (
            spawn_overworld.after(setup_shadow::<Player>),
            spawn_nav_grid::<Overworld>,
        )
            .chain(),
    );

    // Start spawning/despawning chunks and characters and build nav grid
    app.add_systems(
        Update,
        (
            (
                despawn_procgen::<Slime, OverworldProcGen, false>,
                despawn_procgen::<OverworldProcGen, OverworldProcGen, true>,
            )
                .chain()
                .run_if(in_state(ProcGenState::Despawn).and(in_state(Screen::Gameplay))),
            (
                spawn_chunks::<OverworldProcGen, OverworldAssets, Overworld>,
                spawn_characters::<Slime, OverworldProcGen, Overworld>,
            )
                .chain()
                .run_if(in_state(ProcGenState::Spawn).and(in_state(Screen::Gameplay))),
            rebuild_nav_grid
                .run_if(in_state(ProcGenState::RebuildNavGrid).and(in_state(Screen::Gameplay))),
        ),
    );

    // Update agent pos after exiting `ProcGenState::RebuildNavGrid`
    app.add_systems(
        OnExit(ProcGenState::RebuildNavGrid),
        (
            update_nav_grid_agent_pos::<Player, OverworldProcGen>,
            update_nav_grid_agent_pos::<Slime, OverworldProcGen>,
            follow_character::<Slime, Player>,
        )
            .chain()
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
    // Exit pause menu that was used to exit, unpause game and clear chunks
    // and spawn points when exiting `Gameplay` screen
    app.add_systems(
        OnExit(Screen::Gameplay),
        (
            clear_procgen_controller::<OverworldProcGen>,
            clear_procgen_controller::<Slime>,
            reset_procgen_state,
            close_menu,
            unpause,
        )
            .chain(),
    );

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

/// Reset [`ProcGenState`]
fn reset_procgen_state(mut procgen_state: ResMut<NextState<ProcGenState>>) {
    procgen_state.set(ProcGenState::default());
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
