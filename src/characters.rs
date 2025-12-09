/*
 * File: characters.rs
 * Author: Leopold Johannes Meinel (leo@meinel.dev)
 * -----
 * Copyright (c) 2025 Leopold Johannes Meinel & contributors
 * SPDX ID: Apache-2.0
 * URL: https://www.apache.org/licenses/LICENSE-2.0
 */

//! Characters

pub(crate) mod animations;
pub(crate) mod npc;
pub(crate) mod player;

use bevy::prelude::*;

pub(super) fn plugin(app: &mut App) {
    // Add child plugins
    app.add_plugins((animations::plugin, npc::plugin, player::plugin));
}

/// Applies to anything that stores character assets
pub(crate) trait CharacterAssets {
    fn get_step_sounds(&self) -> &Vec<Handle<AudioSource>>;
    fn get_image(&self) -> &Handle<Image>;
}
#[macro_export]
macro_rules! impl_character_assets {
    ($type: ty) => {
        impl CharacterAssets for $type {
            fn get_step_sounds(&self) -> &Vec<Handle<AudioSource>> {
                &self.step_sounds
            }
            fn get_image(&self) -> &Handle<Image> {
                &self.image
            }
        }
    };
}
