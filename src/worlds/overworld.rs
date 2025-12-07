/*
 * File: overworld.rs
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
    asset_tracking::AssetStates,
    audio::music,
    characters::player::{PlayerAssets, player},
    screens::Screen,
};

/// Plugin
pub(super) fn plugin(app: &mut App) {
    app.add_loading_state(
        LoadingState::new(AssetStates::AssetLoading)
            .continue_to_state(AssetStates::Next)
            .load_collection::<LevelAssets>(),
    );
}

#[derive(AssetCollection, Resource)]
pub struct LevelAssets {
    #[asset(path = "audio/music/bit-bit-loop.ogg")]
    music: Handle<AudioSource>,
}

// rgb(107, 114, 128)
const GROUND_COLOR: Srgba = tailwind::GRAY_500;
// rgb(17, 24, 39)
const BORDER_COLOR: Srgba = tailwind::GRAY_900;

pub(crate) fn spawn_level(
    mut commands: Commands,
    level_assets: Res<LevelAssets>,
    player_assets: Res<PlayerAssets>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    commands.spawn((
        Name::new("Level"),
        Mesh2d(meshes.add(Rectangle::new(640., 640.))),
        MeshMaterial2d(materials.add(Into::<Color>::into(GROUND_COLOR))),
        Transform::from_xyz(0., 0., 2.),
        Visibility::default(),
        DespawnOnExit(Screen::Gameplay),
        children![
            player(&player_assets),
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
