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

use bevy::{prelude::*, winit::WinitSettings};

pub(super) fn plugin(app: &mut App) {
    // Add child plugins
    #[cfg(target_os = "android")]
    app.add_plugins(android::plugin);

    // Make the winit loop wait more aggressively when no user input is received
    // This can help reduce cpu usage on mobile devices
    app.insert_resource(WinitSettings::mobile());
}
