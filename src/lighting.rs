/*
 * File: lighting.rs
 * Author: Leopold Johannes Meinel (leo@meinel.dev)
 * -----
 * Copyright (c) 2026 Leopold Johannes Meinel & contributors
 * SPDX ID: Apache-2.0
 * URL: https://www.apache.org/licenses/LICENSE-2.0
 */

// FIXME: For some reason, even though all point lights are despawned, their lighting still affects
//        the splash screen after exiting Gameplay.
//        Also see: https://github.com/malbernaz/bevy_lit/issues/24

use bevy::{color::palettes::tailwind, prelude::*};
use bevy_lit::prelude::*;

use crate::{
    AppSystems, PausableSystems,
    camera::{CanvasCamera, FOREGROUND_Z, ysort::YSort},
    logging::error::ERR_INVALID_DOMAIN_EASING,
    procgen::ProcGenerated,
    screens::Screen,
    visual::Visible,
};

pub(super) fn plugin(app: &mut App) {
    // Add ambient light after entering `Screen::Gameplay` and reset when exiting.
    app.add_systems(OnEnter(Screen::Gameplay), add_settings);
    app.add_systems(OnExit(Screen::Gameplay), remove_settings);

    // Update ambient intensity to simulate a Day/Night cycle.
    app.add_systems(
        Update,
        update_ambient_intensity
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
                Transform::from_translation(pos.extend(FOREGROUND_Z)),
                YSort(FOREGROUND_Z),
                Visibility::Inherited,
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
            intensity: 2.,
            outer_radius: 128.,
            falloff: 4.,
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

/// Insert [`Lighting2dSettings`] into [`CanvasCamera`].
fn add_settings(camera: Single<Entity, With<CanvasCamera>>, mut commands: Commands) {
    commands
        .entity(*camera)
        .insert(Lighting2dSettings::default());
}

/// Remove [`Lighting2dSettings`] and [`AmbientLight2d`] attached to [`CanvasCamera`].
fn remove_settings(
    light: Single<(&mut Lighting2dSettings, &mut AmbientLight2d), With<CanvasCamera>>,
) {
    let (mut settings, mut ambient) = light.into_inner();
    *settings = Lighting2dSettings::default();
    *ambient = AmbientLight2d::default();
}

/// Interval in seconds to run logic in [`update_ambient_intensity`].
const UPDATE_AMBIENT_INTERVAL_SECS: f32 = 5.;
/// Minimum [`AmbientLight2d::intensity`].
const MIN_AMBIENT: f32 = 0.1;
/// Maximum [`AmbientLight2d::intensity`].
const MAX_AMBIENT: f32 = 0.6;

/// Update [`AmbientLight2d::intensity`] from a linear [`EasingCurve`].
///
/// This is to simulate a Day/Night cycle.
fn update_ambient_intensity(
    mut light: Single<&mut AmbientLight2d, With<CanvasCamera>>,
    timer: Res<DayTimer>,
    mut last_update: Local<Option<f32>>,
) {
    // Restrict to run only on `UPDATE_INTERVAL_SECS`
    let elapsed_secs = timer.0.elapsed_secs();
    if let Some(inner) = *last_update {
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
