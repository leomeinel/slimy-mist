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
        CharacterAssets, CollisionData, CollisionHandle, JumpTimer, Movement, VisualMap, collider,
        tick_jump_timer,
    },
    impl_character_assets,
    utils::maths::ease_out_quad,
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

    // Jump or stop jump depending on timer
    app.add_systems(
        Update,
        (
            apply_jump
                .before(PhysicsSet::SyncBackend)
                .in_set(AppSystems::Update)
                .in_set(PausableSystems),
            limit_jump.after(tick_jump_timer),
        )
            .chain(),
    );

    // Handle bevy_enhanced_input with input context and observers
    app.add_input_context::<Player>();
    app.add_observer(apply_walk);
    app.add_observer(stop_walk);
    app.add_observer(set_jump);
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

/// Walk marker
#[derive(Debug, InputAction)]
#[action_output(Vec2)]
struct Walk;

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

/// Walking speed of the player
const WALK_SPEED: f32 = 80.;

/// The player character.
pub(crate) fn player(
    collision_data: &Res<Assets<CollisionData<Player>>>,
    collision_handle: &Res<CollisionHandle<Player>>,
) -> impl Bundle {
    (
        Name::new("Player"),
        Player,
        RigidBody::KinematicVelocityBased,
        GravityScale(0.),
        collider::<Player>(collision_data, collision_handle),
        KinematicCharacterController::default(),
        LockedAxes::ROTATION_LOCKED,
        Movement::default(),
        actions!(
            Player[
                (
                    Action::<Walk>::new(),
                    DeadZone::default(),
                    SmoothNudge::default(),
                    Scale::splat(WALK_SPEED),
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

/// Player child with visual components
pub(crate) fn player_visual(
    animations: &Res<Animations<Player>>,
    animation_delay: f32,
) -> impl Bundle {
    (
        animations.sprite.clone(),
        SpritesheetAnimation::new(animations.idle.clone()),
        AnimationController::default(),
        AnimationTimer(Timer::from_seconds(animation_delay, TimerMode::Once)),
    )
}

/// On a fired walk, set translation to the given input
fn apply_walk(
    event: On<Fire<Walk>>,
    parent: Single<(Entity, &mut KinematicCharacterController, &mut Movement), With<Player>>,
    mut child_query: Query<&mut AnimationController, Without<Player>>,
    pause: Res<State<Pause>>,
    time: Res<Time>,
    visual_map: Res<VisualMap>,
) {
    // Return if game is paused
    if pause.get().0 {
        return;
    }

    let (entity, mut character_controller, mut movement) = parent.into_inner();

    // Extract `animation_controller` from `child_query`
    let Some(visual) = visual_map.0.get(&entity) else {
        return;
    };
    let Ok(mut animation_controller) = child_query.get_mut(*visual) else {
        return;
    };

    // Apply movement from input
    movement.target = event.value * time.delta_secs();
    character_controller.translation = Some(movement.target);

    // Return if we are jumping
    let state = animation_controller.state;
    if state == AnimationState::Jump || state == AnimationState::Fall {
        return;
    }

    // Set animation state
    animation_controller.state = AnimationState::Walk;
}

/// On a completed walk, set translation to zero
fn stop_walk(
    _: On<Complete<Walk>>,
    parent: Single<(Entity, &mut KinematicCharacterController, &mut Movement), With<Player>>,
    mut child_query: Query<&mut AnimationController, Without<Player>>,
    visual_map: Res<VisualMap>,
) {
    let (entity, mut character_controller, mut movement) = parent.into_inner();

    // Extract `animation_controller` from `child_query`
    let Some(visual) = visual_map.0.get(&entity) else {
        return;
    };
    let Ok(mut animation_controller) = child_query.get_mut(*visual) else {
        return;
    };

    // Reset movement target
    movement.target = Vec2::ZERO;

    // Return if we are jumping
    let state = animation_controller.state;
    if state == AnimationState::Jump || state == AnimationState::Fall {
        return;
    }

    // Stop movement
    character_controller.translation = Some(movement.target);
    animation_controller.state = AnimationState::Idle;
}

// On a fired jump, move player up
fn set_jump(
    _: On<Fire<Jump>>,
    parent: Single<Entity, With<Player>>,
    mut child_query: Query<&mut AnimationController, Without<Player>>,
    mut commands: Commands,
    pause: Res<State<Pause>>,
    visual_map: Res<VisualMap>,
) {
    // Return if game is paused
    if pause.get().0 {
        return;
    }

    let entity = parent.into_inner();

    // Extract `animation_controller` from `child_query`
    let Some(visual) = visual_map.0.get(&entity) else {
        return;
    };
    let Ok(mut animation_controller) = child_query.get_mut(*visual) else {
        return;
    };

    // Return if we are already jumping
    let state = animation_controller.state;
    if state == AnimationState::Jump || state == AnimationState::Fall {
        return;
    }

    // Set state to jump
    commands.entity(entity).insert(JumpTimer::default());
    animation_controller.state = AnimationState::Jump;
}

const JUMP_HEIGHT: f32 = 24.;

/// Apply jump
fn apply_jump(
    parent: Single<(Entity, &mut Movement, &JumpTimer), With<Player>>,
    mut child_query: Query<(&AnimationController, &mut Transform), Without<Player>>,
    visual_map: Res<VisualMap>,
) {
    let (entity, mut movement, timer) = parent.into_inner();

    // Extract `animation_controller` from `child_query`
    let Some(visual) = visual_map.0.get(&entity) else {
        return;
    };
    let Ok((animation_controller, mut transform)) = child_query.get_mut(*visual) else {
        return;
    };

    let state = animation_controller.state;

    // Return if we are not jumping or falling
    if state != AnimationState::Jump && state != AnimationState::Fall {
        return;
    }

    // Apply visual jump or fall
    let multiplier = if state == AnimationState::Jump {
        1.0f32
    } else {
        -1.0f32
    };
    let target = JUMP_HEIGHT * multiplier * ease_out_quad(timer.0.fraction());

    transform.translation.y += target - movement.jump_height;
    movement.jump_height = target;
}

/// Limit jump by setting fall after specific time and then switching to walk
fn limit_jump(
    parent: Single<(Entity, &mut Movement, &JumpTimer), With<Player>>,
    mut child_query: Query<&mut AnimationController, Without<Player>>,
    mut commands: Commands,
    visual_map: Res<VisualMap>,
) {
    let (entity, mut movement, timer) = parent.into_inner();

    // Return if timer has not finished
    if !timer.0.just_finished() {
        return;
    }

    // Extract `animation_controller` from `child_query`
    let Some(visual) = visual_map.0.get(&entity) else {
        return;
    };
    let Ok(mut animation_controller) = child_query.get_mut(*visual) else {
        return;
    };

    // Reset jump height
    movement.jump_height = 0.;

    // Set animation states
    match animation_controller.state {
        AnimationState::Jump => {
            commands.entity(entity).insert(JumpTimer::default());
            animation_controller.state = AnimationState::Fall;
        }
        AnimationState::Fall => animation_controller.state = AnimationState::Idle,
        _ => (),
    }
}
