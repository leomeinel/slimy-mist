/*
 * File: animations.rs
 * Author: Leopold Johannes Meinel (leo@meinel.dev)
 * -----
 * Copyright (c) 2026 Leopold Johannes Meinel & contributors
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

use std::{marker::PhantomData, ops::Range};

use bevy::prelude::*;
use bevy_prng::WyRand;
use bevy_rand::{global::GlobalRng, traits::ForkableSeed as _};
use bevy_spritesheet_animation::prelude::*;
use rand::seq::IndexedRandom as _;

use crate::{
    AppSystems,
    audio::sound_effect,
    camera::ysort::{YSortCache, YSorted},
    characters::{Character, CharacterAssets, JUMP_DURATION_SECS, Movement, VisualMap},
    logging::{
        error::{
            ERR_INVALID_REQUIRED_ANIMATION_DATA, ERR_INVALID_TEXTURE_ATLAS, ERR_INVALID_VISUAL_MAP,
            ERR_NOT_LOADED_SPRITE_IMAGE, ERR_UNINITIALIZED_REQUIRED_ANIMATION,
        },
        warn::{WARN_INCOMPLETE_ANIMATION_DATA, WARN_INCOMPLETE_ASSET_DATA},
    },
};

pub(super) fn plugin(app: &mut App) {
    // Add rng for animations
    app.add_systems(Startup, setup_rng);

    // Add library plugins
    app.add_plugins(SpritesheetAnimationPlugin);

    // Tick animation timer
    app.add_systems(Update, tick_animation_timer.in_set(AppSystems::TickTimers));
}

/// Animation delay [`Range`] in seconds
pub(crate) const ANIMATION_DELAY_RANGE_SECS: Range<f32> = 0.0..10.0;

/// Animation data deserialized from a ron file as a generic.
///
/// ## Traits
///
/// - `T` must implement [`Character`].
#[derive(serde::Deserialize, Asset, TypePath, Default)]
pub(crate) struct AnimationData<T>
where
    T: Character,
{
    pub(crate) atlas_columns: usize,
    pub(crate) atlas_rows: usize,
    #[serde(default)]
    pub(crate) idle_row: Option<usize>,
    #[serde(default)]
    pub(crate) idle_frames: Option<usize>,
    #[serde(default)]
    pub(crate) idle_interval_ms: Option<u32>,
    #[serde(default)]
    pub(crate) walk_row: Option<usize>,
    #[serde(default)]
    pub(crate) walk_frames: Option<usize>,
    #[serde(default)]
    pub(crate) walk_interval_ms: Option<u32>,
    #[serde(default)]
    pub(crate) walk_sound_frames: Option<Vec<usize>>,
    #[serde(default)]
    pub(crate) jump_row: Option<usize>,
    #[serde(default)]
    pub(crate) jump_frames: Option<usize>,
    #[serde(default)]
    pub(crate) jump_sound_frames: Option<Vec<usize>>,
    #[serde(default)]
    pub(crate) fall_row: Option<usize>,
    #[serde(default)]
    pub(crate) fall_frames: Option<usize>,
    #[serde(default)]
    pub(crate) fall_sound_frames: Option<Vec<usize>>,
    #[serde(skip)]
    pub(crate) _phantom: PhantomData<T>,
}

/// Handle for [`AnimationData`] as a generic
///
/// ## Traits
///
/// - `T` must implement [`Character`].
#[derive(Resource)]
pub(crate) struct AnimationHandle<T>(pub(crate) Handle<AnimationData<T>>)
where
    T: Character;

/// Cache for [`AnimationData`]
///
/// This is to allow easier access.
///
/// ## Traits
///
/// - `T` must implement [`Character`].
#[derive(Resource, Default)]
pub(crate) struct AnimationDataCache<T>
where
    T: Character,
{
    pub(crate) atlas_columns: usize,
    pub(crate) atlas_rows: usize,
    pub(crate) idle_row: Option<usize>,
    pub(crate) idle_frames: Option<usize>,
    pub(crate) idle_interval_ms: Option<u32>,
    pub(crate) walk_row: Option<usize>,
    pub(crate) walk_frames: Option<usize>,
    pub(crate) walk_interval_ms: Option<u32>,
    pub(crate) walk_sound_frames: Option<Vec<usize>>,
    pub(crate) jump_row: Option<usize>,
    pub(crate) jump_frames: Option<usize>,
    pub(crate) jump_sound_frames: Option<Vec<usize>>,
    pub(crate) fall_row: Option<usize>,
    pub(crate) fall_frames: Option<usize>,
    pub(crate) fall_sound_frames: Option<Vec<usize>>,
    pub(crate) _phantom: PhantomData<T>,
}

/// Animations with generics
///
/// This serves as the main interface for other modules
///
/// ## Traits
///
/// - `T` must implement [`Character`].
#[derive(Resource, Default)]
pub(crate) struct Animations<T>
where
    T: Character,
{
    pub(crate) sprite: Sprite,
    pub(crate) idle: Handle<Animation>,
    pub(crate) walk: Option<Handle<Animation>>,
    pub(crate) jump: Option<Handle<Animation>>,
    pub(crate) fall: Option<Handle<Animation>>,
    _phantom: PhantomData<T>,
}

/// Current state of animation
#[derive(Default, Clone, Copy, PartialEq, Debug)]
pub(crate) enum AnimationState {
    #[default]
    Idle,
    Walk,
    Jump,
    Fall,
}

/// Controller for animations
#[derive(Component, Default)]
pub(crate) struct AnimationController {
    /// Used to determine next animation
    pub(crate) state: AnimationState,
    /// Used to determine if we should play sound again
    pub(crate) sound_frame: Option<usize>,
}
impl AnimationController {
    /// Sets a new [`AnimationState`] if it has not already been set.
    pub(crate) fn set_new_state(&mut self, new_state: AnimationState) {
        if self.state != new_state {
            self.state = new_state;
        }
    }
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
///
/// ## Traits
///
/// - `T` must implement [`Character`] and [`YSorted`].
/// - `A` must implement [`CharacterAssets`]
pub(crate) fn setup_animations<T, A>(
    mut commands: Commands,
    mut atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
    mut global_animations: ResMut<Assets<Animation>>,
    animation_data: Res<AnimationDataCache<T>>,
    assets: Res<A>,
    images: Res<Assets<Image>>,
) where
    T: Character + YSorted,
    A: CharacterAssets,
{
    // Set sprite sheet and generate sprite from it
    let sprite_sheet = Spritesheet::new(
        assets.get_image(),
        animation_data.atlas_columns,
        animation_data.atlas_rows,
    );
    let sprite = sprite_sheet
        .with_loaded_image(&images)
        .expect(ERR_NOT_LOADED_SPRITE_IMAGE)
        .sprite(&mut atlas_layouts);

    // Idle animation: This is the only required animation
    let idle = animation_handle(
        &mut global_animations,
        &sprite_sheet,
        animation_data.idle_row,
        animation_data.idle_frames,
        animation_data.idle_interval_ms,
        AnimationRepeat::Loop,
    )
    .expect(ERR_INVALID_REQUIRED_ANIMATION_DATA);

    // Walk animation
    let walk = animation_handle(
        &mut global_animations,
        &sprite_sheet,
        animation_data.walk_row,
        animation_data.walk_frames,
        animation_data.walk_interval_ms,
        AnimationRepeat::Loop,
    );

    // Jump animation
    let jump = animation_data
        .jump_frames
        .map(|frames| {
            let interval_ms = (JUMP_DURATION_SECS * 500. / frames.max(1) as f32) as u32;
            animation_handle(
                &mut global_animations,
                &sprite_sheet,
                animation_data.jump_row,
                animation_data.jump_frames,
                Some(interval_ms),
                AnimationRepeat::Times(1),
            )
        })
        .unwrap_or_else(|| None);

    // Fall animation
    let fall = animation_data
        .fall_frames
        .map(|frames| {
            let interval_ms = (JUMP_DURATION_SECS * 500. / frames.max(1) as f32) as u32;
            animation_handle(
                &mut global_animations,
                &sprite_sheet,
                animation_data.fall_row,
                animation_data.fall_frames,
                Some(interval_ms),
                AnimationRepeat::Times(1),
            )
        })
        .unwrap_or_else(|| None);

    // Data for `YSortCache`
    let sprite_layout_id = sprite
        .texture_atlas
        .as_ref()
        .expect(ERR_INVALID_TEXTURE_ATLAS)
        .layout
        .id();
    let texture_size = atlas_layouts
        .get(sprite_layout_id)
        .expect(ERR_INVALID_TEXTURE_ATLAS)
        .textures
        .first()
        .expect(ERR_INVALID_TEXTURE_ATLAS)
        .size();

    // Init `Animations`
    commands.insert_resource(Animations::<T> {
        sprite,
        idle,
        walk,
        jump,
        fall,
        ..default()
    });
    // Init `YSortCache`
    commands.insert_resource(YSortCache::<T> {
        texture_size,
        ..default()
    });
}

/// Animation handle customized via parameters
///
/// Returns [`Some`] for valid parameters
/// Returns [`None`] for invalid `row`, `frames` or `interval` parameter
fn animation_handle(
    global_animations: &mut ResMut<Assets<Animation>>,
    sprite_sheet: &Spritesheet,
    row: Option<usize>,
    frames: Option<usize>,
    interval: Option<u32>,
    repetitions: AnimationRepeat,
) -> Option<Handle<Animation>> {
    let (Some(row), Some(frames), Some(interval)) = (row, frames, interval) else {
        warn_once!(WARN_INCOMPLETE_ANIMATION_DATA);
        return None;
    };

    if frames < 1 {
        return None;
    }

    Some(
        global_animations.add(
            sprite_sheet
                .create_animation()
                .add_horizontal_strip(0, row, frames)
                .set_clip_duration(AnimationDuration::PerFrame(interval))
                .set_repetitions(repetitions)
                .build(),
        ),
    )
}

/// Tick animation timer
pub(crate) fn tick_animation_timer(mut query: Query<&mut AnimationTimer>, time: Res<Time>) {
    for mut timer in &mut query {
        timer.0.tick(time.delta());
    }
}

/// Update animations
///
/// ## Traits
///
/// - `T` must implement [`Character`].
pub(crate) fn update_animations<T>(
    parent_query: Query<(Entity, &mut Movement), With<T>>,
    mut child_query: Query<
        (
            &mut AnimationController,
            &mut Sprite,
            &mut SpritesheetAnimation,
            &AnimationTimer,
        ),
        Without<T>,
    >,
    animations: Res<Animations<T>>,
    visual_map: Res<VisualMap>,
) where
    T: Character,
{
    for (entity, movement) in &parent_query {
        // Extract `animation_controller` from `child_query`
        let visual = visual_map.0.get(&entity).expect(ERR_INVALID_VISUAL_MAP);
        let (mut controller, mut sprite, mut animation, timer) =
            child_query.get_mut(*visual).expect(ERR_INVALID_VISUAL_MAP);

        // Reset animation after timer has finished
        if timer.0.just_finished() {
            animation.reset();
        }

        // Sprite flipping
        let dx = movement.direction.x;
        if dx != 0. {
            sprite.flip_x = dx < 0.;
        }

        // Match to current `AnimationState`
        match controller.state {
            AnimationState::Walk => {
                switch_to_new_animation(&mut animation, animations.walk.clone(), &mut controller)
            }
            AnimationState::Idle => switch_to_new_animation(
                &mut animation,
                Some(animations.idle.clone()),
                &mut controller,
            ),
            AnimationState::Jump => {
                switch_to_new_animation(&mut animation, animations.jump.clone(), &mut controller)
            }
            AnimationState::Fall => {
                switch_to_new_animation(&mut animation, animations.fall.clone(), &mut controller)
            }
        }
    }
}

/// Switches to new [`SpritesheetAnimation`] from [`Option<Handle<Animation>>`] if it has not already been switched to.
fn switch_to_new_animation(
    animation: &mut SpritesheetAnimation,
    new_animation: Option<Handle<Animation>>,
    controller: &mut AnimationController,
) {
    let new_animation = new_animation.expect(ERR_UNINITIALIZED_REQUIRED_ANIMATION);

    if animation.animation != new_animation {
        animation.switch(new_animation);
        controller.sound_frame = None;
    }
}

/// Update animation sounds
///
/// ## Traits
///
/// - `T` must implement [`Character`].
/// - `A` must implement [`CharacterAssets`]
pub(crate) fn update_animation_sounds<T, A>(
    mut rng: Single<&mut WyRand, With<AnimationRng>>,
    parent_query: Query<Entity, With<T>>,
    mut child_query: Query<(&mut AnimationController, &mut SpritesheetAnimation), Without<T>>,
    mut commands: Commands,
    animation_data: Res<AnimationDataCache<T>>,
    visual_map: Res<VisualMap>,
    assets: Res<A>,
) where
    T: Character,
    A: CharacterAssets,
{
    let frame_set = (
        animation_data.walk_sound_frames.clone(),
        animation_data.jump_sound_frames.clone(),
        animation_data.fall_sound_frames.clone(),
    );

    for entity in &parent_query {
        // Extract `animation_controller` from `child_query`
        let visual = visual_map.0.get(&entity).expect(ERR_INVALID_VISUAL_MAP);
        let (mut controller, animation) =
            child_query.get_mut(*visual).expect(ERR_INVALID_VISUAL_MAP);

        // Continue if sound has already been played
        if let Some(sound_frame) = controller.sound_frame
            && sound_frame == animation.progress.frame
        {
            continue;
        }

        // Match to current `AnimationState`
        let Some(sound) = (match controller.state {
            AnimationState::Walk => choose_sound(
                &mut rng,
                &animation.progress.frame,
                &frame_set.0,
                assets.get_walk_sounds(),
            ),
            AnimationState::Jump => choose_sound(
                &mut rng,
                &animation.progress.frame,
                &frame_set.1,
                assets.get_jump_sounds(),
            ),
            AnimationState::Fall => choose_sound(
                &mut rng,
                &animation.progress.frame,
                &frame_set.2,
                assets.get_fall_sounds(),
            ),
            _ => None,
        }) else {
            controller.sound_frame = None;
            continue;
        };

        // Play sound
        commands.spawn(sound_effect(sound));
        controller.sound_frame = Some(animation.progress.frame);
    }
}

/// Choose a random customized via parameters for current frame.
///
/// Returns [`Some`] if current frame is a fall sound frame.
/// Returns [`None`] if current frame is not a fall sound frame or on missing data.
fn choose_sound(
    rng: &mut WyRand,
    current_frame: &usize,
    frames: &Option<Vec<usize>>,
    sounds: &Option<Vec<Handle<AudioSource>>>,
) -> Option<Handle<AudioSource>> {
    // Return `None` if frame data is missing or does not contain current frame
    let Some(frames) = frames else {
        warn_once!("{}", WARN_INCOMPLETE_ANIMATION_DATA);
        return None;
    };
    if !frames.contains(current_frame) {
        return None;
    }

    // Return none if asset data is missing
    let Some(sounds) = sounds else {
        warn_once!("{}", WARN_INCOMPLETE_ASSET_DATA);
        return None;
    };

    sounds.choose(rng).cloned()
}
