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

//! Main with [`AppPlugin`]

// Support configuring Bevy lints within code.
#![cfg_attr(bevy_lint, feature(register_tool), register_tool(bevy))]
// Disable console on Windows for non-dev builds.
#![cfg_attr(not(feature = "dev"), windows_subsystem = "windows")]

mod audio;
mod camera;
mod characters;
#[cfg(feature = "dev")]
mod dev_tools;
mod levels;
mod logging;
mod menus;
mod procgen;
mod screens;
mod theme;
mod visual;

use bevy::{asset::AssetMetaCheck, prelude::*};
use bevy_ecs_tilemap::TilemapPlugin;
use bevy_light_2d::prelude::*;
use bevy_prng::WyRand;
use bevy_rand::plugin::EntropyPlugin;
use bevy_rapier2d::plugin::RapierPhysicsPlugin;

/// Main function
fn main() -> AppExit {
    App::new().add_plugins(AppPlugin).run()
}

/// AppPlugin that adds everything this app needs to run
struct AppPlugin;
impl Plugin for AppPlugin {
    fn build(&self, app: &mut App) {
        // Add bevy plugins
        app.add_plugins((DefaultPlugins
            .set(AssetPlugin {
                // Wasm builds will check for meta files (that don't exist) if this isn't set.
                // This causes errors and even panics on web build on itch.
                // See https://github.com/bevyengine/bevy_github_ci_template/issues/48.
                meta_check: AssetMetaCheck::Never,
                ..default()
            })
            .set(WindowPlugin {
                primary_window: Window {
                    title: "bevy-slime-dodge".to_string(),
                    fit_canvas_to_parent: true,
                    ..default()
                }
                .into(),
                ..default()
            })
            .set(ImagePlugin::default_nearest()),));

        // Add library plugins
        app.add_plugins((
            EntropyPlugin::<WyRand>::default(),
            Light2dPlugin,
            RapierPhysicsPlugin::<()>::default(),
            TilemapPlugin,
        ));

        // Add other plugins.
        app.add_plugins((
            audio::plugin,
            camera::plugin,
            characters::plugin,
            procgen::plugin,
            #[cfg(feature = "dev")]
            dev_tools::plugin,
            levels::plugin,
            menus::plugin,
            screens::plugin,
            theme::plugin,
            visual::plugin,
        ));

        // Set up the `Pause` state.
        app.init_state::<Pause>();
        app.configure_sets(Update, PausableSystems.run_if(in_state(Pause(false))));

        // Order new `AppSystems` variants by adding them here:
        app.configure_sets(
            Update,
            (
                AppSystems::TickTimers,
                AppSystems::RecordInput,
                AppSystems::Update,
            )
                .chain(),
        );
    }
}

/// High-level groupings of systems for the app in the `Update` schedule.
/// When adding a new variant, make sure to order it in the `configure_sets`
/// call above.
#[derive(SystemSet, Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
enum AppSystems {
    /// Tick timers.
    TickTimers,
    /// Record player input.
    RecordInput,
    /// Do everything else (consider splitting this into further variants).
    Update,
}

/// Tracks whether the game is paused.
#[derive(States, Copy, Clone, Eq, PartialEq, Hash, Debug, Default)]
struct Pause(pub(crate) bool);

/// A system set for systems that shouldn't run while the game is paused.
#[derive(SystemSet, Copy, Clone, Eq, PartialEq, Hash, Debug)]
struct PausableSystems;
