/*
 * File: audio.rs
 * Author: Leopold Johannes Meinel (leo@meinel.dev)
 * -----
 * Copyright (c) 2025 Leopold Johannes Meinel & contributors
 * SPDX ID: Apache-2.0
 * URL: https://www.apache.org/licenses/LICENSE-2.0
 * -----
 * Heavily inspired by: https://github.com/TheBevyFlock/bevy_new_2d
 */

use bevy::{
    audio::{PlaybackMode, Volume},
    prelude::*,
};

pub(super) fn plugin(app: &mut App) {
    // Apply global volume if it is changed
    app.add_systems(
        Update,
        apply_global_volume.run_if(resource_changed::<GlobalVolume>),
    );
}

/// An organizational marker component that should be added to a spawned [`AudioPlayer`] if it's in the
/// general "music" category (e.g. global background music, soundtrack).
///
/// This can then be used to query for and operate on sounds in that category.
#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub(crate) struct Music;

/// An organizational marker component that should be added to a spawned [`AudioPlayer`] if it's in the
/// general "sound effect" category (e.g. footsteps, the sound of a magic spell, a door opening).
///
/// This can then be used to query for and operate on sounds in that category.
#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub(crate) struct SoundEffect;

/// A music audio instance.
pub(crate) fn music(handle: Handle<AudioSource>) -> impl Bundle {
    (
        AudioPlayer(handle),
        PlaybackSettings {
            mode: PlaybackMode::Loop,
            volume: Volume::Linear(0.15),
            ..default()
        },
        Music,
    )
}

/// A sound effect audio instance.
pub(crate) fn sound_effect(handle: Handle<AudioSource>) -> impl Bundle {
    (AudioPlayer(handle), PlaybackSettings::DESPAWN, SoundEffect)
}

/// [`GlobalVolume`] doesn't apply to already-running audio entities, so this system will update them.
fn apply_global_volume(
    mut query: Query<(&PlaybackSettings, &mut AudioSink)>,
    global_volume: Res<GlobalVolume>,
) {
    for (playback, mut sink) in &mut query {
        sink.set_volume(global_volume.volume * playback.volume);
    }
}
