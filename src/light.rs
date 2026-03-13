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
    app.add_systems(OnEnter(Screen::Gameplay), init_ambient);
    app.add_systems(OnExit(Screen::Gameplay), reset_ambient);

    app.add_systems(
        Update,
        update_ambient_intensity
            .run_if(in_state(Screen::Gameplay))
            .in_set(PausableSystems),
    );

    app.add_systems(
        Update,
        (tick_day_timer, tick_day_update_timer)
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
            outer_radius: 96.,
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

/// Timer tracking progress of a day to simulate day/night cycle.
#[derive(Resource, Debug, Clone, PartialEq, Reflect)]
#[reflect(Resource)]
pub(crate) struct DayTimer(Timer);
impl Default for DayTimer {
    fn default() -> Self {
        Self(Timer::from_seconds(DAY_SECS, TimerMode::Repeating))
    }
}

/// Interval in seconds to update ambient light.
const DAY_UPDATE_SECS: f32 = 5.;

/// Timer for updating [`AmbientLight2d`] to simulate day/night cycle.
#[derive(Resource, Debug, Clone, PartialEq, Reflect)]
#[reflect(Resource)]
pub(crate) struct DayUpdateTimer(Timer);
impl Default for DayUpdateTimer {
    fn default() -> Self {
        Self(Timer::from_seconds(DAY_UPDATE_SECS, TimerMode::Repeating))
    }
}

/// Minimum [`AmbientLight2d::intensity`].
const MIN_AMBIENT: f32 = 0.05;
/// Maximum [`AmbientLight2d::intensity`].
const MAX_AMBIENT: f32 = 0.5;

/// Initialize [`AmbientLight2d`].
fn init_ambient(mut ambient: Single<&mut AmbientLight2d, With<CanvasCamera>>) {
    ambient.intensity = MAX_AMBIENT;
}

/// Reset [`AmbientLight2d`].
fn reset_ambient(mut ambient: Single<&mut AmbientLight2d, With<CanvasCamera>>) {
    **ambient = AmbientLight2d::default();
}

/// Update [`AmbientLight2d::intensity`] from [`EaseFunction::SmootherStep`].
///
/// This is to simulate a Day/Night cycle.
fn update_ambient_intensity(
    mut light: Single<&mut AmbientLight2d, With<CanvasCamera>>,
    day_timer: Res<DayTimer>,
    day_update_timer: Res<DayUpdateTimer>,
) {
    if !day_update_timer.0.just_finished() {
        return;
    }

    // NOTE: Using `SmootherStep` here is based on an approximation of the Clear-sky irradiance from https://re.jrc.ec.europa.eu/pvg_tools/en/#DR.
    //       It does not match the Clear-sky irradiance exactly but mimics it good enough for a game.
    let intensity = EasingCurve::new(MAX_AMBIENT, MIN_AMBIENT, EaseFunction::SmootherStep)
        .ping_pong()
        .expect(ERR_INVALID_DOMAIN_EASING);
    // NOTE: We are multiplying by 2 since `PingPongCurve` has a domain from 0 to 2.
    let intensity = intensity.sample_clamped(day_timer.0.fraction() * 2.);
    light.intensity = intensity;
}

/// Tick [`DayTimer`].
fn tick_day_timer(time: Res<Time>, mut timer: ResMut<DayTimer>) {
    timer.0.tick(time.delta());
}

/// Tick [`DayUpdateTimer`].
fn tick_day_update_timer(time: Res<Time>, mut timer: ResMut<DayUpdateTimer>) {
    timer.0.tick(time.delta());
}
