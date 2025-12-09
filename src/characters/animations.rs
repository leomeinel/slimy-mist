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
use bevy_rapier2d::prelude::*;
use bevy_spritesheet_animation::prelude::*;
use rand::seq::IndexedRandom as _;

use crate::{audio::sound_effect, characters::CharacterAssets};

pub(super) fn plugin(app: &mut App) {
    // Add rng for animations
    app.add_systems(Startup, setup_rng);

    // Add child plugins
    app.add_plugins((npc::plugin, player::plugin));
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
    pub(crate) run: Handle<Animation>,
    _phantom: PhantomData<T>,
}

/// Rng for animations
#[derive(Component)]
pub(crate) struct Rng;

/// Spawn [`Rng`] by forking [`GlobalRng`]
fn setup_rng(mut global: Single<&mut WyRand, With<GlobalRng>>, mut commands: Commands) {
    commands.spawn((Rng, global.fork_seed()));
}

/// Setup the [`Animations`] struct and add animations
fn setup<T, A>(
    mut commands: Commands,
    animation_data: Res<Assets<AnimationData<T>>>,
    mut atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
    mut global_animations: ResMut<Assets<Animation>>,
    animation_handle: Res<AnimationHandle<T>>,
    assets: Res<A>,
    images: Res<Assets<Image>>,
) where
    T: Component + Default + Reflectable,
    A: CharacterAssets + Resource,
{
    // Get animation from `AnimationData` with `AnimationHandle`
    let Some(animation_data) = animation_data.get(animation_handle.0.id()) else {
        return;
    };
    // Set sprite sheet and generate sprite from it
    let sprite_sheet = Spritesheet::new(
        &assets.get_image().clone(),
        animation_data.atlas_columns,
        animation_data.atlas_rows,
    );
    let sprite = sprite_sheet
        .with_loaded_image(&images)
        .unwrap()
        .sprite(&mut atlas_layouts);

    // Idle animation
    let idle_animation = sprite_sheet
        .create_animation()
        .add_horizontal_strip(0, 0, animation_data.idle_frames)
        .set_clip_duration(AnimationDuration::PerFrame(animation_data.idle_interval_ms))
        .set_repetitions(AnimationRepeat::Loop)
        .build();
    let idle = global_animations.add(idle_animation);

    // Run animation
    let run_animation = sprite_sheet
        .create_animation()
        .add_horizontal_strip(0, 1, animation_data.move_frames)
        .set_clip_duration(AnimationDuration::PerFrame(animation_data.move_interval_ms))
        .set_repetitions(AnimationRepeat::Loop)
        .build();
    let run = global_animations.add(run_animation);

    // Add to `Animations`
    commands.insert_resource(Animations::<T> {
        sprite,
        idle,
        run,
        ..default()
    });
}

/// Update animations
fn update<T>(
    mut query: Query<
        (
            &KinematicCharacterController,
            &mut Sprite,
            &mut SpritesheetAnimation,
        ),
        With<T>,
    >,
    animations: Res<Animations<T>>,
) where
    T: Component,
{
    for (controller, mut sprite, mut animation) in &mut query {
        let Some(intent) = controller.translation else {
            continue;
        };

        // If not moving, switch to idle and continue
        if intent == Vec2::ZERO && animation.animation != animations.idle {
            animation.switch(animations.idle.clone());
            continue;
        }

        // Sprite flipping
        let dx = intent.x;
        if dx != 0. {
            sprite.flip_x = dx < 0.;
        }

        // Run animation
        if animation.animation != animations.run {
            animation.switch(animations.run.clone());
        }
    }
}

/// Update animation sounds
fn update_sound<T, A>(
    mut rng: Single<&mut WyRand, With<Rng>>,
    mut query: Query<&mut SpritesheetAnimation, With<T>>,
    mut commands: Commands,
    animation_data: Res<Assets<AnimationData<T>>>,
    animation_handle: Res<AnimationHandle<T>>,
    animations: Res<Animations<T>>,
    assets: If<Res<A>>,
) where
    T: Component + Default + Reflectable,
    A: CharacterAssets + Resource,
{
    // Get animation from `AnimationData` with `AnimationHandle`
    let Some(animation_data) = animation_data.get(animation_handle.0.id()) else {
        return;
    };

    for animation in &mut query {
        // Continue if animation is not run or we are not on the correct frame
        if animation.animation != animations.run
            || !animation_data
                .step_sound_frames
                .contains(&animation.progress.frame)
        {
            continue;
        }

        // Play random step sound
        let step_sound = assets
            .get_step_sounds()
            .choose(rng.as_mut())
            .unwrap()
            .clone();
        commands.spawn(sound_effect(step_sound));
    }
}
