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
use bevy_common_assets::ron::RonAssetPlugin;
use bevy_enhanced_input::prelude::*;
use bevy_rapier2d::prelude::*;
use bevy_spritesheet_animation::prelude::*;

use crate::{
    Pause,
    characters::{
        CharacterAssets, CollisionData, CollisionHandle, animations::Animations, collider,
    },
    impl_character_assets,
};

pub(super) fn plugin(app: &mut App) {
    // Initialize asset state
    app.init_state::<PlayerAssetState>();

    // Add plugin to load ron file
    app.add_plugins((RonAssetPlugin::<CollisionData<Player>>::new(&[
        "animation.ron",
    ]),));

    // Add enhanced input plugin
    app.add_plugins(EnhancedInputPlugin);

    // Add loading states via bevy_asset_loader
    app.add_loading_state(
        LoadingState::new(PlayerAssetState::AssetLoading)
            .continue_to_state(PlayerAssetState::Next)
            .with_dynamic_assets_file::<StandardDynamicAssetCollection>(
                "data/characters/player/male.assets.ron",
            )
            .load_collection::<PlayerAssets>(),
    );

    // Setup player
    app.add_systems(Startup, setup_player);

    // Handle bevy_enhanced_input with input context and observers
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

/// Deserialize ron file for [`CollisionData`]
fn setup_player(mut commands: Commands, assets: Res<AssetServer>) {
    let animation_handle =
        CollisionHandle::<Player>(assets.load("data/characters/player/male.collision.ron"));
    commands.insert_resource(animation_handle);
}

/// The player character.
pub(crate) fn player(
    animations: &Res<Animations<Player>>,
    collision_data: &Res<Assets<CollisionData<Player>>>,
    collision_handle: &Res<CollisionHandle<Player>>,
) -> impl Bundle {
    (
        Name::new("Player"),
        Player,
        animations.sprite.clone(),
        SpritesheetAnimation::new(animations.idle.clone()),
        RigidBody::Dynamic,
        GravityScale(0.),
        collider::<Player>(collision_data, collision_handle),
        KinematicCharacterController::default(),
        LockedAxes::ROTATION_LOCKED,
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

/// Multiplier for the walk speed of the player
const WALK_SPEED: f32 = 0.8;

/// On a fired movement, set translation to the given input
fn apply_movement(
    event: On<Fire<Movement>>,
    pause: Res<State<Pause>>,
    time: Res<Time>,
    mut controller: Single<&mut KinematicCharacterController, With<Player>>,
) {
    // Return if game is paused
    if pause.get().0 {
        return;
    }

    // Apply movement
    controller.translation = Some(event.value * WALK_SPEED * time.delta_secs());
}

/// On a completed movement, set translation to zero
fn stop_movement(
    _: On<Complete<Movement>>,
    mut controller: Single<&mut KinematicCharacterController, With<Player>>,
) {
    controller.translation = Some(Vec2::ZERO);
}
