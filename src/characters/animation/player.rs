/*
 * File: player.rs
 * Author: Leopold Johannes Meinel (leo@meinel.dev)
 * -----
 * Copyright (c) 2025 Leopold Johannes Meinel & contributors
 * SPDX ID: Apache-2.0
 * URL: https://www.apache.org/licenses/LICENSE-2.0
 */

//! Player sprite animation.
//! This is based on multiple examples and may be very different for your game.
//! - [Sprite flipping](https://github.com/bevyengine/bevy/blob/latest/examples/2d/sprite_flipping.rs)
//! - [Sprite animation](https://github.com/bevyengine/bevy/blob/latest/examples/2d/sprite_animation.rs)
//! - [Timers](https://github.com/bevyengine/bevy/blob/latest/examples/time/timers.rs)

use bevy::prelude::*;

use crate::{
    AppSystems, PausableSystems,
    characters::{
        animation::{
            MovementAnimation, MovementAnimationState, SoundFrames, trigger_step_sound_effect,
            update_animation_atlas, update_animation_movement, update_animation_timer,
        },
        player::PlayerAssets,
    },
};

pub(super) fn plugin(app: &mut App) {
    app.insert_resource(PlayerSoundFrames(vec![5, 9]));
    // Animate and play sound effects based on controls.
    app.add_systems(
        Update,
        (
            update_animation_timer::<PlayerAnimation>.in_set(AppSystems::TickTimers),
            (
                update_animation_movement::<PlayerAnimation>,
                update_animation_atlas::<PlayerAnimation>,
                trigger_step_sound_effect::<PlayerAnimation, PlayerAssets, PlayerSoundFrames>,
            )
                .chain()
                .in_set(AppSystems::Update),
        )
            .in_set(PausableSystems),
    );
}

/// Component that tracks player's animation state.
/// It is tightly bound to the texture atlas we use.
#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct PlayerAnimation {
    timer: Timer,
    frame: usize,
    state: MovementAnimationState,
}

impl MovementAnimation for PlayerAnimation {
    /// The number of idle frames.
    const IDLE_FRAMES: usize = 2;
    /// The number of walking frames.
    const WALKING_FRAMES: usize = 6;

    fn with_state(timer: Timer, state: MovementAnimationState) -> Self {
        Self {
            timer,
            frame: 0,
            state,
        }
    }

    fn get_timer(&self) -> &Timer {
        &self.timer
    }
    fn get_timer_mut(&mut self) -> &mut Timer {
        &mut self.timer
    }

    fn get_frame(&self) -> &usize {
        &self.frame
    }
    fn set_frame(&mut self, frame_: usize) {
        self.frame = frame_;
    }

    fn get_state(&self) -> &MovementAnimationState {
        &self.state
    }
}

#[derive(Resource)]
pub struct PlayerSoundFrames(Vec<usize>);

impl SoundFrames for PlayerSoundFrames {
    fn get_frames(&self) -> &Vec<usize> {
        &self.0
    }
}
