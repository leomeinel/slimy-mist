/*
 * File: input.rs
 * Author: Leopold Johannes Meinel (leo@meinel.dev)
 * -----
 * Copyright (c) 2026 Leopold Johannes Meinel & contributors
 * SPDX ID: Apache-2.0
 * URL: https://www.apache.org/licenses/LICENSE-2.0
 */

// FIXME: We currently don't have a way to handle joystick drift.

pub(crate) mod actions;
pub(crate) mod joystick;
mod mock;
pub(crate) mod pointer;

use std::marker::PhantomData;

use bevy::prelude::*;
use bevy_enhanced_input::prelude::*;
use bevy_rapier2d::prelude::*;

use crate::{
    Pause,
    animations::{AnimationCache, AnimationState},
    characters::{
        JumpTimer, Movement,
        attack::{Attack, AttackTimer, MeleeAttack},
        player::Player,
    },
    input::actions::{Aim, Jump, Melee, Walk},
    screens::Screen,
};

pub(super) fn plugin(app: &mut App) {
    // Add library plugins
    app.add_plugins(EnhancedInputPlugin);

    // Add child plugins
    app.add_plugins((joystick::plugin, mock::plugin, pointer::plugin));

    // Order new `InitGameplaySystems` variants by adding them here:
    app.configure_sets(
        PreUpdate,
        (InputSystems::Cache, InputSystems::Mock)
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

/// A [`SystemSet`] for systems that initialize [`Screen::Gameplay`]
#[derive(SystemSet, Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub(crate) enum InputSystems {
    Cache,
    Mock,
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
