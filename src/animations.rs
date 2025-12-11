/*
 * File: animations.rs
 * Author: Leopold Johannes Meinel (leo@meinel.dev)
 * -----
 * Copyright (c) 2025 Leopold Johannes Meinel & contributors
 * SPDX ID: Apache-2.0
 * URL: https://www.apache.org/licenses/LICENSE-2.0
 * -----
 * Heavily inspired by:
 * - https://github.com/NiklasEi/bevy_common_assets/tree/main
 * - https://github.com/merwaaan/bevy_spritesheet_animation
 */

//! Player sprite animation.
//! This is based on multiple examples and may be very different for your game.
//! - [Sprite flipping](https://github.com/bevyengine/bevy/blob/latest/examples/2d/sprite_flipping.rs)
//! - [Sprite animation](https://github.com/bevyengine/bevy/blob/latest/examples/2d/sprite_animation.rs)
//! - [Timers](https://github.com/bevyengine/bevy/blob/latest/examples/time/timers.rs)

pub(crate) mod npc;
pub(crate) mod player;

use std::marker::PhantomData;

use bevy::{prelude::*, reflect::Reflectable};
use bevy_prng::WyRand;
use bevy_rand::{global::GlobalRng, traits::ForkableSeed as _};
use bevy_spritesheet_animation::prelude::*;
use rand::seq::IndexedRandom as _;

use crate::{AppSystems, audio::sound_effect, characters::CharacterAssets};

pub(super) fn plugin(app: &mut App) {
    // Add rng for animations
    app.add_systems(Startup, setup_rng);

    // Add child plugins
    app.add_plugins((npc::plugin, player::plugin));

    // Add plugin for sprite animation
    app.add_plugins(SpritesheetAnimationPlugin);

    // Tick animation timer
    app.add_systems(Update, tick_animation_timer.in_set(AppSystems::TickTimers));
}

/// Animation data deserialized from a ron file as a generic
#[derive(serde::Deserialize, Asset, TypePath)]
struct AnimationData<T>
where
    T: Reflectable,
{
    atlas_columns: usize,
    atlas_rows: usize,
    idle_frames: usize,
    idle_interval_ms: u32,
    move_frames: usize,
    move_interval_ms: u32,
    step_sound_frames: Vec<usize>,
    #[serde(skip)]
    _phantom: PhantomData<T>,
}

/// Handle for [`AnimationData`] as a generic
#[derive(Resource)]
struct AnimationHandle<T>(Handle<AnimationData<T>>)
where
    T: Reflectable;

/// Animations with generics
///
/// This serves as the main interface for other modules
#[derive(Resource, Default)]
pub(crate) struct Animations<T> {
    pub(crate) sprite: Sprite,
    pub(crate) idle: Handle<Animation>,
    pub(crate) movement: Handle<Animation>,
    _phantom: PhantomData<T>,
}

/// Current state of animation
#[derive(Default, Clone, Copy, PartialEq, Debug)]
pub(crate) enum AnimationState {
    #[default]
    Idle,
    Movement(Vec2),
    Jump,
    Fall,
}

/// Controller for animations
#[derive(Component, Default)]
pub(crate) struct AnimationController {
    /// Used to determine next animation
    pub(crate) state: AnimationState,
    /// Used to determine if we should play sound again
    pub(crate) sound_played: bool,
}

/// Timer that tracks animation
#[derive(Component, Debug, Clone, PartialEq, Reflect)]
#[reflect(Component)]
pub(crate) struct AnimationTimer(pub(crate) Timer);

/// Rng for animations
#[derive(Component)]
pub(crate) struct AnimationRng;

/// Spawn [`AnimationRng`] by forking [`GlobalRng`]
fn setup_rng(mut global: Single<&mut WyRand, With<GlobalRng>>, mut commands: Commands) {
    commands.spawn((AnimationRng, global.fork_seed()));
}

