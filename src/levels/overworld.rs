/*
 * File: overworld.rs
 * Author: Leopold Johannes Meinel (leo@meinel.dev)
 * -----
 * Copyright (c) 2025 Leopold Johannes Meinel & contributors
 * SPDX ID: Apache-2.0
 * URL: https://www.apache.org/licenses/LICENSE-2.0
 */

//! Overworld-specific behavior.

use bevy::prelude::*;
use bevy_asset_loader::prelude::*;
use bevy_prng::WyRand;
use rand::{Rng as _, seq::IndexedRandom};

use crate::{
    audio::music,
    characters::{
        Character as _, CollisionData, CollisionHandle, Shadow, VisualMap,
        animations::{ANIMATION_DELAY_RANGE, AnimationRng, Animations},
        npc::Slime,
        player::Player,
    },
    impl_level_assets,
    levels::{DEFAULT_Z, LEVEL_Z, Level, LevelAssets, LevelRng},
    logging::{error::ERR_LOADING_COLLISION_DATA, warn::WARN_INCOMPLETE_ASSET_DATA},
    procgen::{ProcGenController, ProcGenerated},
    screens::Screen,
};

pub(super) fn plugin(app: &mut App) {
    // Add controllers for procedural generation
    app.insert_resource(ProcGenController::<OverworldProcGen>::default());
    app.insert_resource(ProcGenController::<Slime>::default());
}

/// Assets for the overworld
#[derive(AssetCollection, Resource, Default, Reflect)]
pub(crate) struct OverworldAssets {
    #[asset(key = "overworld.music", collection(typed), optional)]
    music: Option<Vec<Handle<AudioSource>>>,

    #[asset(key = "overworld.tile_set")]
    pub(crate) tile_set: Handle<Image>,
}
impl_level_assets!(OverworldAssets);

/// Overworld marker
#[derive(Component, Default, Reflect)]
pub(crate) struct Overworld;
impl Level for Overworld {}

/// Marker component for overworld procedural generation
#[derive(Component, Default, Reflect)]
pub(crate) struct OverworldProcGen;
impl ProcGenerated for OverworldProcGen {}

/// Level position
const LEVEL_POS: Vec3 = Vec3::new(0., 0., LEVEL_Z);

/// Player position
const PLAYER_POS: Vec3 = Vec3::new(0., 0., DEFAULT_Z);

/// Spawn overworld with player, enemies and objects
pub(crate) fn spawn_overworld(
    mut animation_rng: Single<&mut WyRand, (With<AnimationRng>, Without<LevelRng>)>,
    mut level_rng: Single<&mut WyRand, (With<LevelRng>, Without<AnimationRng>)>,
    mut commands: Commands,
    mut visual_map: ResMut<VisualMap>,
    animations: Res<Animations<Player>>,
    data: Res<Assets<CollisionData<Player>>>,
    handle: Res<CollisionHandle<Player>>,
    level_assets: Res<OverworldAssets>,
    shadow: Res<Shadow<Player>>,
) {
    // Get data from `CollisionData` with `CollisionHandle`
    let data = data.get(handle.0.id()).expect(ERR_LOADING_COLLISION_DATA);
    let data = (data.shape.clone(), data.width, data.height);

    let level = commands
        .spawn((
            Name::new("Level"),
            Overworld,
            Transform::from_translation(LEVEL_POS),
            DespawnOnExit(Screen::Gameplay),
            Visibility::default(),
        ))
        .id();

    // Spawn music
    if let Some(level_music) = level_assets
        .get_music()
        .clone()
        .unwrap_or_else(|| {
            warn_once!("{}", WARN_INCOMPLETE_ASSET_DATA);
            Vec::default()
        })
        .choose(level_rng.as_mut())
        .cloned()
    {
        commands.entity(level).with_children(|commands| {
            commands.spawn((Name::new("Gameplay Music"), music(level_music)));
        });
    }

    // Spawn player
    let player = Player::spawn(
        &mut commands,
        &mut visual_map,
        &data,
        PLAYER_POS,
        &animations,
        &shadow,
        animation_rng.random_range(ANIMATION_DELAY_RANGE),
    );
    commands.entity(level).add_child(player);
}
