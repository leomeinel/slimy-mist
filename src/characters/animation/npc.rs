/*
 * File: npc.rs
 * Author: Leopold Johannes Meinel (leo@meinel.dev)
 * -----
 * Copyright (c) 2025 Leopold Johannes Meinel & contributors
 * SPDX ID: Apache-2.0
 * URL: https://www.apache.org/licenses/LICENSE-2.0
 */

//! Animation for npc characters

use bevy::prelude::*;
use std::time::Duration;

use crate::{
    AppSystems, PausableSystems,
    characters::{
        animation::{
            MovementAnimation, MovementAnimationState, SoundFrames, trigger_step_sound_effect,
            update_animation_atlas, update_animation_movement, update_animation_timer,
        },
        npc::SlimeAssets,
    },
};

pub(super) fn plugin(app: &mut App) {
    app.insert_resource(SlimeSoundFrames(vec![3]));
    // Animate and play sound effects based on controls.
    app.add_systems(
        Update,
        (
            update_animation_timer::<SlimeAnimation>.in_set(AppSystems::TickTimers),
            (
                update_animation_movement::<SlimeAnimation>,
                update_animation_atlas::<SlimeAnimation>,
                trigger_step_sound_effect::<SlimeAnimation, SlimeAssets, SlimeSoundFrames>,
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
pub struct SlimeAnimation {
    timer: Timer,
    frame: usize,
    state: MovementAnimationState,
}

impl MovementAnimation for SlimeAnimation {
    /// The number of walking frames.
    const WALKING_FRAMES: usize = 2;
    /// The duration of each walking frame.
    const WALKING_INTERVAL: Duration = Duration::from_millis(200);

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
pub struct SlimeSoundFrames(Vec<usize>);

impl SoundFrames for SlimeSoundFrames {
    fn get_frames(&self) -> &Vec<usize> {
        &self.0
    }
}
