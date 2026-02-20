/*
 * File: splash.rs
 * Author: Leopold Johannes Meinel (leo@meinel.dev)
 * -----
 * Copyright (c) 2026 Leopold Johannes Meinel & contributors
 * SPDX ID: Apache-2.0
 * URL: https://www.apache.org/licenses/LICENSE-2.0
 * -----
 * Heavily inspired by: https://github.com/TheBevyFlock/bevy_new_2d
 */

//! A splash screen that plays briefly at startup.

use bevy::{input::common_conditions::input_just_pressed, prelude::*};
use bevy_asset_loader::prelude::*;

use crate::{
    AppSystems,
    screens::Screen,
    ui::{palette::*, prelude::*},
};

pub(super) fn plugin(app: &mut App) {
    // Insert resources
    app.insert_resource(ClearColor(CLEAR_BACKGROUND.into()));

    // Open splash screen
    app.add_systems(OnEnter(Screen::Splash), spawn_splash_screen);

    // Exit splash screen early on pressing Escape
    app.add_systems(
        Update,
        enter_title_screen
            .run_if(input_just_pressed(KeyCode::Escape).and(in_state(Screen::Splash))),
    );

    // Animate splash screen
    app.add_systems(
        Update,
        (
            tick_fade_in_out.in_set(AppSystems::TickTimers),
            apply_fade_in_out.in_set(AppSystems::Update),
        )
            .run_if(in_state(Screen::Splash)),
    );

    // Add splash timer
    app.add_systems(OnEnter(Screen::Splash), insert_splash_timer);
    app.add_systems(OnExit(Screen::Splash), remove_splash_timer);
    app.add_systems(
        Update,
        (
            tick_splash_timer.in_set(AppSystems::TickTimers),
            check_splash_timer.in_set(AppSystems::Update),
        )
            .run_if(in_state(Screen::Splash)),
    );
}

/// Assets for splash screen
#[derive(AssetCollection, Resource)]
pub(crate) struct SplashAssets {
    #[asset(path = "images/ui/splash.webp")]
    #[asset(image(sampler(filter = linear)))]
    splash: Handle<Image>,
}

/// Fading in and out of splash screen
#[derive(Component, Reflect)]
#[reflect(Component)]
struct ImageNodeFadeInOut {
    /// Total duration in seconds.
    total_duration: f32,
    /// Fade duration in seconds.
    fade_duration: f32,
    /// Current progress in seconds, between 0 and [`Self::total_duration`].
    t: f32,
}
impl ImageNodeFadeInOut {
    fn alpha(&self) -> f32 {
        // Normalize by duration.
        let t = (self.t / self.total_duration).clamp(0.0, 1.0);
        let fade = self.fade_duration / self.total_duration;

        // Regular trapezoid-shaped graph, flat at the top with alpha = 1.0.
        ((1.0 - (2.0 * t - 1.0).abs()) / fade).min(1.0)
    }
}

/// Timer that tracks splash screen
#[derive(Resource, Debug, Clone, PartialEq, Reflect)]
#[reflect(Resource)]
struct SplashTimer(Timer);
impl Default for SplashTimer {
    fn default() -> Self {
        Self(Timer::from_seconds(SPLASH_DURATION_SECS, TimerMode::Once))
    }
}

/// Default display duration of the splash screen
const SPLASH_DURATION_SECS: f32 = 1.8;
/// Fade duration of the splash screen
const SPLASH_FADE_DURATION_SECS: f32 = 0.6;

/// Spawn splash screen
fn spawn_splash_screen(mut commands: Commands, splash_assets: Res<SplashAssets>) {
    commands.spawn((
        widgets::ui_root("Splash Screen"),
        BackgroundColor(CLEAR_BACKGROUND.into()),
        DespawnOnExit(Screen::Splash),
        children![(
            Name::new("Splash image"),
            Node {
                margin: UiRect::all(auto()),
                width: percent(70),
                ..default()
            },
            ImageNode::new(splash_assets.splash.clone()),
            ImageNodeFadeInOut {
                total_duration: SPLASH_DURATION_SECS,
                fade_duration: SPLASH_FADE_DURATION_SECS,
                t: 0.0,
            },
        )],
    ));
}

/// Apply [`ImageNodeFadeInOut`]
fn apply_fade_in_out(mut query: Query<(&ImageNodeFadeInOut, &mut ImageNode)>) {
    for (anim, mut image) in &mut query {
        image.color.set_alpha(anim.alpha())
    }
}

/// Enter title screen
fn enter_title_screen(mut next_state: ResMut<NextState<Screen>>) {
    next_state.set(Screen::Title);
}

/// Check status of [`SplashTimer`]
fn check_splash_timer(timer: ResMut<SplashTimer>, mut next_state: ResMut<NextState<Screen>>) {
    if timer.0.just_finished() {
        next_state.set(Screen::Title);
    }
}

/// Initialize [`SplashTimer`]
fn insert_splash_timer(mut commands: Commands) {
    commands.init_resource::<SplashTimer>();
}

/// Remove [`SplashTimer`]
fn remove_splash_timer(mut commands: Commands) {
    commands.remove_resource::<SplashTimer>();
}

/// Tick [`ImageNodeFadeInOut`]
fn tick_fade_in_out(mut query: Query<&mut ImageNodeFadeInOut>, time: Res<Time>) {
    for mut anim in &mut query {
        anim.t += time.delta_secs();
    }
}

/// Tick [`SplashTimer`]
fn tick_splash_timer(time: Res<Time>, mut timer: ResMut<SplashTimer>) {
    timer.0.tick(time.delta());
}
