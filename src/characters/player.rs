/*
 * File: player.rs
 * Author: Leopold Johannes Meinel (leo@meinel.dev)
 * -----
 * Copyright (c) 2026 Leopold Johannes Meinel & contributors
 * SPDX ID: Apache-2.0
 * URL: https://www.apache.org/licenses/LICENSE-2.0
 * -----
 * Heavily inspired by:
 * - https://github.com/TheBevyFlock/bevy_new_2d
 * - https://github.com/NiklasEi/bevy_common_assets/tree/main
 * - https://github.com/merwaaan/bevy_spritesheet_animation
 */

//! Player-specific behavior.

use bevy::prelude::*;
use bevy_asset_loader::prelude::*;
use bevy_enhanced_input::prelude::*;
use bevy_rapier2d::prelude::*;

use crate::{
    AppSystems, PausableSystems, Pause,
    characters::{
        Character, CharacterAssets, JumpTimer, Movement, MovementSpeed, VisualMap,
        animations::{self, AnimationController, AnimationState, Animations},
        character_collider,
        nav::{NavController, NavState},
        tick_jump_timer,
    },
    impl_character_assets,
    levels::{DEFAULT_Z, YSort, YSortOffset, YSorted},
    logging::error::ERR_INVALID_VISUAL_MAP,
    screens::Screen,
};

pub(super) fn plugin(app: &mut App) {
    // Insert Animation resource
    app.insert_resource(Animations::<Player>::default());

    // Add enhanced input plugin
    app.add_plugins(EnhancedInputPlugin);

    // Jump or stop jump depending on timer
    app.add_systems(
        Update,
        (
            apply_jump
                .before(PhysicsSet::SyncBackend)
                .in_set(AppSystems::Update),
            limit_jump.after(tick_jump_timer),
        )
            .chain(),
    );

    // Animation updates
    app.add_systems(
        Update,
        (
            animations::update_animations::<Player>.after(animations::tick_animation_timer),
            animations::update_animation_sounds::<Player, PlayerAssets>
                .run_if(in_state(Screen::Gameplay)),
        )
            .chain()
            .in_set(AppSystems::Update)
            .in_set(PausableSystems),
    );

    // Handle bevy_enhanced_input with input context and observers
    app.add_input_context::<Player>();
    app.add_observer(apply_walk);
    app.add_observer(stop_walk);
    app.add_observer(set_jump);
}

/// Assets that are serialized from a ron file
#[derive(AssetCollection, Resource, Reflect, Default)]
pub(crate) struct PlayerAssets {
    #[asset(key = "male.walk_sounds", collection(typed), optional)]
    pub(crate) walk_sounds: Option<Vec<Handle<AudioSource>>>,

    #[asset(key = "male.jump_sounds", collection(typed), optional)]
    pub(crate) jump_sounds: Option<Vec<Handle<AudioSource>>>,

    #[asset(key = "male.fall_sounds", collection(typed), optional)]
    pub(crate) fall_sounds: Option<Vec<Handle<AudioSource>>>,

    #[asset(key = "male.image")]
    pub(crate) image: Handle<Image>,
}
impl_character_assets!(PlayerAssets);

