/*
 * File: input.rs
 * Author: Leopold Johannes Meinel (leo@meinel.dev)
 * -----
 * Copyright (c) 2026 Leopold Johannes Meinel & contributors
 * SPDX ID: Apache-2.0
 * URL: https://www.apache.org/licenses/LICENSE-2.0
 */

use bevy::{input::mouse::MouseButtonInput, prelude::*, window::PrimaryWindow};
use bevy_enhanced_input::prelude::*;
use bevy_rapier2d::prelude::*;
#[cfg(any(target_os = "android", target_os = "ios"))]
use virtual_joystick::VirtualJoystickMessage;

#[cfg(any(target_os = "android", target_os = "ios"))]
use crate::mobile::VirtualJoystick;
use crate::{
    Pause,
    camera::CanvasCamera,
    characters::{
        JumpTimer, Movement, VisualMap,
        animations::{AnimationController, AnimationState},
        combat::{AttackTimer, Attacked},
        player::Player,
    },
    logging::error::ERR_INVALID_VISUAL_MAP,
    screens::Screen,
};

pub(super) fn plugin(app: &mut App) {
    // Insert resources
    app.init_resource::<AimDirection>();

    // Add library plugins
    app.add_plugins(EnhancedInputPlugin);

    // FIXME: Currently when walking, melee is also triggered. We will have to
    //        Determine whether the touch was on the joystick or somewhere else.
    //        Using states to not allow melee while using the joystick will prohibit
    //        the player from attacking while waling and is not easily implemented
    //        because of scheduling.
    app.add_systems(
        PreUpdate,
        (
            // Mock `Aim` from clicks or override with touch input
            (mock_aim_from_click, mock_aim_from_touch).chain(),
            mock_melee_from_touch,
            #[cfg(any(target_os = "android", target_os = "ios"))]
            mock_walk_from_virtual_joystick,
        )
            .before(EnhancedInputSystems::Update)
            .run_if(in_state(Screen::Gameplay)),
    );

    // Handle bevy_enhanced_input with input context and observers
    app.add_input_context::<Player>();
    app.add_observer(apply_walk);
    app.add_observer(stop_walk);
    app.add_observer(set_jump);
    app.add_observer(trigger_melee);
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

/// Aim direction
#[derive(Resource, Default)]
pub(crate) struct AimDirection(pub(crate) Vec2);
impl AimDirection {
    /// Sets a new [`AimDirection::0`] if it has not already been set.
    pub(crate) fn set_new(&mut self, new: Vec2) {
        if self.0 != new {
            self.0 = new;
        }
    }
}

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
            // Combat
            (
                Action::<Melee>::new(),
                Tap::new(TAP_MAX_DURATION_SECS),
                bindings![MouseButton::Left, GamepadButton::RightTrigger],
            ),
            (
                Action::<Aim>::new(),
                Bindings::spawn(Axial::right_stick())
            ),
        ]
    )
}

/// Use [`ActionMock`] to mock [`Walk`] from the virtual joystick
#[cfg(any(target_os = "android", target_os = "ios"))]
fn mock_walk_from_virtual_joystick(
    mut reader: MessageReader<VirtualJoystickMessage<VirtualJoystick>>,
    walk: Single<Entity, With<Action<Walk>>>,
    mut commands: Commands,
) {
    for joystick in reader.read() {
        if joystick.id() != VirtualJoystick::Movement {
            continue;
        }

        let input = joystick.axis();
        if input == &Vec2::ZERO {
            continue;
        }
        commands.entity(walk.entity()).insert(ActionMock::once(
            ActionState::Fired,
            ActionValue::from(*input * PLAYER_WALK_SPEED),
        ));
    }
}

/// Use [`ActionMock`] to mock [`Melee`] from touch inputs.
fn mock_melee_from_touch(
    melee: Single<Entity, With<Action<Melee>>>,
    mut commands: Commands,
    touches: Res<Touches>,
) {
    // FIXME: We should check for taps within `TAP_MAX_DURATION_SECS` instead.
    if touches.any_just_released() {
        commands.entity(melee.entity()).insert(ActionMock::once(
            ActionState::Fired,
            ActionValue::Bool(true),
        ));
    }
}

/// Use [`ActionMock`] to mock [`Aim`] from touch inputs.
fn mock_aim_from_touch(
    aim: Single<Entity, With<Action<Aim>>>,
    camera: Single<(&Camera, &GlobalTransform), With<CanvasCamera>>,
    player_transform: Single<&Transform, With<Player>>,
    mut commands: Commands,
    touches: Res<Touches>,
) {
    let (camera, camera_transform) = *camera;

    // FIXME: We should check for taps within `TAP_MAX_DURATION_SECS` instead.
    for touch in touches.iter_just_released() {
        if let Ok(pos) = camera.viewport_to_world_2d(camera_transform, touch.position()) {
            let direction = pos - player_transform.translation.xy();
            commands.entity(aim.entity()).insert(ActionMock::once(
                ActionState::Fired,
                ActionValue::from(direction),
            ));
        }
    }
}

