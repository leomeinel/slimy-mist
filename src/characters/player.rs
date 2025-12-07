/*
 * File: player.rs
 * Author: Leopold Johannes Meinel (leo@meinel.dev)
 * -----
 * Copyright (c) 2025 Leopold Johannes Meinel & contributors
 * SPDX ID: Apache-2.0
 * URL: https://www.apache.org/licenses/LICENSE-2.0
 * -----
 * Heavily inspired by: https://github.com/TheBevyFlock/bevy_new_2d
 */

//! Player-specific behavior.

use bevy::{
    image::{ImageLoaderSettings, ImageSampler},
    prelude::*,
};
use bevy_enhanced_input::prelude::*;
use bevy_rapier2d::prelude::*;

use crate::{asset_tracking::LoadResource, characters::animation::PlayerAnimation};

/// Plugin
pub(super) fn plugin(app: &mut App) {
    app.add_input_context::<Player>();
    app.load_resource::<PlayerAssets>();

    app.add_observer(apply_movement);
}

#[derive(Debug, InputAction)]
#[action_output(Vec2)]
struct Movement;

#[derive(Resource, Asset, Clone, Reflect)]
#[reflect(Resource)]
pub struct PlayerAssets {
    #[dependency]
    image: Handle<Image>,
    #[dependency]
    pub steps: Vec<Handle<AudioSource>>,

    sprite_sheet: Handle<TextureAtlasLayout>,
}

impl FromWorld for PlayerAssets {
    fn from_world(world: &mut World) -> Self {
        let atlas = TextureAtlasLayout::from_grid((24, 24).into(), 9, 1, None, None);
        let mut atlases = world
            .get_resource_mut::<Assets<TextureAtlasLayout>>()
            .unwrap();
        let sprite_sheet = atlases.add(atlas);

        let assets = world.resource::<AssetServer>();
        let image: Handle<Image> = assets.load_with_settings(
            "images/characters/player/male.webp",
            |settings: &mut ImageLoaderSettings| {
                // Use `nearest` image sampling to preserve pixel art style.
                settings.sampler = ImageSampler::nearest();
            },
        );

        Self {
            image,
            sprite_sheet,
            steps: vec![assets.load("audio/sound-effects/step/stone01.ogg")],
        }
    }
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Default, Reflect)]
#[reflect(Component)]
pub(crate) struct Player;

/// The player character.
pub fn player(player_assets: &PlayerAssets) -> impl Bundle {
    let player_animation = PlayerAnimation::new();

    (
        Name::new("Player"),
        Player,
        Sprite {
            image: player_assets.image.clone(),
            texture_atlas: Some(TextureAtlas {
                layout: player_assets.sprite_sheet.clone(),
                ..default()
            }),
            ..default()
        },
        RigidBody::Dynamic,
        GravityScale(0.),
        Collider::cuboid(12., 12.),
        KinematicCharacterController::default(),
        player_animation,
        actions!(
            Player[(
                Action::<Movement>::new(),
                DeadZone::default(),
                SmoothNudge::default(),
                Scale::splat(120.),
                Bindings::spawn((
                    Cardinal::arrows(),
                    Cardinal::wasd_keys(),
                    Axial::left_stick(),
                )),
            )]
        ),
    )
}

/// Apply movement
fn apply_movement(
    movement_event: On<Fire<Movement>>,
    time: Res<Time>,
    mut controller_query: Single<&mut KinematicCharacterController, With<Player>>,
) {
    controller_query.translation = Some(movement_event.value * time.delta_secs());
}
