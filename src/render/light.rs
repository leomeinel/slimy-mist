use std::marker::PhantomData;

use bevy::{color::palettes::tailwind, prelude::*};
use bevy_fast_light::prelude::*;

use crate::{
    core::prelude::*, log::prelude::*, procgen::prelude::*, render::prelude::*,
    screens::prelude::*, utils::prelude::*,
};

pub(super) struct LightPlugin;
impl Plugin for LightPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(FastLightPlugin::default());

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
            (
                tick_resource_timer::<DayTimer>,
                tick_resource_timer::<DayUpdateTimer>,
            )
                .in_set(AppSystems::TickTimers)
                .run_if(in_state(Screen::Gameplay))
                .in_set(PausableSystems),
        );
    }
}

/// Wrapper for lights.
pub(crate) trait LightWrapper
where
    Self: Component + Default + Clone,
{
    type Inner: Bundle;
    fn into_inner(self) -> Self::Inner;
    fn new(mesh: Handle<Mesh>) -> Self;
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

/// [`Handle<Mesh>`] for light `T`.
#[derive(Resource, Default)]
pub(crate) struct LightMeshHandle<T>
where
    T: LightWrapper,
{
    pub(crate) handle: Handle<Mesh>,
    _phantom: PhantomData<T>,
}
impl<T> LightMeshHandle<T>
where
    T: LightWrapper,
{
    pub(crate) fn new(handle: Handle<Mesh>) -> Self {
        Self {
            handle,
            ..default()
        }
    }
}

/// Light that is attached to a street lamp.
#[derive(Component, Reflect, Clone, Default)]
pub(crate) struct StreetLight((MeshLight2d, Mesh2d));
impl StreetLight {
    pub(crate) fn primitive() -> Circle {
        Circle::new(96.)
    }
}
impl LightWrapper for StreetLight {
    type Inner = (MeshLight2d, Mesh2d);
    fn new(mesh: Handle<Mesh>) -> Self {
        Self((
            MeshLight2d {
                color: tailwind::AMBER_500.into(),
                ..default()
            },
            Mesh2d(mesh),
        ))
    }
    fn into_inner(self) -> Self::Inner {
        self.0
    }
}
impl ProcGenerated for StreetLight {}
impl Visible for StreetLight {}

/// Seconds in a day.
const DAY_SECS: f32 = 600.;

/// Timer tracking progress of a day to simulate day/night cycle.
#[derive(Resource, Debug, Clone, PartialEq, Reflect, Deref, DerefMut)]
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
#[derive(Resource, Debug, Clone, PartialEq, Reflect, Deref, DerefMut)]
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
const MAX_AMBIENT: f32 = 0.8;

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
