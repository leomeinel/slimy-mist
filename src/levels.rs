/*
 * File: levels.rs
 * Author: Leopold Johannes Meinel (leo@meinel.dev)
 * -----
 * Copyright (c) 2026 Leopold Johannes Meinel & contributors
 * SPDX ID: Apache-2.0
 * URL: https://www.apache.org/licenses/LICENSE-2.0
 */

//! Game worlds

pub(crate) mod overworld;

use bevy::{prelude::*, reflect::Reflectable};
use bevy_asset_loader::asset_collection::AssetCollection;
use bevy_prng::WyRand;
use bevy_rand::{global::GlobalRng, traits::ForkableSeed as _};

pub(super) fn plugin(app: &mut App) {
    // Add rng for levels
    app.add_systems(Startup, setup_rng);
}

/// Applies to anything that stores level assets
pub(crate) trait LevelAssets
where
    Self: AssetCollection + Resource + Default + Reflectable,
{
    fn music(&self) -> &Option<Vec<Handle<AudioSource>>>;
    fn tile_set(&self) -> &Handle<Image>;
}
#[macro_export]
macro_rules! impl_level_assets {
    ($type: ty) => {
        impl LevelAssets for $type {
            fn music(&self) -> &Option<Vec<Handle<AudioSource>>> {
                &self.music
            }
            fn tile_set(&self) -> &Handle<Image> {
                &self.tile_set
            }
        }
    };
}

/// Applies to anything that is a level
pub(crate) trait Level
where
    Self: Component + Default + Reflectable,
{
}

/// Rng for levels
#[derive(Component)]
pub(crate) struct LevelRng;

/// Spawn [`LevelRng`] by forking [`GlobalRng`]
fn setup_rng(mut global: Single<&mut WyRand, With<GlobalRng>>, mut commands: Commands) {
    commands.spawn((LevelRng, global.fork_seed()));
}
