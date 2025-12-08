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

use bevy::prelude::*;
use bevy_asset_loader::prelude::*;
use bevy_enhanced_input::prelude::*;
use bevy_rapier2d::prelude::*;

use crate::{asset_tracking::AssetStates, characters::animation::PlayerAnimation};

/// Plugin
pub(super) fn plugin(app: &mut App) {
    app.add_loading_state(
        LoadingState::new(AssetStates::AssetLoading)
            .continue_to_state(AssetStates::Next)
            .load_collection::<PlayerAssets>(),
    );

    app.add_input_context::<Player>();
    app.add_observer(apply_movement);
    app.add_observer(stop_movement);
}

#[derive(Debug, InputAction)]
#[action_output(Vec2)]
struct Movement;

#[derive(AssetCollection, Resource)]
pub struct PlayerAssets {
    #[asset(
        paths(
            "audio/sound-effects/movement/player-step-hard0.ogg",
            "audio/sound-effects/movement/player-step-hard1.ogg",
            "audio/sound-effects/movement/player-step-hard2.ogg"
        ),
        collection(typed)
    )]
    pub(crate) steps: Vec<Handle<AudioSource>>,

    #[asset(texture_atlas_layout(tile_size_x = 24, tile_size_y = 24, columns = 9, rows = 1))]
    pub(crate) sprite_sheet: Handle<TextureAtlasLayout>,
    #[asset(image(sampler(filter = nearest)))]
    #[asset(path = "images/characters/player/male.webp")]
    pub(crate) image: Handle<Image>,
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
        Sprite::from_atlas_image(
            player_assets.image.clone(),
            TextureAtlas::from(player_assets.sprite_sheet.clone()),
        ),
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

/// Stop movement
fn stop_movement(
    _movement_event: On<Complete<Movement>>,
    mut controller_query: Single<&mut KinematicCharacterController, With<Player>>,
) {
    controller_query.translation = Some(Vec2::ZERO);
}
