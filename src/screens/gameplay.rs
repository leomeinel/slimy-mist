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
use crate::mobile::{
    JoystickID, JoystickInteractionRectMap, setup_joystick_interaction_rect_map, spawn_joystick,
};
use crate::{
    Pause,
    animations::setup_animations,
    camera::center_camera_on_player,
    characters::{nav::NavTargetPosMap, npc::Slime, player::Player},
    input::PointerInputCache,
    levels::overworld::{Overworld, OverworldProcGen, spawn_overworld},
    lighting::{DayTimer, StreetLight},
    menus::Menu,
    procgen::{ProcGenCache, navmesh::spawn_navmesh},
    screens::Screen,
    utils::run_conditions::window_unfocused,
    visual::{
        Visible,
        layers::{DisplayImage, LayerDataRelatedCache},
    },
};

pub(super) fn plugin(app: &mut App) {
    // Order new `InitGameplaySystems` variants by adding them here:
    app.configure_sets(
        OnEnter(Screen::Gameplay),
        (
            InitGameplaySystems::Resources,
            InitGameplaySystems::Finalize,
        )
            .chain(),
    );

    // Run `InitGameplaySystems::Resources`
    app.add_systems(
        OnEnter(Screen::Gameplay),
        (
            insert_resources,
            insert_display_image::<Player>,
            insert_display_image::<Slime>,
        )
            .in_set(InitGameplaySystems::Resources),
    );
    app.add_systems(OnExit(Screen::Gameplay), remove_resources);

    // Exit pause menu that was used to exit and unpause game
    app.add_systems(OnExit(Screen::Gameplay), (close_menu, unpause));
    // Unpause if in no menu and in gameplay screen
    app.add_systems(
        OnEnter(Menu::None),
        unpause.run_if(in_state(Screen::Gameplay)),
    );

    // Run `InitGameplaySystems::Finalize`
    app.add_systems(
        OnEnter(Screen::Gameplay),
        (
            (setup_animations::<Player>, setup_animations::<Slime>),
            spawn_overworld,
            center_camera_on_player,
            spawn_navmesh::<OverworldProcGen, Overworld>,
            #[cfg(any(target_os = "android", target_os = "ios"))]
            spawn_joystick::<{ JoystickID::Movement as u8 }>,
        )
            .in_set(InitGameplaySystems::Finalize)
            .chain(),
    );

    // Setup `JoystickInteractionRectMap`
    #[cfg(any(target_os = "android", target_os = "ios"))]
    app.add_systems(
        PostUpdate,
        setup_joystick_interaction_rect_map::<{ JoystickID::Movement as u8 }>
            .after(InitGameplaySystems::Finalize)
            .after(TransformSystems::Propagate)
            .run_if(in_state(Screen::Gameplay).and(run_once)),
    );

    // Open pause on pressing P or Escape and pause game
    app.add_systems(
        Update,
        (
            (pause, spawn_pause_overlay, open_pause_menu).run_if(
                in_state(Menu::None).and(
                    input_just_pressed(KeyCode::KeyP)
                        .or(input_just_pressed(KeyCode::Escape))
                        .or(window_unfocused),
                ),
            ),
            close_menu.run_if(not(in_state(Menu::None)).and(input_just_pressed(KeyCode::KeyP))),
        )
            .run_if(in_state(Screen::Gameplay)),
    );
}

/// A [`SystemSet`] for systems that initialize [`Screen::Gameplay`]
#[derive(SystemSet, Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub(crate) enum InitGameplaySystems {
    Resources,
    Finalize,
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

/// Insert [`DisplayImage`].
///
/// ## Traits
///
/// - `T` must implement [`Visible`].
pub(crate) fn insert_display_image<T>(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    data: Res<LayerDataRelatedCache<T>>,
) where
    T: Visible,
{
    commands.insert_resource(data.to_display_image(&mut images));
}

/// Insert [`Resource`]s
fn insert_resources(mut commands: Commands) {
    commands.init_resource::<DayTimer>();
    commands.init_resource::<PointerInputCache>();
    commands.init_resource::<NavTargetPosMap>();
    #[cfg(any(target_os = "android", target_os = "ios"))]
    commands.init_resource::<JoystickInteractionRectMap>();
    commands.init_resource::<ProcGenCache<OverworldProcGen>>();
    commands.init_resource::<ProcGenCache<Slime>>();
    commands.init_resource::<ProcGenCache<StreetLight>>();
}

/// Remove [`Resource`]s
fn remove_resources(mut commands: Commands) {
    commands.remove_resource::<DayTimer>();
    commands.remove_resource::<DisplayImage<Player>>();
    commands.remove_resource::<DisplayImage<Slime>>();
    commands.remove_resource::<PointerInputCache>();
    commands.remove_resource::<NavTargetPosMap>();
    #[cfg(any(target_os = "android", target_os = "ios"))]
    commands.remove_resource::<JoystickInteractionRectMap>();
    commands.remove_resource::<ProcGenCache<OverworldProcGen>>();
    commands.remove_resource::<ProcGenCache<Slime>>();
    commands.remove_resource::<ProcGenCache<StreetLight>>();
}
