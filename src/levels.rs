/*
 * File: levels.rs
 * Author: Leopold Johannes Meinel (leo@meinel.dev)
 * -----
 * Copyright (c) 2025 Leopold Johannes Meinel & contributors
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

    // Add child plugins
    app.add_plugins(overworld::plugin);

    // Sort entities with `DynamicZ` by Y
    app.add_systems(PostUpdate, sort_by_y);
}

/// Z-level for the level
pub(crate) const LEVEL_Z: f32 = 1.;
/// Z-level for shadows
pub(crate) const SHADOW_Z: f32 = 9.;
/// Z-level for any foreground object
pub(crate) const DEFAULT_Z: f32 = 10.;

/// Render distance in chunks
pub(crate) const RENDER_DISTANCE: i32 = 3;

/// Applies to anything that stores level assets
pub(crate) trait LevelAssets
where
    Self: AssetCollection + Resource + Default + Reflectable,
{
    fn get_music(&self) -> &Option<Vec<Handle<AudioSource>>>;
    fn get_tile_set(&self) -> &Handle<Image>;
}
#[macro_export]
macro_rules! impl_level_assets {
    ($type: ty) => {
        impl LevelAssets for $type {
            fn get_music(&self) -> &Option<Vec<Handle<AudioSource>>> {
                &self.music
            }
            fn get_tile_set(&self) -> &Handle<Image> {
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

// FIXME: Jumping currently also triggers this. Maybe somehow decouple from visual or find better solution.
/// Sorts entities by their y position.
///
/// Takes in a base value usually the sprite default Z with possibly an height offset.
/// this value could be tweaked to implement virtual Z for jumping
#[derive(Component, Default, Reflect)]
#[reflect(Component)]
pub(crate) struct DynamicZ(pub(crate) f32);

/// Rng for levels
#[derive(Component)]
pub(crate) struct LevelRng;

/// Spawn [`LevelRng`] by forking [`GlobalRng`]
fn setup_rng(mut global: Single<&mut WyRand, With<GlobalRng>>, mut commands: Commands) {
    commands.spawn((LevelRng, global.fork_seed()));
}

/// Applies the y-sorting to the entities Z position.
///
/// Heavily inspired by: <https://github.com/fishfolk/punchy>
fn sort_by_y(mut query: Query<(&mut Transform, &DynamicZ)>) {
    for (mut transform, z_order) in query.iter_mut() {
        transform.translation.z = z_order.0 - (transform.translation.y * 0.00001);
    }
}