/// Use [`ActionMock`] to mock [`Aim`] from clicks.
fn mock_aim_from_click(
    mut reader: MessageReader<MouseButtonInput>,
    aim: Single<Entity, With<Action<Aim>>>,
    camera: Single<(&Camera, &GlobalTransform), With<CanvasCamera>>,
    player_transform: Single<&Transform, With<Player>>,
    window: Single<&Window, With<PrimaryWindow>>,
    mut commands: Commands,
) {
    let (camera, camera_transform) = *camera;

    // FIXME: We should check for taps within `TAP_MAX_DURATION_SECS` instead.
    for click in reader.read() {
        if click.button != MouseButton::Left {
            continue;
        }

        if let Some(pos) = window.cursor_position()
            && let Ok(pos) = camera.viewport_to_world_2d(camera_transform, pos)
        {
            let direction = pos - player_transform.translation.xy();
            commands.entity(aim.entity()).insert(ActionMock::once(
                ActionState::Fired,
                ActionValue::from(direction),
            ));
        }
    }
}

/// On a fired [`Walk`], set translation to the given input.
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

    // Apply movement from input
    movement.direction = event.value * time.delta_secs();
    character_controller.translation = Some(movement.direction);

    let visual = visual_map.0.get(&entity).expect(ERR_INVALID_VISUAL_MAP);
    let mut animation_controller = child_query.get_mut(*visual).expect(ERR_INVALID_VISUAL_MAP);

    // Set animation state if we are `Idle`
    if animation_controller.state == AnimationState::Idle {
        animation_controller.set_new_state(AnimationState::Walk);
    }
}

/// On a completed [`Walk`], set translation to zero.
fn stop_walk(
    _: On<Complete<Walk>>,
    parent: Single<(Entity, &mut KinematicCharacterController, &mut Movement), With<Player>>,
    mut child_query: Query<&mut AnimationController, Without<Player>>,
    visual_map: Res<VisualMap>,
) {
    let (entity, mut character_controller, mut movement) = parent.into_inner();

    // Reset movement diretion
    movement.direction = Vec2::ZERO;

    let visual = visual_map.0.get(&entity).expect(ERR_INVALID_VISUAL_MAP);
    let mut animation_controller = child_query.get_mut(*visual).expect(ERR_INVALID_VISUAL_MAP);

    // Stop movement if we are not jumping or falling
    if !matches!(
        animation_controller.state,
        AnimationState::Jump | AnimationState::Fall
    ) {
        character_controller.translation = Some(movement.direction);
        animation_controller.set_new_state(AnimationState::Idle);
    }
}

/// On a fired [`Jump`], add [`JumpTimer`].
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

    let visual = visual_map.0.get(&entity).expect(ERR_INVALID_VISUAL_MAP);
    let mut animation_controller = child_query.get_mut(*visual).expect(ERR_INVALID_VISUAL_MAP);

    // Set state to jump if we are not jumping or falling
    if !matches!(
        animation_controller.state,
        AnimationState::Jump | AnimationState::Fall
    ) {
        commands.entity(entity).insert(JumpTimer::default());
        animation_controller.set_new_state(AnimationState::Jump);
    }
}

/// On a fired [`Melee`], trigger [`Attacked`].
fn trigger_melee(
    _: On<Fire<Melee>>,
    parent: Single<(Entity, Option<&AttackTimer>), With<Player>>,
    aim: Single<&Action<Aim>>,
    mut commands: Commands,
    mut aim_direction: ResMut<AimDirection>,
    pause: Res<State<Pause>>,
) {
    // Return if game is paused
    if pause.get().0 {
        return;
    }
    // Return if `timer` has not finished
    let (entity, timer) = parent.into_inner();
    if let Some(timer) = timer
        && !timer.0.is_finished()
    {
        return;
    }

    // FIXME: Fall back to movement in certain scenarios. It feels weird if the player is moving
    //        in a different direction but their attack stays at the last aim_direction.
    //        The exact scenarios are quite hard to get right in my opinion, but this should be
    //        considered.
    //        I have already tried to set the direction in a system externally and some other
    //        approaches but none of them worked reliably. I'm not sure if this is worth
    //        fixing for now, but later it has to be done.
    //        I have already decoupled `AimDirection` to be able to access/change it externally.
    let direction = if (***aim).is_normalized() {
        ***aim
    } else {
        (***aim).normalize_or_zero()
    };
    aim_direction.set_new(direction);
    commands.trigger(Attacked(entity));
}
