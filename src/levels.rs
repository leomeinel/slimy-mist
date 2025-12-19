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

    // Sort entities with `YSort` by Y
    app.add_systems(PostUpdate, y_sort);
}

/// Z-level for the level
pub(crate) const LEVEL_Z: f32 = 1.;
/// Z-level for shadows
pub(crate) const SHADOW_Z: f32 = 9.;
/// Z-level for any foreground object
pub(crate) const DEFAULT_Z: f32 = 10.;

/// Factor for [`y_sort`]
///
/// This is required to ensure that we stay within the default Z-levels supported by bevy's camera.
pub(crate) const Y_SORT_FACTOR: f32 = 1e-5;

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

/// Sorts entities by their y position.
#[derive(Component, Default, Reflect)]
#[reflect(Component)]
pub(crate) struct YSort(pub(crate) f32);

/// Applies an offset to the [`YSort`].
///
/// The offset is expected to be in px.
#[derive(Component, Default, Reflect)]
#[reflect(Component)]
pub(crate) struct YSortOffset(pub(crate) f32);

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
fn y_sort(mut query: Query<(&mut Transform, &YSort, Option<&YSortOffset>)>) {
    for (mut transform, sort, sort_offset) in query.iter_mut() {
        transform.translation.z = (sort.0
            + sort_offset.map_or(0., |offset| offset.0) * Y_SORT_FACTOR)
            - transform.translation.y * Y_SORT_FACTOR;
    }
}
