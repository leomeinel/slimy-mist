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

//! Main with [`AppPlugin`]

// Support configuring Bevy lints within code.
#![cfg_attr(bevy_lint, feature(register_tool), register_tool(bevy))]
// Disable console on Windows for non-dev builds.
#![cfg_attr(not(feature = "dev"), windows_subsystem = "windows")]

mod audio;
mod characters;
#[cfg(feature = "dev")]
mod dev_tools;
mod levels;
mod logging;
mod menus;
mod screens;
mod theme;
mod utils;

use bevy::{asset::AssetMetaCheck, color::palettes::tailwind, prelude::*, window::WindowResized};
use bevy_ecs_tilemap::TilemapPlugin;
use bevy_light_2d::prelude::*;
use bevy_prng::WyRand;
use bevy_rand::plugin::EntropyPlugin;
use bevy_rapier2d::plugin::RapierPhysicsPlugin;

use crate::characters::player::Player;

/// Main function
fn main() -> AppExit {
    App::new().add_plugins(AppPlugin).run()
}

/// AppPlugin that adds everything this app needs to run
struct AppPlugin;
impl Plugin for AppPlugin {
    fn build(&self, app: &mut App) {
        // Add Bevy plugins.
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
            characters::plugin,
            #[cfg(feature = "dev")]
            dev_tools::plugin,
            levels::plugin,
            menus::plugin,
            screens::plugin,
            theme::plugin,
        ));

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

        // Set up the `Pause` state and resource.
        app.init_state::<Pause>();
        app.configure_sets(Update, PausableSystems.run_if(in_state(Pause(false))));

        // Spawn the main camera.
        app.add_systems(Startup, spawn_camera);

        app.add_systems(Update, (fit_canvas, update_camera));
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

/// Whether or not the game is paused.
#[derive(States, Copy, Clone, Eq, PartialEq, Hash, Debug, Default)]
struct Pause(pub(crate) bool);

/// A system set for systems that shouldn't run while the game is paused.
#[derive(SystemSet, Copy, Clone, Eq, PartialEq, Hash, Debug)]
struct PausableSystems;

/// Camera that renders the world to the canvas.
#[derive(Component)]
struct CanvasCamera;

/// Color for the ambient light: rgb(254, 243, 199)
const AMBIENT_LIGHT_COLOR: Srgba = tailwind::AMBER_100;

/// Spawn [`Camera2d`]
fn spawn_camera(mut commands: Commands) {
    commands.spawn((
        Name::new("Canvas Camera"),
        Camera2d,
        Msaa::Off,
        CanvasCamera,
        Light2d {
            ambient_light: AmbientLight2d {
                color: AMBIENT_LIGHT_COLOR.into(),
                ..default()
            },
        },
    ));
}

/// In-game resolution height.
const RES_HEIGHT: f32 = 180.;

/// Scales camera projection to fit the window (integer multiples only).
///
/// Heavily inspired by: <https://bevy.org/examples/2d-rendering/pixel-grid-snap/>
fn fit_canvas(
    mut msgs: MessageReader<WindowResized>,
    mut projection: Single<&mut Projection, With<CanvasCamera>>,
) {
    let Projection::Orthographic(projection) = &mut **projection else {
        return;
    };
    for msg in msgs.read() {
        let scale_factor = 1. / (msg.height / RES_HEIGHT).round();
        projection.scale = scale_factor;
    }
}

/// How quickly should the camera snap to the target location.
const CAMERA_DECAY_RATE: f32 = 3.;

/// Update the camera position by tracking the player.
///
/// Heavily inspired by: <https://bevy.org/examples/camera/2d-top-down-camera/>
fn update_camera(
    mut camera: Single<&mut Transform, (With<CanvasCamera>, Without<Player>)>,
    player: Single<&Transform, (With<Player>, Without<CanvasCamera>)>,
    time: Res<Time>,
) {
    let Vec3 { x, y, .. } = player.translation;
    let direction = Vec3::new(x, y, camera.translation.z);

    // Applies a smooth effect to camera movement using stable interpolation
    // between the camera position and the player position on the x and y axes.
    camera
        .translation
        .smooth_nudge(&direction, CAMERA_DECAY_RATE, time.delta_secs());
}
