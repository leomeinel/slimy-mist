/*
 * File: pointer.rs
 * Author: Leopold Johannes Meinel (leo@meinel.dev)
 * -----
 * Copyright (c) 2026 Leopold Johannes Meinel & contributors
 * SPDX ID: Apache-2.0
 * URL: https://www.apache.org/licenses/LICENSE-2.0
 */

use bevy::{
    input::touch::{Touch, TouchPhase},
    prelude::*,
    window::PrimaryWindow,
};
use bevy_enhanced_input::prelude::*;

use crate::screens::Screen;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(
        PreUpdate,
        (update_pointer_start_time_secs, update_mouse_drag)
            .before(EnhancedInputSystems::Update)
            .run_if(in_state(Screen::Gameplay)),
    );
}

/// Threshold for a valid swipe action from touch input in logical pixels.
const SWIPE_THRESHOLD: f32 = 50.;

/// Trait for determining if input is a swipe.
pub(crate) trait Swipe {
    fn is_vertical_swipe(&self) -> bool;
    fn is_swipe_up(&self) -> bool;
}
impl Swipe for Touch {
    fn is_vertical_swipe(&self) -> bool {
        let d = self.distance();
        d.y.abs() > SWIPE_THRESHOLD && d.y.abs() > d.x.abs()
    }
    fn is_swipe_up(&self) -> bool {
        // NOTE: We are inverting y to align with user intent because `distance` is reversed on the y axis.
        self.is_vertical_swipe() && self.distance().y < 0.
    }
}

/// Max duration for a tap to be recognized.
const TAP_MAX_DURATION_SECS: f32 = 0.5;

/// Info on pointer input that is not natively provided by [`bevy`].
#[derive(Resource, Default)]
pub(crate) struct PointerStartTimeSecs(f32);
impl PointerStartTimeSecs {
    pub(crate) fn is_tap(&self, time_secs: f32) -> bool {
        time_secs - self.0 <= TAP_MAX_DURATION_SECS
    }
}

/// Info on pointer input that is not natively provided by [`bevy`].
#[derive(Resource, Default)]
pub(crate) struct MouseDrag {
    pub(crate) start_pos: Option<Vec2>,
    pub(crate) current_pos: Option<Vec2>,
}
impl MouseDrag {
    fn distance(&self) -> Option<Vec2> {
        Some(self.current_pos? - self.start_pos?)
    }
}
impl Swipe for MouseDrag {
    fn is_vertical_swipe(&self) -> bool {
        self.distance()
            .is_some_and(|d| d.y.abs() > SWIPE_THRESHOLD && d.y.abs() > d.x.abs())
    }
    fn is_swipe_up(&self) -> bool {
        // NOTE: We are inverting y to align with user intent because `distance` is reversed on the y axis.
        self.distance()
            .is_some_and(|d| self.is_vertical_swipe() && d.y < 0.)
    }
}

/// Update info in [`PointerStartTimeSecs`].
///
/// This handles [`TouchInput`] and [`ButtonInput<MouseButton>`] for [`MouseButton::Left`].
fn update_pointer_start_time_secs(
    mut reader: MessageReader<TouchInput>,
    mut pointer_start: ResMut<PointerStartTimeSecs>,
    mouse: Res<ButtonInput<MouseButton>>,
    time: Res<Time>,
) {
    if reader.read().any(|t| t.phase == TouchPhase::Started)
        || mouse.just_pressed(MouseButton::Left)
    {
        pointer_start.0 = time.elapsed_secs();
    }
}

/// Update info in [`MouseDrag`].
///
/// This only handles [`ButtonInput<MouseButton>`] for [`MouseButton::Left`].
fn update_mouse_drag(
    window: Single<&Window, With<PrimaryWindow>>,
    mut drag: ResMut<MouseDrag>,
    mouse: Res<ButtonInput<MouseButton>>,
) {
    drag.current_pos = window.cursor_position();
    if mouse.just_pressed(MouseButton::Left) {
        drag.start_pos = drag.current_pos;
    }
}
