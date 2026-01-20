/*
 * File: gameplay.rs
 * Author: Leopold Johannes Meinel (leo@meinel.dev)
 * -----
 * Copyright (c) 2026 Leopold Johannes Meinel & contributors
 * SPDX ID: Apache-2.0
 * URL: https://www.apache.org/licenses/LICENSE-2.0
 * -----
 * Heavily inspired by: https://github.com/TheBevyFlock/bevy_new_2d
 */

//! The screen state for the main gameplay.

use bevy::{input::common_conditions::input_just_pressed, prelude::*};

#[cfg(any(target_os = "android", target_os = "ios"))]
use crate::mobile::spawn_joystick;
use crate::{
    Pause,
    camera::center_camera_on_player,
    characters::{
        VisualMap,
        animations::setup_animations,
        nav::NavTargetPosMap,
        npc::{Slime, SlimeAssets},
        player::{Player, PlayerAssets},
    },
    levels::overworld::{Overworld, OverworldProcGen, spawn_overworld},
    menus::Menu,
    procgen::{ProcGenController, navmesh::spawn_navmesh},
    screens::Screen,
    visual::particles::{ParticleMap, ParticleWalkingDust},
};

pub(super) fn plugin(app: &mut App) {
    // Insert/Remove resources and cache deserialized data in resources
    app.add_systems(
        OnEnter(Screen::Gameplay),
        insert_resources.in_set(PrepareGameplaySystems),
    );
    app.add_systems(OnExit(Screen::Gameplay), remove_resources);

    // Exit pause menu that was used to exit and unpause game
    app.add_systems(OnExit(Screen::Gameplay), (close_menu, unpause));
    // Unpause if in no menu and in gameplay screen
    app.add_systems(
        OnEnter(Menu::None),
        unpause.run_if(in_state(Screen::Gameplay)),
    );

    // Spawn overworld with navmesh and run required systems
    app.add_systems(
        OnEnter(Screen::Gameplay),
        (
            (
                setup_animations::<Player, PlayerAssets>,
                setup_animations::<Slime, SlimeAssets>,
            ),
            spawn_overworld,
            center_camera_on_player,
            spawn_navmesh::<OverworldProcGen, Overworld>,
            #[cfg(any(target_os = "android", target_os = "ios"))]
            spawn_joystick,
        )
            .after(PrepareGameplaySystems)
            .chain(),
    );

    // Open pause on pressing P or Escape and pause game
    app.add_systems(
        Update,
        (
            (pause, spawn_pause_overlay, open_pause_menu)
                .run_if(in_state(Menu::None).and(
                    input_just_pressed(KeyCode::KeyP).or(input_just_pressed(KeyCode::Escape)),
                )),
            close_menu.run_if(not(in_state(Menu::None)).and(input_just_pressed(KeyCode::KeyP))),
        )
            .run_if(in_state(Screen::Gameplay)),
    );
}

/// A system set for systems that inserts [`Resource`]s dynamically for [`Screen::Gameplay`]
#[derive(SystemSet, Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub(crate) struct PrepareGameplaySystems;

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
fn open_pause_menu(mut next_state: ResMut<NextState<Menu>>) {
    next_state.set(Menu::Pause);
}

/// Close pause menu
fn close_menu(mut next_state: ResMut<NextState<Menu>>) {
    next_state.set(Menu::None);
}

/// Unpause the game
fn unpause(mut next_state: ResMut<NextState<Pause>>) {
    next_state.set(Pause(false));
}

/// Pause the game
fn pause(mut next_state: ResMut<NextState<Pause>>) {
    next_state.set(Pause(true));
}

/// Insert resources for [`crate::procgen`]
fn insert_resources(mut commands: Commands) {
    commands.init_resource::<NavTargetPosMap>();
    commands.init_resource::<ProcGenController<OverworldProcGen>>();
    commands.init_resource::<ProcGenController<Slime>>();
    commands.init_resource::<ParticleMap<ParticleWalkingDust>>();
    commands.init_resource::<VisualMap>();
}

/// Remove resources for [`crate::procgen`]
fn remove_resources(mut commands: Commands) {
    commands.remove_resource::<NavTargetPosMap>();
    commands.remove_resource::<ProcGenController<OverworldProcGen>>();
    commands.remove_resource::<ProcGenController<Slime>>();
    commands.remove_resource::<ParticleMap<ParticleWalkingDust>>();
    commands.remove_resource::<VisualMap>();
}
