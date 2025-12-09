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
use bevy_rapier2d::prelude::*;

use crate::{
    audio::music,
    characters::{
        animations::Animations,
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

// rgb(107, 114, 128)
const GROUND_COLOR: Srgba = tailwind::GRAY_500;
// rgb(17, 24, 39)
const BORDER_COLOR: Srgba = tailwind::GRAY_900;

/// Spawn arena with player, enemies and objects
pub(crate) fn spawn_arena(
    mut commands: Commands,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
    level_assets: Res<ArenaAssets>,
    player_animations: Res<Animations<Player>>,
    slime_animations: Res<Animations<Slime>>,
) {
    commands.spawn((
        Name::new("Level"),
        Mesh2d(meshes.add(Rectangle::new(640., 640.))),
        MeshMaterial2d(materials.add(Into::<Color>::into(GROUND_COLOR))),
        Transform::from_xyz(0., 0., 2.),
        Visibility::default(),
        DespawnOnExit(Screen::Gameplay),
        children![
            player(&player_animations),
            slime(&slime_animations),
            (
                Name::new("Gameplay Music"),
                music(level_assets.music.clone())
            ),
            border(
                Transform {
                    translation: Vec3::new(320. + 10., 0., 3.),
                    rotation: Quat::from_rotation_z(std::f32::consts::PI / 2.0),
                    ..default()
                },
                &mut meshes,
                &mut materials
            ),
            border(
                Transform {
                    translation: Vec3::new(-320. - 10., 0., 3.),
                    rotation: Quat::from_rotation_z(std::f32::consts::PI / 2.0),
                    ..default()
                },
                &mut meshes,
                &mut materials
            ),
            border(
                Transform {
                    translation: Vec3::new(0., 320. + 10., 3.),
                    ..default()
                },
                &mut meshes,
                &mut materials
            ),
            border(
                Transform {
                    translation: Vec3::new(0., -320. - 10., 3.),
                    ..default()
                },
                &mut meshes,
                &mut materials
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
        Collider::cuboid(340.0, 10.0),
        transform,
        Mesh2d(meshes.add(Rectangle::new(680., 20.))),
        MeshMaterial2d(materials.add(Into::<Color>::into(BORDER_COLOR))),
        Visibility::default(),
        DespawnOnExit(Screen::Gameplay),
    )
}
