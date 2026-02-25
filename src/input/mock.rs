/*
 * File: mock.rs
 * Author: Leopold Johannes Meinel (leo@meinel.dev)
 * -----
 * Copyright (c) 2026 Leopold Johannes Meinel & contributors
 * SPDX ID: Apache-2.0
 * URL: https://www.apache.org/licenses/LICENSE-2.0
 */

use bevy::{prelude::*, window::PrimaryWindow};
use bevy_enhanced_input::prelude::*;
use virtual_joystick::VirtualJoystickMessage;

use crate::characters::player::PLAYER_WALK_SPEED;
use crate::input::InputSystems;
use crate::input::actions::{Aim, Jump, Melee, Walk};
use crate::input::joystick::{JoystickID, JoystickRect};
use crate::input::pointer::{MouseDrag, PointerStartTimeSecs, Swipe as _};
use crate::{camera::CanvasCamera, characters::player::Player};

pub(super) fn plugin(app: &mut App) {
    app.add_systems(
        PreUpdate,
        (
            mock_walk_from_virtual_joystick,
            mock_jump_from_touch,
            (mock_melee_from_click, mock_melee_from_touch).chain(),
            (mock_aim_from_click, mock_aim_from_touch).chain(),
        )
            .in_set(InputSystems::Mock)
            .chain(),
    );
}

/// Mock [`Walk`] from the virtual joystick
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
            .mock_once::<Player, Walk>(TriggerState::Fired, *input * PLAYER_WALK_SPEED);
    }
}

/// Mock [`Jump`] from touch inputs.
fn mock_jump_from_touch(
    jump: Single<Entity, With<Player>>,
    mut commands: Commands,
    touches: Res<Touches>,
    rect: Res<JoystickRect<{ JoystickID::Movement as u8 }>>,
) {
    for touch in touches.iter_just_released() {
        if rect.intersects_with(touch.start_position()) {
            continue;
        }

        if touch.is_swipe_up() {
            commands
                .entity(*jump)
                .mock_once::<Player, Jump>(TriggerState::Fired, true);
        }
    }
}

/// Mock [`Melee`] from touch inputs.
fn mock_melee_from_touch(
    melee: Single<Entity, With<Player>>,
    mut commands: Commands,
    pointer_start: Res<PointerStartTimeSecs>,
    touches: Res<Touches>,
    rect: Res<JoystickRect<{ JoystickID::Movement as u8 }>>,
    time: Res<Time>,
) {
    if !pointer_start.is_tap(time.elapsed_secs())
        || touches
            .iter_just_released()
            .any(|t| rect.intersects_with(t.start_position()))
    {
        return;
    }

    if touches.iter_just_released().any(|t| !t.is_vertical_swipe()) {
        commands
            .entity(*melee)
            .mock_once::<Player, Melee>(TriggerState::Fired, true);
    }
}

/// Mock [`Aim`] from touch inputs.
fn mock_aim_from_touch(
    aim: Single<Entity, With<Player>>,
    camera: Single<(&Camera, &GlobalTransform), With<CanvasCamera>>,
    player_transform: Single<&Transform, With<Player>>,
    mut commands: Commands,
    touches: Res<Touches>,
    rect: Res<JoystickRect<{ JoystickID::Movement as u8 }>>,
) {
    let (camera, camera_transform) = *camera;

    // NOTE: We are using `just_pressed` to allow use in `Melee`.
    for touch in touches.iter_just_pressed() {
        if let Ok(pos) = camera.viewport_to_world_2d(camera_transform, touch.position()) {
            if rect.intersects_with(pos) {
                continue;
            }

            let direction = pos - player_transform.translation.xy();
            commands.entity(*aim).mock::<Player, Aim>(
                TriggerState::Fired,
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
    drag: Res<MouseDrag>,
    mouse: Res<ButtonInput<MouseButton>>,
    pointer_start: Res<PointerStartTimeSecs>,
    rect: Res<JoystickRect<{ JoystickID::Movement as u8 }>>,
    time: Res<Time>,
) {
    let Some(start_pos) = drag.start_pos else {
        return;
    };
    if !mouse.just_released(MouseButton::Left)
        || !pointer_start.is_tap(time.elapsed_secs())
        || rect.intersects_with(start_pos)
        || drag.is_vertical_swipe()
    {
        return;
    }

    commands
        .entity(*melee)
        .mock_once::<Player, Melee>(TriggerState::Fired, true);
}

/// Mock [`Aim`] from clicks.
fn mock_aim_from_click(
    aim: Single<Entity, With<Player>>,
    camera: Single<(&Camera, &GlobalTransform), With<CanvasCamera>>,
    player_transform: Single<&Transform, With<Player>>,
    window: Single<&Window, With<PrimaryWindow>>,
    mut commands: Commands,
    mouse: Res<ButtonInput<MouseButton>>,
    rect: Res<JoystickRect<{ JoystickID::Movement as u8 }>>,
) {
    // NOTE: We are using `just_pressed` to allow use in `Melee`.
    if !mouse.just_pressed(MouseButton::Left) {
        return;
    }

    let (camera, camera_transform) = *camera;

    if let Some(pos) = window.cursor_position()
        && let Ok(pos) = camera.viewport_to_world_2d(camera_transform, pos)
    {
        if rect.intersects_with(pos) {
            return;
        }

        let direction = pos - player_transform.translation.xy();
        commands.entity(*aim).mock::<Player, Aim>(
            TriggerState::Fired,
            direction.normalize_or_zero(),
            MockSpan::Manual,
        );
    }
}
