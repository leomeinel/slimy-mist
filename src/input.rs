/*
 * File: input.rs
 * Author: Leopold Johannes Meinel (leo@meinel.dev)
 * -----
 * Copyright (c) 2026 Leopold Johannes Meinel & contributors
 * SPDX ID: Apache-2.0
 * URL: https://www.apache.org/licenses/LICENSE-2.0
 */

// FIXME: We currently don't have a way to handle joystick drift.

use std::marker::PhantomData;

#[cfg(any(target_os = "android", target_os = "ios"))]
use bevy::math::u8;
use bevy::{input::touch::TouchPhase, prelude::*, window::PrimaryWindow};
use bevy_enhanced_input::prelude::*;
use bevy_rapier2d::prelude::*;
#[cfg(any(target_os = "android", target_os = "ios"))]
use virtual_joystick::VirtualJoystickMessage;

use crate::{
    Pause,
    animations::{AnimationCache, AnimationState},
    camera::CanvasCamera,
    characters::{
        JumpTimer, Movement,
        attack::{Attack, AttackTimer, MeleeAttack},
        player::Player,
    },
    screens::Screen,
};
#[cfg(any(target_os = "android", target_os = "ios"))]
use crate::{
    logging::error::ERR_INVALID_POINTER_CACHE,
    mobile::{JoystickID, JoystickInteractionRectMap},
};

pub(super) fn plugin(app: &mut App) {
    // Add library plugins
    app.add_plugins(EnhancedInputPlugin);

    app.add_systems(
        PreUpdate,
        (
            (update_pointer_input_cache),
            #[cfg(any(target_os = "android", target_os = "ios"))]
            mock_walk_from_virtual_joystick,
            (
                mock_jump_from_touch,
                (mock_melee_from_click, mock_melee_from_touch).chain(),
                (mock_aim_from_click, mock_aim_from_touch).chain(),
            ),
        )
            .before(EnhancedInputSystems::Update)
            .run_if(in_state(Screen::Gameplay))
            .chain(),
    );

    // Handle bevy_enhanced_input with input context and observers
    app.add_input_context::<Player>();
    app.add_observer(apply_walk);
    app.add_observer(reset_walk);
    app.add_observer(set_jump);
    app.add_observer(trigger_melee_attack);
    app.add_observer(reset_aim);
}

/// Walk [`InputAction`]
#[derive(InputAction)]
#[action_output(Vec2)]
pub(crate) struct Walk;

/// Jump [`InputAction`]
#[derive(InputAction)]
#[action_output(bool)]
pub(crate) struct Jump;

/// Melee attack [`InputAction`]
#[derive(InputAction)]
#[action_output(bool)]
pub(crate) struct Melee;

/// Aim direction [`InputAction`]
#[derive(InputAction)]
#[action_output(Vec2)]
pub(crate) struct Aim;

/// Max duration for a tap to be recognized.
const TAP_MAX_DURATION_SECS: f32 = 0.5;

/// Walk speed of [`Player`].
const PLAYER_WALK_SPEED: f32 = 80.;