/// Setup the [`Animations`] struct and add animations
fn setup<T, A>(
    mut commands: Commands,
    data: Res<Assets<AnimationData<T>>>,
    mut atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
    mut global_animations: ResMut<Assets<Animation>>,
    handle: Res<AnimationHandle<T>>,
    assets: Res<A>,
    images: Res<Assets<Image>>,
) where
    T: Component + Default + Reflectable,
    A: CharacterAssets + Resource,
{
    // Get animation from `AnimationData` with `AnimationHandle`
    let Some(data) = data.get(handle.0.id()) else {
        return;
    };
    // Set sprite sheet and generate sprite from it
    let sprite_sheet = Spritesheet::new(
        &assets.get_image().clone(),
        data.atlas_columns,
        data.atlas_rows,
    );
    let sprite = sprite_sheet
        .with_loaded_image(&images)
        .unwrap()
        .sprite(&mut atlas_layouts);

    // Idle animation
    let idle_animation = sprite_sheet
        .create_animation()
        .add_horizontal_strip(0, 0, data.idle_frames)
        .set_clip_duration(AnimationDuration::PerFrame(data.idle_interval_ms))
        .set_repetitions(AnimationRepeat::Loop)
        .build();
    let idle = global_animations.add(idle_animation);

    // Movement animation
    let movement_animation = sprite_sheet
        .create_animation()
        .add_horizontal_strip(0, 1, data.move_frames)
        .set_clip_duration(AnimationDuration::PerFrame(data.move_interval_ms))
        .set_repetitions(AnimationRepeat::Loop)
        .build();
    let movement = global_animations.add(movement_animation);

    // Add to `Animations`
    commands.insert_resource(Animations::<T> {
        sprite,
        idle,
        movement,
        ..default()
    });
}

/// Update animations
fn update<T>(
    mut query: Query<
        (
            &AnimationController,
            &mut Sprite,
            &mut SpritesheetAnimation,
            &AnimationTimer,
        ),
        With<T>,
    >,
    animations: Res<Animations<T>>,
) where
    T: Component,
{
    for (controller, mut sprite, mut animation, timer) in &mut query {
        // Continue if timer is not finished
        if !timer.0.is_finished() {
            continue;
        }

        // Reset animation after timer has finished
        if timer.0.just_finished() {
            animation.reset();
        }

        // Set translation to desired translation because we even want to animate if walking against a wall
        let desired_animation = &controller.state;

        // Match to current `AnimationState`
        match desired_animation {
            AnimationState::Movement(translation) => {
                // Set speed factor to vector length
                animation.speed_factor = translation.length();

                if animation.animation == animations.movement {
                    // Sprite flipping
                    let dx = translation.x;
                    if dx != 0. {
                        sprite.flip_x = dx < 0.;
                    }
                    continue;
                }

                // Switch to movement animation
                animation.switch(animations.movement.clone())
            }
            _ => {
                // Reset speed factor
                animation.speed_factor = 1.;

                // Match again, this avoids code duplication
                match desired_animation {
                    AnimationState::Idle => {
                        if animation.animation == animations.idle {
                            continue;
                        }

                        // Switch to idle animation
                        animation.switch(animations.idle.clone());
                    }
                    AnimationState::Jump => {
                        if animation.animation == animations.idle {
                            continue;
                        }

                        // Switch to jump animation
                        animation.switch(animations.idle.clone());
                    }
                    AnimationState::Fall => {
                        if animation.animation == animations.idle {
                            continue;
                        }

                        // Switch to fall animation
                        animation.switch(animations.idle.clone());
                    }
                    _ => unreachable!(),
                }
            }
        }
    }
}

/// Update animation sounds
fn update_sound<T, A>(
    mut rng: Single<&mut WyRand, With<AnimationRng>>,
    mut query: Query<(&mut AnimationController, &mut SpritesheetAnimation), With<T>>,
    mut commands: Commands,
    data: Res<Assets<AnimationData<T>>>,
    handle: Res<AnimationHandle<T>>,
    animations: Res<Animations<T>>,
    assets: If<Res<A>>,
) where
    T: Component + Default + Reflectable,
    A: CharacterAssets + Resource,
{
    // Get animation from `AnimationData` with `AnimationHandle`
    let Some(data) = data.get(handle.0.id()) else {
        return;
    };

    for (mut controller, animation) in &mut query {
        // Continue if animation is not movement or we are not on the correct frame
        if animation.animation != animations.movement
            || !data.step_sound_frames.contains(&animation.progress.frame)
        {
            // Reset sound_played
            controller.sound_played = false;
            continue;
        }

        // Continue if sound has already been played
        if controller.sound_played {
            continue;
        }

        // Play random step sound
        let step_sound = assets
            .get_step_sounds()
            .choose(rng.as_mut())
            .unwrap()
            .clone();
        commands.spawn(sound_effect(step_sound));

        // Set sound_playeds
        controller.sound_played = true;
    }
}

/// Tick animation timer
fn tick_animation_timer(mut query: Query<&mut AnimationTimer>, time: Res<Time>) {
    for mut timer in &mut query {
        timer.0.tick(time.delta());
    }
}
