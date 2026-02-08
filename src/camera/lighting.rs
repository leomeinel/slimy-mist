/*
 * File: lighting.rs
 * Author: Leopold Johannes Meinel (leo@meinel.dev)
 * -----
 * Copyright (c) 2026 Leopold Johannes Meinel & contributors
 * SPDX ID: Apache-2.0
 * URL: https://www.apache.org/licenses/LICENSE-2.0
 */

use bevy::prelude::*;
use bevy_light_2d::prelude::*;

use crate::{
    AppSystems, PausableSystems, camera::CanvasCamera, logging::error::ERR_INVALID_DOMAIN_EASING,
    screens::Screen,
};

pub(super) fn plugin(app: &mut App) {
    // Add ambient light after entering `Screen::Gameplay` and reset when exiting.
    app.add_systems(OnEnter(Screen::Gameplay), add_ambient);
    app.add_systems(OnExit(Screen::Gameplay), reset_ambient);

    // Update ambient brightness to simulate Day/Night cycle.
    app.add_systems(
        Update,
        update_ambient_brightness
            .run_if(in_state(Screen::Gameplay))
            .in_set(PausableSystems),
    );

    // Tick day timer
    app.add_systems(
        Update,
        tick_day_timer
            .in_set(AppSystems::TickTimers)
            .run_if(in_state(Screen::Gameplay))
            .in_set(PausableSystems),
    );
}

/// Seconds in a day.
const DAY_SECS: f32 = 300.;

/// Timer that tracks splash screen
#[derive(Resource, Debug, Clone, PartialEq, Reflect)]
#[reflect(Resource)]
pub(crate) struct DayTimer(Timer);
impl Default for DayTimer {
    fn default() -> Self {
        Self(Timer::from_seconds(DAY_SECS, TimerMode::Repeating))
    }
}

/// Insert [`Light2d`] into [`CanvasCamera`].
fn add_ambient(camera: Single<Entity, With<CanvasCamera>>, mut commands: Commands) {
    commands.entity(*camera).insert(Light2d {
        ambient_light: AmbientLight2d::default(),
    });
}

/// Reset [`Light2d`] attached to [`CanvasCamera`].
fn reset_ambient(mut light: Single<&mut Light2d, With<CanvasCamera>>) {
    light.ambient_light = AmbientLight2d::default();
}

/// Interval in seconds to run logic in [`update_ambient_brightness`].
const UPDATE_INTERVAL_SECS: f32 = 5.;
/// Minimum [`AmbientLight2d::brightness`].
const MIN_AMBIENT: f32 = 0.1;
/// Maximum [`AmbientLight2d::brightness`].
const MAX_AMBIENT: f32 = 0.6;

/// Update [`AmbientLight2d::brightness`] from a linear [`EasingCurve`].
///
/// This is to simulate Day/Night cycle.
fn update_ambient_brightness(
    mut light: Single<&mut Light2d, With<CanvasCamera>>,
    timer: Res<DayTimer>,
    mut last_update: Local<f32>,
) {
    // Return if not on correct update interval
    if timer.0.elapsed_secs() - *last_update < UPDATE_INTERVAL_SECS {
        return;
    }

    let brightness = EasingCurve::new(MIN_AMBIENT, MAX_AMBIENT, EaseFunction::Linear)
        .ping_pong()
        .expect(ERR_INVALID_DOMAIN_EASING);
    // NOTE: We are multiplying by 2 since `PingPongCurve` has a domain from 0 to 2.
    let brightness = brightness.sample_clamped(timer.0.fraction() * 2.);
    light.ambient_light.brightness = brightness;

    *last_update = timer.0.elapsed_secs();
}

/// Tick [`DayTimer`]
fn tick_day_timer(time: Res<Time>, mut timer: ResMut<DayTimer>) {
    timer.0.tick(time.delta());
}
