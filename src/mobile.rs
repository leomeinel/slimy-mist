/*
 * File: mobile.rs
 * Author: Leopold Johannes Meinel (leo@meinel.dev)
 * -----
 * Copyright (c) 2026 Leopold Johannes Meinel & contributors
 * SPDX ID: Apache-2.0
 * URL: https://www.apache.org/licenses/LICENSE-2.0
 * -----
 * Heavily inspired by: https://github.com/bevyengine/bevy/tree/main/examples/mobile
 */

#[cfg(target_os = "android")]
mod android;

use bevy::{
    prelude::*,
    window::{ScreenEdge, WindowMode},
    winit::WinitSettings,
};

pub(super) fn plugin(app: &mut App) {
    // Add bevy plugins
    app.add_plugins(
        DefaultPlugins
            .set(WindowPlugin {
                primary_window: Window {
                    title: "Slimy Mist".to_string(),
                    resizable: false,
                    mode: WindowMode::BorderlessFullscreen(MonitorSelection::Primary),
                    recognize_rotation_gesture: true,
                    prefers_home_indicator_hidden: true,
                    prefers_status_bar_hidden: true,
                    preferred_screen_edges_deferring_system_gestures: ScreenEdge::Bottom,
                    ..default()
                }
                .into(),
                ..default()
            })
            .set(ImagePlugin::default_nearest()),
    );

    // Add child plugins
    #[cfg(target_os = "android")]
    app.add_plugins(android::plugin);

    // Make the winit loop wait more aggressively when no user input is received
    // This can help reduce cpu usage on mobile devices
    app.insert_resource(WinitSettings::mobile());
}
