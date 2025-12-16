/*
 * File: overworld.rs
 * Author: Leopold Johannes Meinel (leo@meinel.dev)
 * -----
 * Copyright (c) 2025 Leopold Johannes Meinel & contributors
 * SPDX ID: Apache-2.0
 * URL: https://www.apache.org/licenses/LICENSE-2.0
 */

//! Overworld-specific behavior.

use std::ops::Range;

use bevy::prelude::*;
use bevy_asset_loader::prelude::*;
use bevy_common_assets::ron::RonAssetPlugin;
use bevy_prng::WyRand;
use rand::{Rng, seq::IndexedRandom};

use crate::{
    animations::{AnimationRng, Animations},
    audio::music,
    characters::{
        CollisionData, CollisionHandle, VisualMap, collider,
        npc::{Slime, slime, slime_visual},
        player::{Player, player, player_visual},
    },
    impl_level_assets,
    levels::{
        ChunkController, DEFAULT_Z, DynamicZ, LEVEL_Z, LevelAssets, LevelRng, SHADOW_COLOR,
        SHADOW_Z, TileData, TileHandle,
    },
    logging::warn::{CHARACTER_FALLBACK_COLLISION_DATA, LEVEL_MISSING_OPTIONAL_ASSET_DATA},
    screens::Screen,
};

pub(super) fn plugin(app: &mut App) {
    // Initialize asset state
    app.init_state::<OverWorldAssetState>();

    // Add `ChunkController`
    app.insert_resource(ChunkController::<Overworld>::default());

    // Add plugin to load ron file
    app.add_plugins((RonAssetPlugin::<TileData<Overworld>>::new(&["tiles.ron"]),));

    // Setup overworld
    app.add_systems(Startup, setup_overworld);

    // Add loading states via bevy_asset_loader
    app.add_loading_state(
        LoadingState::new(OverWorldAssetState::AssetLoading)
            .continue_to_state(OverWorldAssetState::Next)
            .with_dynamic_assets_file::<StandardDynamicAssetCollection>(
                "data/levels/overworld.assets.ron",
            )
            .load_collection::<OverworldAssets>(),
    );
}

/// Asset state that tracks asset loading
#[derive(Clone, Eq, PartialEq, Debug, Hash, Default, States)]
enum OverWorldAssetState {
    #[default]
    AssetLoading,
    Next,
}

/// Assets for the overworld
#[derive(AssetCollection, Resource)]
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

/// Deserialize ron file for [`TileData`]
fn setup_overworld(mut commands: Commands, assets: Res<AssetServer>) {
    let handle = TileHandle::<Overworld>(assets.load("data/levels/overworld.tiles.ron"));
    commands.insert_resource(handle);
}

/// Level position
const LEVEL_POS: Vec3 = Vec3::new(0., 0., LEVEL_Z);

/// Slime positions
const SLIME_POSITIONS: [Vec3; 4] = [
    Vec3::new(40., 0., DEFAULT_Z),
    Vec3::new(-40., 0., DEFAULT_Z),
    Vec3::new(0., 40., DEFAULT_Z),
    Vec3::new(0., -40., DEFAULT_Z),
];
/// Slime animation delay
const SLIME_ANIMATION_DELAY: Range<f32> = 1.0..10.0;

/// Player position
const PLAYER_POS: Vec3 = Vec3::new(0., 0., DEFAULT_Z);
/// Player animation delay
const PLAYER_ANIMATION_DELAY: Range<f32> = 1.0..5.0;

