/*
 * File: arena.rs
 * Author: Leopold Johannes Meinel (leo@meinel.dev)
 * -----
 * Copyright (c) 2025 Leopold Johannes Meinel & contributors
 * SPDX ID: Apache-2.0
 * URL: https://www.apache.org/licenses/LICENSE-2.0
 */

//! Overworld-specific behavior.

use bevy::{color::palettes::tailwind, prelude::*};
use bevy_asset_loader::prelude::*;
use bevy_prng::WyRand;
use bevy_rapier2d::prelude::*;
use rand::Rng;

use crate::{
    animations::{AnimationRng, Animations},
    audio::music,
    characters::{
        CollisionData, CollisionHandle,
        npc::{Slime, slime},
        player::{Player, player},
    },
    screens::Screen,
};

pub(super) fn plugin(app: &mut App) {
    // Initialize asset state
    app.init_state::<ArenaAssetState>();

    // Add loading states via bevy_asset_loader
    app.add_loading_state(
        LoadingState::new(ArenaAssetState::AssetLoading)
            .continue_to_state(ArenaAssetState::Next)
            .load_collection::<ArenaAssets>(),
    );
}

/// Asset state that tracks asset loading
#[derive(Clone, Eq, PartialEq, Debug, Hash, Default, States)]
enum ArenaAssetState {
    #[default]
    AssetLoading,
    Next,
}

/// Assets for the arena
#[derive(AssetCollection, Resource)]
pub(crate) struct ArenaAssets {
    #[asset(path = "audio/music/bit-bit-loop.ogg")]
    music: Handle<AudioSource>,
}

/// rgb(107, 114, 128)
const GROUND_COLOR: Srgba = tailwind::GRAY_500;
/// Width and height of the ground
const GROUND_WIDTH_HEIGHT: f32 = 640.;

/// rgb(17, 24, 39)
const BORDER_COLOR: Srgba = tailwind::GRAY_900;
/// Height of the border
const BORDER_HEIGHT: f32 = 20.;

/// Spawn arena with player, enemies and objects
pub(crate) fn spawn_arena(
    mut animation_rng: Single<&mut WyRand, With<AnimationRng>>,
    mut commands: Commands,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
    level_assets: Res<ArenaAssets>,
    player_animations: Res<Animations<Player>>,
    player_collision_data: Res<Assets<CollisionData<Player>>>,
    player_collision_handle: Res<CollisionHandle<Player>>,
    slime_animations: Res<Animations<Slime>>,
    slime_collision_data: Res<Assets<CollisionData<Slime>>>,
    slime_collision_handle: Res<CollisionHandle<Slime>>,
) {
    commands.spawn((
        Name::new("Level"),
        Mesh2d(meshes.add(Rectangle::new(GROUND_WIDTH_HEIGHT, GROUND_WIDTH_HEIGHT))),
        MeshMaterial2d(materials.add(Into::<Color>::into(GROUND_COLOR))),
        Transform::from_xyz(0., 0., 2.),
        Visibility::default(),
        DespawnOnExit(Screen::Gameplay),
        children![
            player(
                &player_animations,
                &player_collision_data,
                &player_collision_handle,
                animation_rng.random_range(1.0f32..5.0f32),
            ),
            (
                Transform::from_xyz(40., 0., 3.),
                slime(
                    &slime_animations,
                    &slime_collision_data,
                    &slime_collision_handle,
                    animation_rng.random_range(2.0f32..10.0f32),
                ),
            ),
            (
                Transform::from_xyz(-40., 0., 3.),
                slime(
                    &slime_animations,
                    &slime_collision_data,
                    &slime_collision_handle,
                    animation_rng.random_range(2.0f32..10.0f32),
                ),
            ),
            (
                Transform::from_xyz(0., 40., 3.),
                slime(
                    &slime_animations,
                    &slime_collision_data,
                    &slime_collision_handle,
                    animation_rng.random_range(2.0f32..10.0f32),
                ),
            ),
            (
                Transform::from_xyz(0., -40., 3.),
                slime(
                    &slime_animations,
                    &slime_collision_data,
                    &slime_collision_handle,
                    animation_rng.random_range(2.0f32..10.0f32),
                ),
            ),
            border(
                Transform {
                    translation: Vec3::new(GROUND_WIDTH_HEIGHT / 2. + BORDER_HEIGHT / 2., 0., 3.),
                    rotation: Quat::from_rotation_z(std::f32::consts::PI / 2.0),
                    ..default()
                },
                &mut meshes,
                &mut materials
            ),
            border(
                Transform {
                    translation: Vec3::new(-GROUND_WIDTH_HEIGHT / 2. - BORDER_HEIGHT / 2., 0., 3.),
                    rotation: Quat::from_rotation_z(std::f32::consts::PI / 2.0),
                    ..default()
                },
                &mut meshes,
                &mut materials
            ),
            border(
                Transform {
                    translation: Vec3::new(0., GROUND_WIDTH_HEIGHT / 2. + BORDER_HEIGHT / 2., 3.),
                    ..default()
                },
                &mut meshes,
                &mut materials
            ),
            border(
                Transform {
                    translation: Vec3::new(0., -GROUND_WIDTH_HEIGHT / 2. - BORDER_HEIGHT / 2., 3.),
                    ..default()
                },
                &mut meshes,
                &mut materials
            ),
            (
                Name::new("Gameplay Music"),
                music(level_assets.music.clone())
            ),
        ],
    ));
}

/// Border for the arena
fn border(
    transform: Transform,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<ColorMaterial>>,
) -> impl Bundle {
    (
        RigidBody::Fixed,
        Collider::cuboid(
            (GROUND_WIDTH_HEIGHT + BORDER_HEIGHT * 2.) / 2.,
            BORDER_HEIGHT / 2.,
        ),
        transform,
        Mesh2d(meshes.add(Rectangle::new(
            GROUND_WIDTH_HEIGHT + BORDER_HEIGHT * 2.,
            BORDER_HEIGHT,
        ))),
        MeshMaterial2d(materials.add(Into::<Color>::into(BORDER_COLOR))),
        Visibility::default(),
        DespawnOnExit(Screen::Gameplay),
    )
}
