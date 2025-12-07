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
use bevy_rapier2d::prelude::*;

use crate::{
    asset_tracking::LoadResource,
    audio::music,
    characters::player::{PlayerAssets, player},
    screens::Screen,
};

/// Plugin
pub(super) fn plugin(app: &mut App) {
    app.load_resource::<LevelAssets>();
}

#[derive(Resource, Asset, Clone, Reflect)]
#[reflect(Resource)]
pub struct LevelAssets {
    #[dependency]
    music: Handle<AudioSource>,
}

impl FromWorld for LevelAssets {
    fn from_world(world: &mut World) -> Self {
        let assets = world.resource::<AssetServer>();
        Self {
            music: assets.load("audio/music/bit-bit-loop.ogg"),
        }
    }
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
                Transform::from_xyz(320. + 10., 0., 3.),
                Some(Transform::from_rotation(Quat::from_rotation_z(
                    std::f32::consts::PI / 2.0
                ))),
                &mut meshes,
                &mut materials
            ),
            border(
                Transform::from_xyz(-320. - 10., 0., 3.),
                Some(Transform::from_rotation(Quat::from_rotation_z(
                    std::f32::consts::PI / 2.0
                ))),
                &mut meshes,
                &mut materials
            ),
            border(
                Transform::from_xyz(0., 320. + 10., 3.),
                None,
                &mut meshes,
                &mut materials
            ),
            border(
                Transform::from_xyz(2., -320. - 10., 3.),
                None,
                &mut meshes,
                &mut materials
            ),
        ],
    ));
}

fn border(
    position: Transform,
    rotation: Option<Transform>,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<ColorMaterial>>,
) -> impl Bundle {
    (
        RigidBody::Fixed,
        Collider::cuboid(340.0, 10.0),
        position,
        rotation.unwrap_or_default(),
        Mesh2d(meshes.add(Rectangle::new(680., 20.))),
        MeshMaterial2d(materials.add(Into::<Color>::into(BORDER_COLOR))),
        Visibility::default(),
        DespawnOnExit(Screen::Gameplay),
    )
}
