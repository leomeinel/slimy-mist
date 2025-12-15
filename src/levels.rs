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

use std::marker::PhantomData;

use bevy::{
    color::palettes::tailwind, platform::collections::HashSet, prelude::*, reflect::Reflectable,
};
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

/// Color for cast shadows
pub(crate) const SHADOW_COLOR: Srgba = tailwind::GRAY_700;

/// Z-level for the level
pub(crate) const LEVEL_Z: f32 = 1.;
/// Z-level for shadows
pub(crate) const SHADOW_Z: f32 = 9.;
/// Z-level for any foreground object
pub(crate) const DEFAULT_Z: f32 = 10.;

/// Applies to anything that stores level assets
pub(crate) trait LevelAssets {
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

/// Animation data deserialized from a ron file as a generic
#[derive(serde::Deserialize, Asset, TypePath, Default)]
pub(crate) struct TileData<T>
where
    T: Reflectable,
{
    atlas_columns: usize,
    atlas_rows: usize,
    #[serde(default)]
    full_dirt_tiles: Option<HashSet<(u32, u32)>>,
    #[serde(default)]
    full_grass_tiles: Option<HashSet<(u32, u32)>>,
    #[serde(default)]
    corner_outer_grass_to_dirt_tiles: Option<HashSet<(u32, u32)>>,
    #[serde(default)]
    corner_outer_dirt_to_grass_tiles: Option<HashSet<(u32, u32)>>,
    #[serde(default)]
    side_dirt_and_grass_tiles: Option<HashSet<(u32, u32)>>,
    #[serde(default)]
    diag_stripe_grass_in_dirt_tiles: Option<HashSet<(u32, u32)>>,
    #[serde(skip)]
    _phantom: PhantomData<T>,
}

impl<T> TileData<T>
where
    T: Reflectable,
{
    fn get_tiles(
        &self,
    ) -> Option<(
        HashSet<(u32, u32)>,
        HashSet<(u32, u32)>,
        HashSet<(u32, u32)>,
        HashSet<(u32, u32)>,
        HashSet<(u32, u32)>,
        HashSet<(u32, u32)>,
    )> {
        Some((
            self.full_dirt_tiles.as_ref().cloned()?,
            self.full_grass_tiles.as_ref().cloned()?,
            self.corner_outer_grass_to_dirt_tiles.as_ref().cloned()?,
            self.corner_outer_dirt_to_grass_tiles.as_ref().cloned()?,
            self.side_dirt_and_grass_tiles.as_ref().cloned()?,
            self.diag_stripe_grass_in_dirt_tiles.as_ref().cloned()?,
        ))
    }
}

/// Handle for [`CollisionData`] as a generic
#[derive(Resource)]
pub(crate) struct TileHandle<T>(pub(crate) Handle<TileData<T>>)
where
    T: Reflectable;

/// Sorts entities by their y position.
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
