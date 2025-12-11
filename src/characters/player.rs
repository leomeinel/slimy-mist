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
    AppSystems, PausableSystems, Pause,
    animations::{AnimationController, AnimationState, AnimationTimer, Animations},
    characters::{
        CharacterAssets, CollisionData, CollisionHandle, JumpTimer, collider, tick_jump_timer,
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

    app.add_systems(
        Update,
        stop_jump
            .after(tick_jump_timer)
            .in_set(AppSystems::Update)
            .in_set(PausableSystems),
    );

    // Handle bevy_enhanced_input with input context and observers
    app.add_input_context::<Player>();
    app.add_observer(apply_movement);
    app.add_observer(stop_movement);
    app.add_observer(apply_jump);
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

/// Jump marker
#[derive(Debug, InputAction)]
#[action_output(bool)]
struct Jump;

/// Deserialize ron file for [`CollisionData`]
fn setup_player(mut commands: Commands, assets: Res<AssetServer>) {
    let animation_handle =
        CollisionHandle::<Player>(assets.load("data/characters/player/male.collision.ron"));
    commands.insert_resource(animation_handle);
}

/// Movement velocity of the player
const MOVEMENT_VELOCITY: f32 = 100.;

/// The player character.
pub(crate) fn player(
    animations: &Res<Animations<Player>>,
    collision_data: &Res<Assets<CollisionData<Player>>>,
    collision_handle: &Res<CollisionHandle<Player>>,
    animation_delay: f32,
) -> impl Bundle {
    (
        Name::new("Player"),
        Player,
        animations.sprite.clone(),
        SpritesheetAnimation::new(animations.idle.clone()),
        RigidBody::KinematicVelocityBased,
        GravityScale(0.),
        collider::<Player>(collision_data, collision_handle),
        KinematicCharacterController::default(),
        LockedAxes::ROTATION_LOCKED,
        JumpTimer::default(),
        AnimationTimer(Timer::from_seconds(animation_delay, TimerMode::Once)),
        AnimationController::default(),
        actions!(
            Player[
                (
                    Action::<Movement>::new(),
                    DeadZone::default(),
                    SmoothNudge::default(),
                    Scale::splat(MOVEMENT_VELOCITY),
                    Bindings::spawn((
                        Cardinal::arrows(),
                        Cardinal::wasd_keys(),
                        Axial::left_stick(),
                    ))
                ),
                (
                    Action::<Jump>::new(),
                    bindings![KeyCode::Space, GamepadButton::South],
                ),
            ]
        ),
    )
}

/// On a fired movement, set translation to the given input
fn apply_movement(
    event: On<Fire<Movement>>,
    controllers: Single<
        (&mut AnimationController, &mut KinematicCharacterController),
        With<Player>,
    >,
    pause: Res<State<Pause>>,
    time: Res<Time>,
) {
    // Return if game is paused
    if pause.get().0 {
        return;
    }

    // Assign controllers
    let (mut animation_controller, mut character_controller) = controllers.into_inner();

    // Apply movement from input
    let translation = event.value * time.delta_secs();
    animation_controller.state = AnimationState::Movement(translation);
    character_controller.translation = Some(translation);
}

/// On a completed movement, set translation to zero
fn stop_movement(
    _: On<Complete<Movement>>,
    controllers: Single<
        (&mut AnimationController, &mut KinematicCharacterController),
        With<Player>,
    >,
) {
    // Assign controllers
    let (mut animation_controller, mut character_controller) = controllers.into_inner();

    animation_controller.state = AnimationState::Idle;
    character_controller.translation = Some(Vec2::ZERO);
}

/// Jump velocity of the player
const JUMP_VELOCITY: f32 = 1.;

/// On a fired jump, move player up
fn apply_jump(
    _: On<Fire<Jump>>,
    controllers: Single<
        (&mut AnimationController, &mut KinematicCharacterController),
        With<Player>,
    >,
    pause: Res<State<Pause>>,
) {
    // Return if game is paused or jump has not been pressed
    if pause.get().0 {
        return;
    }

    // Assign controllers
    let (mut animation_controller, mut character_controller) = controllers.into_inner();

    // Return if we are already jumping
    if animation_controller.state == AnimationState::Jump {
        return;
    }

    // Get mutable Some of character_controller's translation or return
    let Some(ref mut translation) = character_controller.translation else {
        return;
    };

    // Apply jump
    animation_controller.state = AnimationState::Jump;
    translation.y = JUMP_VELOCITY;
}

/// On a completed jump, move player down
fn stop_jump(
    query: Single<
        (
            &mut AnimationController,
            &mut KinematicCharacterController,
            &JumpTimer,
        ),
        With<Player>,
    >,
) {
    // Assign controllers and timer from query
    let (mut animation_controller, mut character_controller, timer) = query.into_inner();

    // Return if timer has not finished
    if !timer.0.just_finished() {
        return;
    }

    // Get mutable Some of character_controller's translation or return
    let Some(ref mut translation) = character_controller.translation else {
        return;
    };

    // Apply fall
    animation_controller.state = AnimationState::Fall;
    translation.y = -JUMP_VELOCITY;
}
