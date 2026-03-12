/*
 * File: light.rs
 * Author: Leopold Johannes Meinel (leo@meinel.dev)
 * -----
 * Copyright (c) 2026 Leopold Johannes Meinel & contributors
 * SPDX ID: Apache-2.0
 * URL: https://www.apache.org/licenses/LICENSE-2.0
 */

use bevy::{color::palettes::tailwind, prelude::*};
use bevy_fast_light::prelude::*;

use crate::{
    AppSystems, PausableSystems,
    camera::{CanvasCamera, LIGHT_Z},
    log::error::*,
    procgen::ProcGenerated,
    screens::Screen,
    visual::Visible,
};

pub(super) fn plugin(app: &mut App) {
    // Reset ambient light after exiting `Screen::Gameplay`.
    app.add_systems(OnExit(Screen::Gameplay), reset_ambient);

    // Update ambient brightness to simulate a Day/Night cycle.
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

/// Wrapper for lights.
pub(crate) trait LightWrapper
where
    Self: Component + Default + Clone,
{
    type Inner: Component;
    fn into_inner(self) -> Self::Inner;
    fn spawn(&self, commands: &mut Commands, pos: Vec2) -> Entity {
        commands
            .spawn((
                // FIXME: Add sprite with animated flame that simulates a light.
                // FIXME: Having self.clone() here twice seems unnecessary.
                //        The problem is that we need the marker `T` and the wrapped light.
                self.clone(),
                self.clone().into_inner(),
                Transform::from_translation(pos.extend(LIGHT_Z)),
            )) //
            .id()
    }
}

/// Light that is attached to a street lamp.
#[derive(Component, Reflect, Clone)]
pub(crate) struct StreetLight(PointLight2d);
impl Default for StreetLight {
    fn default() -> Self {
        Self(PointLight2d {
            color: tailwind::AMBER_500.into(),
            inner_radius: 16.,
            outer_radius: 64.,
            ..default()
        })
    }
}
impl LightWrapper for StreetLight {
    type Inner = PointLight2d;
    fn into_inner(self) -> Self::Inner {
        self.0
    }
}
impl ProcGenerated for StreetLight {}
impl Visible for StreetLight {}

/// Seconds in a day.
const DAY_SECS: f32 = 600.;

/// Timer that tracks splash screen
#[derive(Resource, Debug, Clone, PartialEq, Reflect)]
#[reflect(Resource)]
pub(crate) struct DayTimer(Timer);
impl Default for DayTimer {
    fn default() -> Self {
        Self(Timer::from_seconds(DAY_SECS, TimerMode::Repeating))
    }
}

/// Reset [`AmbientLight2d`] in [`CanvasCamera`].
fn reset_ambient(camera: Single<Entity, With<CanvasCamera>>, mut commands: Commands) {
    commands.entity(*camera).insert(AmbientLight2d::default());
}

/// Interval in seconds to run logic in [`update_ambient_brightness`].
const UPDATE_AMBIENT_INTERVAL_SECS: f32 = 5.;
/// Minimum [`AmbientLight2d::intensity`].
const MIN_AMBIENT: f32 = 0.1;
/// Maximum [`AmbientLight2d::intensity`].
const MAX_AMBIENT: f32 = 0.5;

/// Update [`AmbientLight2d::intensity`] from [`EaseFunction::SmootherStep`].
///
/// This is to simulate a Day/Night cycle.
fn update_ambient_brightness(
    mut light: Single<&mut AmbientLight2d, With<CanvasCamera>>,
    timer: Res<DayTimer>,
    mut last_update: Local<Option<f32>>,
) {
    // Restrict to run only on `UPDATE_INTERVAL_SECS`
    let elapsed_secs = timer.0.elapsed_secs();
    // FIXME: Checking elapsed_secs > 1. is a hack to make sure this triggers when reentering gameplay.
    if elapsed_secs > 1.
        && let Some(inner) = *last_update
    {
        if inner > elapsed_secs {
            *last_update = None;
            return;
        }
        if elapsed_secs - inner < UPDATE_AMBIENT_INTERVAL_SECS {
            return;
        }
    }

    // NOTE: Using `SmootherStep` here is based on an approximation of the Clear-sky irradiance from https://re.jrc.ec.europa.eu/pvg_tools/en/#DR.
    //       It does not match the Clear-sky irradiance exactly but mimics it good enough for a game.
    let intensity = EasingCurve::new(MAX_AMBIENT, MIN_AMBIENT, EaseFunction::SmootherStep)
        .ping_pong()
        .expect(ERR_INVALID_DOMAIN_EASING);
    // NOTE: We are multiplying by 2 since `PingPongCurve` has a domain from 0 to 2.
    let intensity = intensity.sample_clamped(timer.0.fraction() * 2.);
    light.intensity = intensity;

    *last_update = Some(elapsed_secs);
}

/// Tick [`DayTimer`]
fn tick_day_timer(time: Res<Time>, mut timer: ResMut<DayTimer>) {
    timer.0.tick(time.delta());
}
