/*
 * File: credits.rs
 * Author: Leopold Johannes Meinel (leo@meinel.dev)
 * -----
 * Copyright (c) 2026 Leopold Johannes Meinel & contributors
 * SPDX ID: Apache-2.0
 * URL: https://www.apache.org/licenses/LICENSE-2.0
 * -----
 * Heavily inspired by: https://github.com/TheBevyFlock/bevy_new_2d
 */

//! The credits menu.

// FIXME: Add auto scrolling, also prevent overflow on small displays, make adaptive.

use bevy::{ecs::spawn::SpawnIter, input::common_conditions::input_just_pressed, prelude::*};
use bevy_asset_loader::prelude::*;

use crate::{audio::music, menus::Menu, ui::prelude::*};

pub(super) fn plugin(app: &mut App) {
    // Open credits menu and start music
    app.add_systems(
        OnEnter(Menu::Credits),
        (spawn_credits_menu, start_credits_music),
    );

    // Exit credits menu on pressing Escape
    app.add_systems(
        Update,
        go_back.run_if(in_state(Menu::Credits).and(input_just_pressed(KeyCode::Escape))),
    );
}

/// Assets for credits
#[derive(AssetCollection, Resource)]
pub(crate) struct CreditsAssets {
    #[asset(path = "audio/music/screen-saver.ogg")]
    music: Handle<AudioSource>,
}

/// Spawn menu with credits for assets and creators of the game
fn spawn_credits_menu(mut commands: Commands) {
    commands.spawn((
        widgets::ui_root("Credits Menu"),
        GlobalZIndex(2),
        DespawnOnExit(Menu::Credits),
        children![
            widgets::header("Created by"),
            created_by(),
            widgets::header("Assets"),
            assets(),
            widgets::button("Back", go_back_on_click),
        ],
    ));
}

// FIXME: Get data from ron instead of having it in code.
/// Grid for created by section
fn created_by() -> impl Bundle {
    grid(vec![
        ["Leopold Meinel", "Game design and programming"],
        ["Shave", "Sprites published under CC0-1.0"],
    ])
}

// FIXME: Get data from ron instead of having it in code.
/// Grid for assets section
fn assets() -> impl Bundle {
    grid(vec![
        [
            "Code & Structure",
            "CC0-1.0/Apache-2.0/MIT by bevy_new_2d and contributors",
        ],
        [
            "Code & Game Engine",
            "Apache-2.0/MIT by bevyengine and contributors",
        ],
        ["Music", "CC0-1.0 by freepd.com and creators"],
        ["SFX", "CC0-1.0 by Jaszunio15"],
        ["SFX", "CC0-1.0 by OwlishMedia"],
        ["SFX", "CC-BY-4.0/CC-BY-3.0 by leohpaz"],
        ["Fonts", "OFL-1.1 by Google Fonts"],
    ])
}

/// Grid with custom settings that fit the credits screen
fn grid(content: Vec<[&'static str; 2]>) -> impl Bundle {
    (
        Name::new("Grid"),
        Node {
            display: Display::Grid,
            row_gap: px(10),
            column_gap: px(30),
            grid_template_columns: RepeatedGridTrack::px(2, 400.0),
            ..default()
        },
        Children::spawn(SpawnIter(content.into_iter().flatten().enumerate().map(
            |(i, text)| {
                (
                    widgets::label(text),
                    Node {
                        justify_self: if i.is_multiple_of(2) {
                            JustifySelf::End
                        } else {
                            JustifySelf::Start
                        },
                        ..default()
                    },
                )
            },
        ))),
    )
}

/// Go back to main menu on click
fn go_back_on_click(_: On<Pointer<Click>>, mut next_state: ResMut<NextState<Menu>>) {
    next_state.set(Menu::Main);
}

/// Go back to main menu if a menu switch is initialized
fn go_back(mut next_state: ResMut<NextState<Menu>>) {
    next_state.set(Menu::Main);
}

/// Play music for credits
fn start_credits_music(mut commands: Commands, credits_music: Res<CreditsAssets>) {
    commands.spawn((
        Name::new("Credits Music"),
        DespawnOnExit(Menu::Credits),
        music(credits_music.music.clone()),
    ));
}