/// Player marker
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Default, Reflect)]
#[reflect(Component)]
pub(crate) struct Player;
impl Character for Player {
    fn container_bundle(
        &self,
        collision_set: &(Option<String>, Option<f32>, Option<f32>),
        pos: Vec2,
    ) -> impl Bundle {
        let movement_speed = MovementSpeed::default();

        (
            // FIXME: Use struct for this bundle
            Name::new("Player"),
            Self,
            Transform::from_translation(pos.extend(DEFAULT_Z)),
            YSort(DEFAULT_Z),
            character_collider(collision_set),
            Visibility::Inherited,
            RigidBody::KinematicVelocityBased,
            GravityScale(0.),
            KinematicCharacterController {
                filter_flags: QueryFilterFlags::EXCLUDE_KINEMATIC,
                ..default()
            },
            LockedAxes::ROTATION_LOCKED,
            NavController::default(),
            Movement::default(),
            movement_speed,
            actions!(
                Self[
                    (
                        Action::<Walk>::new(),
                        DeadZone::default(),
                        SmoothNudge::default(),
                        Scale::splat(movement_speed.0),
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
}
impl YSorted for Player {}

/// Walk marker
#[derive(Debug, InputAction)]
#[action_output(Vec2)]
struct Walk;

/// Jump marker
#[derive(Debug, InputAction)]
#[action_output(bool)]
struct Jump;

/// On a fired walk, set translation to the given input
fn apply_walk(
    event: On<Fire<Walk>>,
    parent: Single<
        (
            Entity,
            &mut KinematicCharacterController,
            &mut NavController,
            &mut Movement,
        ),
        With<Player>,
    >,
    mut child_query: Query<&mut AnimationController, Without<Player>>,
    pause: Res<State<Pause>>,
    time: Res<Time>,
    visual_map: Res<VisualMap>,
) {
    // Return if game is paused
    if pause.get().0 {
        return;
    }

    let (entity, mut character_controller, mut nav_controller, mut movement) = parent.into_inner();

    // Extract `animation_controller` from `child_query`
    let visual = visual_map.0.get(&entity).expect(ERR_INVALID_VISUAL_MAP);
    let mut animation_controller = child_query.get_mut(*visual).expect(ERR_INVALID_VISUAL_MAP);

    // Apply movement from input
    movement.direction = event.value * time.delta_secs();
    character_controller.translation = Some(movement.direction);

    // Update nav position if timer just finished
    nav_controller.state = NavState::UpdatePos;

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
    let visual = visual_map.0.get(&entity).expect(ERR_INVALID_VISUAL_MAP);
    let mut animation_controller = child_query.get_mut(*visual).expect(ERR_INVALID_VISUAL_MAP);

    // Reset movement target
    movement.direction = Vec2::ZERO;

    // Return if we are jumping
    let state = animation_controller.state;
    if state == AnimationState::Jump || state == AnimationState::Fall {
        return;
    }

    // Stop movement
    character_controller.translation = Some(movement.direction);
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

    let entity = parent.entity();

    // Extract `animation_controller` from `child_query`
    let visual = visual_map.0.get(&entity).expect(ERR_INVALID_VISUAL_MAP);
    let mut animation_controller = child_query.get_mut(*visual).expect(ERR_INVALID_VISUAL_MAP);

    // Return if we are already jumping
    let state = animation_controller.state;
    if state == AnimationState::Jump || state == AnimationState::Fall {
        return;
    }

    // Set state to jump
    commands.entity(entity).insert(JumpTimer::default());
    animation_controller.state = AnimationState::Jump;
}

/// Jump height
const JUMP_HEIGHT: f32 = 12.;

/// Apply jump
fn apply_jump(
    parent: Single<(Entity, &mut Movement, &JumpTimer), With<Player>>,
    mut child_query: Query<(&AnimationController, &mut Transform), Without<Player>>,
    mut commands: Commands,
    visual_map: Res<VisualMap>,
) {
    let (entity, mut movement, timer) = parent.into_inner();

    // Extract `animation_controller` from `child_query`
    let visual = visual_map.0.get(&entity).expect(ERR_INVALID_VISUAL_MAP);
    let (animation_controller, mut transform) =
        child_query.get_mut(*visual).expect(ERR_INVALID_VISUAL_MAP);

    let state = animation_controller.state;

    // Return if we are not jumping or falling
    if state != AnimationState::Jump && state != AnimationState::Fall {
        return;
    }

    // Apply visual jump or fall
    let factor = if state == AnimationState::Jump {
        1.0f32
    } else {
        -1.0f32
    };
    let eased_time = EasingCurve::new(0., 1., EaseFunction::QuadraticOut);
    let eased_time = eased_time.sample_clamped(timer.0.fraction());
    let target = JUMP_HEIGHT * factor * eased_time;

    transform.translation.y += target - movement.jump_height;
    movement.jump_height = target;

    // Apply `YSortOffset` for jump
    let y_sort_offset = if target < 0. {
        JUMP_HEIGHT + target
    } else {
        target
    };
    commands.entity(entity).insert(YSortOffset(y_sort_offset));
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
    let visual = visual_map.0.get(&entity).expect(ERR_INVALID_VISUAL_MAP);
    let mut animation_controller = child_query.get_mut(*visual).expect(ERR_INVALID_VISUAL_MAP);

    // Reset jump height
    movement.jump_height = 0.;

    // Set animation states
    match animation_controller.state {
        AnimationState::Jump => {
            commands.entity(entity).insert(JumpTimer::default());
            animation_controller.state = AnimationState::Fall;
        }
        AnimationState::Fall => {
            commands.entity(entity).remove::<YSortOffset>();
            animation_controller.state = AnimationState::Idle
        }
        _ => (),
    }
}
