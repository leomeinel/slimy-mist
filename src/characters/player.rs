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
use bevy_spritesheet_animation::prelude::*;

use crate::{
    characters::{CharacterAssets, animations::Animations},
    impl_character_assets,
};

pub(super) fn plugin(app: &mut App) {
    // Initialize asset state
    app.init_state::<PlayerAssetState>();

    // Add loading states via bevy_asset_loader
    app.add_loading_state(
        LoadingState::new(PlayerAssetState::AssetLoading)
            .continue_to_state(PlayerAssetState::Next)
            .with_dynamic_assets_file::<StandardDynamicAssetCollection>(
                "data/characters/player/male.assets.ron",
            )
            .load_collection::<PlayerAssets>(),
    );

    // Handle bevy_enhanced_input with input context and observers
    // FIXME: This currently can not be paused
    app.add_input_context::<Player>();
    app.add_observer(apply_movement);
    app.add_observer(stop_movement);
}

/// Asset state that tracks asset loading
#[derive(Clone, Eq, PartialEq, Debug, Hash, Default, States)]
pub(crate) enum PlayerAssetState {
    #[default]
    AssetLoading,
    Next,
}

/// Assets that are serialized from a ron file
#[derive(AssetCollection, Resource)]
pub(crate) struct PlayerAssets {
    #[asset(key = "male.step_sounds", collection(typed))]
    pub(crate) step_sounds: Vec<Handle<AudioSource>>,

    #[asset(key = "male.image")]
    pub(crate) image: Handle<Image>,
}
impl_character_assets!(PlayerAssets);

/// Player marker
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Default, Reflect)]
#[reflect(Component)]
pub(crate) struct Player;

/// Movement marker
#[derive(Debug, InputAction)]
#[action_output(Vec2)]
struct Movement;

/// The player character.
pub(crate) fn player(animations: &Res<Animations<Player>>) -> impl Bundle {
    (
        Name::new("Player"),
        Player,
        animations.sprite.clone(),
        SpritesheetAnimation::new(animations.idle.clone()),
        RigidBody::Dynamic,
        GravityScale(0.),
        Collider::cuboid(12., 12.),
        KinematicCharacterController::default(),
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

/// On a fired movement, set translation to the given input
fn apply_movement(
    event: On<Fire<Movement>>,
    time: Res<Time>,
    mut controller: Single<&mut KinematicCharacterController, With<Player>>,
) {
    controller.translation = Some(event.value * time.delta_secs());
}

/// On a completed movement, set translation to zero
fn stop_movement(
    _: On<Complete<Movement>>,
    mut controller: Single<&mut KinematicCharacterController, With<Player>>,
) {
    controller.translation = Some(Vec2::ZERO);
}