/// Input [`Action`]s for [`Player`].
pub(crate) fn player_input() -> impl Bundle {
    actions!(
        Player[
            // Movement
            (
                Action::<Walk>::new(),
                ActionSettings {
                    require_reset: true,
                    ..default()
                },
                DeadZone::default(),
                SmoothNudge::default(),
                Scale::splat(PLAYER_WALK_SPEED),
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
            // Attack
            (
                Action::<Melee>::new(),
                bindings![GamepadButton::RightTrigger],
            ),
            (
                Action::<Aim>::new(),
                ActionSettings {
                    require_reset: true,
                    ..default()
                },
                Bindings::spawn(Axial::right_stick())
            ),
        ]
    )
}

/// Info on pointer input that is not natively provided by [`bevy`].
#[derive(Resource, Default)]
pub(crate) struct PointerInputCache {
    start_pos: Option<Vec2>,
    start_time_secs: f32,
}

/// Update info in [`PointerInputCache`].
///
/// This prioritizes [`TouchInput`] but also handles [`ButtonInput<MouseButton>`] for [`MouseButton::Left`].
fn update_pointer_input_cache(
    mut reader: MessageReader<TouchInput>,
    window: Single<&Window, With<PrimaryWindow>>,
    mut input_cache: ResMut<PointerInputCache>,
    mouse: Res<ButtonInput<MouseButton>>,
    time: Res<Time>,
) {
    if reader.read().any(|t| t.phase == TouchPhase::Started) {
        input_cache.start_pos = None;
        input_cache.start_time_secs = time.elapsed_secs();
        return;
    }

    if mouse.just_pressed(MouseButton::Left)
        && let Some(pos) = window.cursor_position()
    {
        input_cache.start_pos = Some(pos);
        input_cache.start_time_secs = time.elapsed_secs();
    }
}

/// Mock [`Walk`] from the virtual joystick
#[cfg(any(target_os = "android", target_os = "ios"))]
fn mock_walk_from_virtual_joystick(
    mut reader: MessageReader<VirtualJoystickMessage<u8>>,
    walk: Single<Entity, With<Player>>,
    mut commands: Commands,
) {
    for joystick in reader.read() {
        if joystick.id() != JoystickID::Movement as u8 {
            continue;
        }

        let input = joystick.axis();
        if input == &Vec2::ZERO {
            continue;
        }
        commands
            .entity(*walk)
            .mock_once::<Player, Walk>(ActionState::Fired, *input * PLAYER_WALK_SPEED);
    }
}

/// Threshold for a valid swipe action from touch input in logical pixels.
const SWIPE_THRESHOLD: f32 = 50.;

/// Mock [`Jump`] from touch inputs.
fn mock_jump_from_touch(
    jump: Single<Entity, With<Player>>,
    mut commands: Commands,
    touches: Res<Touches>,
    #[cfg(any(target_os = "android", target_os = "ios"))] rect_map: Res<JoystickInteractionRectMap>,
) {
    for touch in touches.iter_just_released() {
        #[cfg(any(target_os = "android", target_os = "ios"))]
        if rect_map.any_intersect_with(touch.start_position()) {
            continue;
        }

        let distance = touch.distance();
        // NOTE: We are inverting y to align with user intent because `distance` is reversed on the y axis.
        if -distance.y > SWIPE_THRESHOLD && distance.y.abs() > distance.x.abs() {
            commands
                .entity(*jump)
                .mock_once::<Player, Jump>(ActionState::Fired, true);
        }
    }
}

/// Mock [`Melee`] from touch inputs.
fn mock_melee_from_touch(
    melee: Single<Entity, With<Player>>,
    mut commands: Commands,
    touches: Res<Touches>,
    input_cache: Res<PointerInputCache>,
    #[cfg(any(target_os = "android", target_os = "ios"))] rect_map: Res<JoystickInteractionRectMap>,
    time: Res<Time>,
) {
    if time.elapsed_secs() - input_cache.start_time_secs > TAP_MAX_DURATION_SECS {
        return;
    }
    #[cfg(any(target_os = "android", target_os = "ios"))]
    if touches
        .iter_just_released()
        .any(|t| rect_map.any_intersect_with(t.start_position()))
    {
        return;
    }

    if touches.any_just_released() {
        commands
            .entity(*melee)
            .mock_once::<Player, Melee>(ActionState::Fired, true);
    }
}

/// Mock [`Aim`] from touch inputs.
fn mock_aim_from_touch(
    aim: Single<Entity, With<Player>>,
    camera: Single<(&Camera, &GlobalTransform), With<CanvasCamera>>,
    player_transform: Single<&Transform, With<Player>>,
    mut commands: Commands,
    touches: Res<Touches>,
    #[cfg(any(target_os = "android", target_os = "ios"))] rect_map: Res<JoystickInteractionRectMap>,
) {
    let (camera, camera_transform) = *camera;

    // NOTE: We are using `just_pressed` to allow use in `Melee`.
    for touch in touches.iter_just_pressed() {
        if let Ok(pos) = camera.viewport_to_world_2d(camera_transform, touch.position()) {
            #[cfg(any(target_os = "android", target_os = "ios"))]
            if rect_map.any_intersect_with(pos) {
                continue;
            }

            let direction = pos - player_transform.translation.xy();
            commands.entity(*aim).mock::<Player, Aim>(
                ActionState::Fired,
                direction.normalize_or_zero(),
                MockSpan::Manual,
            );
        }
    }
}

/// Mock [`Melee`] from clicks.
fn mock_melee_from_click(
    melee: Single<Entity, With<Player>>,
    mut commands: Commands,
    input_cache: Res<PointerInputCache>,
    mouse: Res<ButtonInput<MouseButton>>,
    #[cfg(any(target_os = "android", target_os = "ios"))] rect_map: Res<JoystickInteractionRectMap>,
    time: Res<Time>,
) {
    if !mouse.just_released(MouseButton::Left)
        || time.elapsed_secs() - input_cache.start_time_secs > TAP_MAX_DURATION_SECS
    {
        return;
    }
    #[cfg(any(target_os = "android", target_os = "ios"))]
    if rect_map.any_intersect_with(input_cache.pos.expect(ERR_INVALID_POINTER_CACHE)) {
        return;
    }

    commands
        .entity(*melee)
        .mock_once::<Player, Melee>(ActionState::Fired, true);
}

/// Mock [`Aim`] from clicks.
fn mock_aim_from_click(
    aim: Single<Entity, With<Player>>,
    camera: Single<(&Camera, &GlobalTransform), With<CanvasCamera>>,
    player_transform: Single<&Transform, With<Player>>,
    window: Single<&Window, With<PrimaryWindow>>,
    mut commands: Commands,
    mouse: Res<ButtonInput<MouseButton>>,
    #[cfg(any(target_os = "android", target_os = "ios"))] rect_map: Res<JoystickInteractionRectMap>,
) {
    // NOTE: We are using `just_pressed` to allow use in `Melee`.
    if !mouse.just_pressed(MouseButton::Left) {
        return;
    }

    let (camera, camera_transform) = *camera;

    if let Some(pos) = window.cursor_position()
        && let Ok(pos) = camera.viewport_to_world_2d(camera_transform, pos)
    {
        #[cfg(any(target_os = "android", target_os = "ios"))]
        if rect_map.any_intersect_with(pos) {
            return;
        }

        let direction = pos - player_transform.translation.xy();
        commands.entity(*aim).mock::<Player, Aim>(
            ActionState::Fired,
            direction.normalize_or_zero(),
            MockSpan::Manual,
        );
    }
}

/// On a fired [`Walk`], set translation to the given input.
fn apply_walk(
    event: On<Fire<Walk>>,
    player: Single<
        (
            &mut AnimationCache,
            &mut KinematicCharacterController,
            &mut Movement,
        ),
        With<Player>,
    >,
    pause: Res<State<Pause>>,
    time: Res<Time>,
) {
    // Return if game is paused
    if pause.get().0 {
        return;
    }

    let (mut cache, mut controller, mut movement) = player.into_inner();

    // Apply movement from input
    movement.direction = event.value * time.delta_secs();
    controller.translation = Some(movement.direction);

    // Set animation state if we are `Idle`
    if cache.state == AnimationState::Idle {
        cache.set_new_state(AnimationState::Walk);
    }
}

/// On a completed [`Walk`], set translation to zero.
fn reset_walk(
    _: On<Complete<Walk>>,
    player: Single<
        (
            &mut AnimationCache,
            &mut KinematicCharacterController,
            &mut Movement,
        ),
        With<Player>,
    >,
) {
    let (mut cache, mut controller, mut movement) = player.into_inner();

    // Reset `movement.direction`
    movement.direction = Vec2::ZERO;

    // Stop movement if we are not jumping or falling
    if !matches!(cache.state, AnimationState::Jump | AnimationState::Fall) {
        controller.translation = Some(movement.direction);
        cache.set_new_state(AnimationState::Idle);
    }
}

/// On a fired [`Jump`], add [`JumpTimer`].
fn set_jump(
    _: On<Fire<Jump>>,
    player: Single<(Entity, &mut AnimationCache), With<Player>>,
    mut commands: Commands,
    pause: Res<State<Pause>>,
) {
    // Return if game is paused
    if pause.get().0 {
        return;
    }

    let (entity, mut cache) = player.into_inner();

    // Set state to jump if we are not jumping or falling
    if !matches!(cache.state, AnimationState::Jump | AnimationState::Fall) {
        commands.entity(entity).insert(JumpTimer::default());
        cache.set_new_state(AnimationState::Jump);
    }
}

/// On a fired [`Melee`], trigger [`Attack`].
fn trigger_melee_attack(
    _: On<Fire<Melee>>,
    aim: Single<&Action<Aim>>,
    player: Single<(Entity, Option<&AttackTimer>), With<Player>>,
    mut commands: Commands,
    pause: Res<State<Pause>>,
) {
    // Return if game is paused
    if pause.get().0 {
        return;
    }
    // Return if `timer` has not finished
    let (entity, timer) = *player;
    if let Some(timer) = timer
        && !timer.0.is_finished()
    {
        return;
    }

    commands.trigger(Attack::<MeleeAttack> {
        entity,
        direction: ***aim,
        _phantom: PhantomData,
    });
}

/// On a completed [`Melee`], reset [`Aim`].
fn reset_aim(_: On<Complete<Melee>>, aim: Single<(&mut Action<Aim>, Option<&mut ActionMock>)>) {
    let (mut aim, mock) = aim.into_inner();

    // Reset `aim` and `mock`
    **aim = Vec2::ZERO;
    if let Some(mut mock) = mock {
        mock.enabled = false;
    }
}