/// Spawn overworld with player, enemies and objects
pub(crate) fn spawn_overworld(
    mut animation_rng: Single<&mut WyRand, (With<AnimationRng>, Without<LevelRng>)>,
    mut level_rng: Single<&mut WyRand, (With<LevelRng>, Without<AnimationRng>)>,
    mut commands: Commands,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut visual_map: ResMut<VisualMap>,
    level_assets: Res<OverworldAssets>,
    player_animations: Res<Animations<Player>>,
    player_data: Res<Assets<CollisionData<Player>>>,
    player_handle: Res<CollisionHandle<Player>>,
    slime_animations: Res<Animations<Slime>>,
    slime_data: Res<Assets<CollisionData<Slime>>>,
    slime_handle: Res<CollisionHandle<Slime>>,
) {
    // Get data from `CollisionData` with `CollisionHandle`
    let slime_data = slime_data.get(slime_handle.0.id()).unwrap();
    let slime_width = slime_data.width.unwrap_or_else(|| {
        warn_once!("{}", CHARACTER_FALLBACK_COLLISION_DATA);
        8.
    });
    let player_data = player_data.get(player_handle.0.id()).unwrap();
    let player_width = slime_data.width.unwrap_or_else(|| {
        warn_once!("{}", CHARACTER_FALLBACK_COLLISION_DATA);
        9.
    });

    let level = commands
        .spawn((
            Name::new("Level"),
            Overworld,
            Transform::from_translation(LEVEL_POS),
            DespawnOnExit(Screen::Gameplay),
            Visibility::default(),
        ))
        .id();

    if let Some(level_music) = level_assets
        .get_music()
        .clone()
        .unwrap_or_else(|| {
            warn_once!("{}", LEVEL_MISSING_OPTIONAL_ASSET_DATA);
            Vec::default()
        })
        .choose(level_rng.as_mut())
        .cloned()
    {
        commands.entity(level).with_children(|commands| {
            commands.spawn((Name::new("Gameplay Music"), music(level_music)));
        });
    }

    for pos in SLIME_POSITIONS {
        commands.entity(level).with_children(|commands_p| {
            let slime = commands_p
                .spawn((
                    Visibility::Inherited,
                    DynamicZ(DEFAULT_Z),
                    Transform::from_translation(pos),
                    collider::<Slime>(slime_data),
                    slime(),
                ))
                .id();
            commands_p
                .commands()
                .entity(slime)
                .with_children(|commands_c| {
                    let slime_visual = commands_c
                        .spawn((
                            DynamicZ(DEFAULT_Z),
                            slime_visual(
                                &slime_animations,
                                animation_rng.random_range(SLIME_ANIMATION_DELAY),
                            ),
                        ))
                        .id();
                    visual_map.0.insert(slime, slime_visual);
                });
            commands_p
                .commands()
                .entity(slime)
                .with_children(|commands_c| {
                    commands_c.spawn((
                        DynamicZ(SHADOW_Z),
                        Transform::from_xyz(0., -slime_width / 2., SHADOW_Z),
                        Mesh2d(meshes.add(Circle::new(-slime_width / 4.))),
                        MeshMaterial2d(materials.add(Color::from(SHADOW_COLOR.with_alpha(0.25)))),
                    ));
                });
        });
    }

    commands.entity(level).with_children(|commands_p| {
        let player = commands_p
            .spawn((
                DynamicZ(DEFAULT_Z),
                Visibility::Inherited,
                Transform::from_translation(PLAYER_POS),
                collider::<Player>(player_data),
                player(),
            ))
            .id();
        commands_p
            .commands()
            .entity(player)
            .with_children(|commands_c| {
                let player_visual = commands_c
                    .spawn((
                        DynamicZ(DEFAULT_Z),
                        player_visual(
                            &player_animations,
                            animation_rng.random_range(PLAYER_ANIMATION_DELAY),
                        ),
                    ))
                    .id();
                visual_map.0.insert(player, player_visual);
            });
        commands_p
            .commands()
            .entity(player)
            .with_children(|commands_c| {
                commands_c.spawn((
                    DynamicZ(SHADOW_Z),
                    Transform::from_xyz(0., -player_width / 2., SHADOW_Z),
                    Mesh2d(meshes.add(Circle::new(player_width / 4.))),
                    MeshMaterial2d(materials.add(Color::from(SHADOW_COLOR.with_alpha(0.25)))),
                ));
            });
    });
}
