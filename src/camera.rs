/*
 * File: camera.rs
 * Author: Leopold Johannes Meinel (leo@meinel.dev)
 * -----
 * Copyright (c) 2026 Leopold Johannes Meinel & contributors
 * SPDX ID: Apache-2.0
 * URL: https://www.apache.org/licenses/LICENSE-2.0
 */

pub(crate) mod ysort;

use bevy::{color::palettes::tailwind, prelude::*, window::WindowResized};
use bevy_light_2d::prelude::*;

use crate::characters::player::Player;

pub(super) fn plugin(app: &mut App) {
    // Add child plugins
    app.add_plugins(ysort::plugin);

    // Spawn the main camera.
    app.add_systems(Startup, spawn_camera);

    // Update the main camera
    app.add_systems(Update, (fit_canvas, update_camera));
}

/// Z-level for the level
pub(crate) const LEVEL_Z: f32 = 1.;
/// Z-level for any foreground object
pub(crate) const DEFAULT_Z: f32 = 10.;

/// Camera that renders the world to the canvas.
#[derive(Component)]
pub(crate) struct CanvasCamera;

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

// FIXME: When reentering gameplay while having moved, since we are using Changed, the camera will not update correctly.
//        We could probably fix this by triggering a manual update when player spawns.^
/// Update the camera position by tracking the player.
///
/// Heavily inspired by: <https://bevy.org/examples/camera/2d-top-down-camera/>
fn update_camera(
    mut camera: Single<&mut Transform, (With<CanvasCamera>, Without<Player>)>,
    player: Single<&Transform, (Changed<Transform>, With<Player>, Without<CanvasCamera>)>,
    time: Res<Time>,
) {
    let target_pos = player.translation.xy().extend(camera.translation.z);

    // Applies a smooth effect to camera movement using stable interpolation
    // between the camera position and the player position on the x and y axes.
    camera
        .translation
        .smooth_nudge(&target_pos, CAMERA_DECAY_RATE, time.delta_secs());
}
