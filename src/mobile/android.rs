/*
 * File: android.rs
 * Author: Leopold Johannes Meinel (leo@meinel.dev)
 * -----
 * Copyright (c) 2026 Leopold Johannes Meinel & contributors
 * SPDX ID: Apache-2.0
 * URL: https://www.apache.org/licenses/LICENSE-2.0
 * -----
 * Heavily inspired by: https://github.com/bevyengine/bevy/tree/main/examples/mobile
 */

use bevy::{prelude::*, window::AppLifecycle};

pub(super) fn plugin(app: &mut App) {
    // Only run the lifetime handler when an [`AudioSink`] component exists in the world.
    // This ensures we don't try to manage audio that hasn't been initialized yet.
    app.add_systems(
        Update,
        handle_lifetime.run_if(any_with_component::<AudioSink>),
    );
}

/// Pause audio when app goes into background and resume when it returns.
///
/// This is necessary for android
fn handle_lifetime(mut reader: MessageReader<AppLifecycle>, audio_sink: Single<&AudioSink>) {
    for app_lifecycle in reader.read() {
        match app_lifecycle {
            AppLifecycle::Suspended => audio_sink.pause(),
            AppLifecycle::Running => audio_sink.play(),
            AppLifecycle::Idle | AppLifecycle::WillSuspend | AppLifecycle::WillResume => (),
        }
    }
}
