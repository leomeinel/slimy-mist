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
    AppSystems, PausableSystems,
    audio::sound_effect,
    characters::{
        Character, CharacterAssets, JUMP_DURATION_SECS, Movement,
        npc::{Slime, SlimeAssets},
        player::{Player, PlayerAssets},
    },
    logging::{
        error::{
            ERR_INVALID_CHILDREN, ERR_INVALID_REQUIRED_ANIMATION_DATA, ERR_INVALID_TEXTURE_ATLAS,
            ERR_NOT_LOADED_SPRITE_IMAGE, ERR_UNINITIALIZED_REQUIRED_ANIMATION,
        },
        warn::{WARN_INCOMPLETE_ANIMATION_DATA, WARN_INCOMPLETE_ASSET_DATA},
    },
    screens::Screen,
    visual::{TextureInfoCache, Visible, layers::DisplayImage},
};

pub(super) fn plugin(app: &mut App) {
    // Add library plugins
    app.add_plugins(SpritesheetAnimationPlugin);

    // Add rng for animations
    app.add_systems(Startup, setup_rng);

    // Animation updates
    app.add_systems(
        Update,
        (
            flip_sprites::<Player>,
            flip_sprites::<Slime>,
            (
                update_animations::<Player>,
                update_animation_sounds::<Player, PlayerAssets>,
            )
                .chain(),
            (
                update_animations::<Slime>,
                update_animation_sounds::<Slime, SlimeAssets>,
            )
                .chain(),
        )
            .run_if(in_state(Screen::Gameplay))
            .in_set(AppSystems::Update)
            .in_set(PausableSystems),
    );

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
    pub(crate) idle_frames: Option<Vec<(usize, usize)>>,
    #[serde(default)]
    pub(crate) idle_interval_ms: Option<u32>,
    #[serde(default)]
    pub(crate) walk_frames: Option<Vec<(usize, usize)>>,
    #[serde(default)]
    pub(crate) walk_interval_ms: Option<u32>,
    #[serde(default)]
    pub(crate) walk_sound_frames: Option<Vec<usize>>,
    #[serde(default)]
    pub(crate) run_frames: Option<Vec<(usize, usize)>>,
    #[serde(default)]
    pub(crate) run_interval_ms: Option<u32>,
    #[serde(default)]
    pub(crate) run_sound_frames: Option<Vec<usize>>,
    #[serde(default)]
    pub(crate) jump_frames: Option<Vec<(usize, usize)>>,
    #[serde(default)]
    pub(crate) jump_sound_frames: Option<Vec<usize>>,
    #[serde(default)]
    pub(crate) fall_frames: Option<Vec<(usize, usize)>>,
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
    pub(crate) idle_frames: Option<Vec<(usize, usize)>>,
    pub(crate) idle_interval_ms: Option<u32>,
    pub(crate) walk_frames: Option<Vec<(usize, usize)>>,
    pub(crate) walk_interval_ms: Option<u32>,
    pub(crate) walk_sound_frames: Option<Vec<usize>>,
    // FIXME: We should use fields prefixed with `_`
    pub(crate) _run_frames: Option<Vec<(usize, usize)>>,
    pub(crate) _run_interval_ms: Option<u32>,
    pub(crate) _run_sound_frames: Option<Vec<usize>>,
    pub(crate) jump_frames: Option<Vec<(usize, usize)>>,
    pub(crate) jump_sound_frames: Option<Vec<usize>>,
    pub(crate) fall_frames: Option<Vec<(usize, usize)>>,
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
#[derive(Default, Clone, Copy, PartialEq, Reflect, Debug)]
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
/// - `T` must implement [`Character`] and [`Visible`].
pub(crate) fn setup_animations<T>(
    mut commands: Commands,
    mut atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
    mut global_animations: ResMut<Assets<Animation>>,
    animation_data: Res<AnimationDataCache<T>>,
    display_image: Res<DisplayImage<T>>,
    images: Res<Assets<Image>>,
) where
    T: Character + Visible,
{
    // Set sprite sheet and generate sprite from it
    let sprite_sheet = Spritesheet::new(
        &display_image.image,
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
        animation_data.idle_frames.as_ref(),
        animation_data.idle_interval_ms,
        AnimationRepeat::Loop,
    )
    .expect(ERR_INVALID_REQUIRED_ANIMATION_DATA);

    // Walk animation
    let walk = animation_handle(
        &mut global_animations,
        &sprite_sheet,
        animation_data.walk_frames.as_ref(),
        animation_data.walk_interval_ms,
        AnimationRepeat::Loop,
    );

    // Jump animation
    let jump = animation_data
        .jump_frames
        .as_ref()
        .map(|frames| {
            let interval_ms = (JUMP_DURATION_SECS * 500. / frames.len().max(1) as f32) as u32;
            animation_handle(
                &mut global_animations,
                &sprite_sheet,
                animation_data.jump_frames.as_ref(),
                Some(interval_ms),
                AnimationRepeat::Times(1),
            )
        })
        .unwrap_or_else(|| None);

    // Fall animation
    let fall = animation_data
        .fall_frames
        .as_ref()
        .map(|frames| {
            let interval_ms = (JUMP_DURATION_SECS * 500. / frames.len().max(1) as f32) as u32;
            animation_handle(
                &mut global_animations,
                &sprite_sheet,
                animation_data.fall_frames.as_ref(),
                Some(interval_ms),
                AnimationRepeat::Times(1),
            )
        })
        .unwrap_or_else(|| None);

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

    commands.insert_resource(TextureInfoCache::<T> {
        size: texture_size,
        ..default()
    });
    commands.insert_resource(Animations::<T> {
        sprite,
        idle,
        walk,
        jump,
        fall,
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
    frames: Option<&Vec<(usize, usize)>>,
    interval: Option<u32>,
    repetitions: AnimationRepeat,
) -> Option<Handle<Animation>> {
    let (Some(frames), Some(interval)) = (frames, interval) else {
        warn_once!("{}", WARN_INCOMPLETE_ANIMATION_DATA);
        return None;
    };
    if frames.is_empty() {
        return None;
    }

    Some(
        global_animations.add(
            sprite_sheet
                .create_animation()
                .add_cells(frames.clone())
                .set_clip_duration(AnimationDuration::PerFrame(interval))
                .set_repetitions(repetitions)
                .build(),
        ),
    )
}

// FIXME: We should also follow the aim direction in some scenarios.
/// Flip [`Sprite`]s
///
/// ## Traits
///
/// - `T` must implement [`Character`].
fn flip_sprites<T>(
    character_query: Query<(&Movement, &Children), With<T>>,
    mut sprite_query: Query<&mut Sprite, With<SpritesheetAnimation>>,
) where
    T: Character,
{
    for (movement, children) in character_query {
        let direction = movement.direction;
        if direction.x == 0. {
            continue;
        }

        let child = children
            .iter()
            .find(|e| sprite_query.contains(*e))
            .expect(ERR_INVALID_CHILDREN);
        let mut sprite = sprite_query.get_mut(child).expect(ERR_INVALID_CHILDREN);
        sprite.flip_x = direction.x < 0.;
    }
}

/// Update animations
///
/// ## Traits
///
/// - `T` must implement [`Character`].
fn update_animations<T>(
    character_query: Query<(&mut AnimationController, &AnimationTimer, &Children), With<T>>,
    mut animation_query: Query<&mut SpritesheetAnimation, With<SpritesheetAnimation>>,
    animations: Res<Animations<T>>,
) where
    T: Character,
{
    for (mut controller, timer, children) in character_query {
        let child = children
            .iter()
            .find(|e| animation_query.contains(*e))
            .expect(ERR_INVALID_CHILDREN);
        let mut animation = animation_query.get_mut(child).expect(ERR_INVALID_CHILDREN);

        // Reset animation after timer has finished
        if timer.0.just_finished() {
            animation.reset();
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
    character_query: Query<(&mut AnimationController, &Children), With<T>>,
    animation_query: Query<&mut SpritesheetAnimation>,
    mut commands: Commands,
    animation_data: Res<AnimationDataCache<T>>,
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

    for (mut controller, children) in character_query {
        let child = children
            .iter()
            .find(|e| animation_query.contains(*e))
            .expect(ERR_INVALID_CHILDREN);
        let animation = animation_query.get(child).expect(ERR_INVALID_CHILDREN);

        // Continue if sound has already been played
        let current_frame = animation.progress.frame;
        if let Some(sound_frame) = controller.sound_frame
            && sound_frame == current_frame
        {
            continue;
        }

        // Match to current `AnimationState`
        let Some(sound) = (match controller.state {
            AnimationState::Walk => {
                choose_sound(&mut rng, &current_frame, &frame_set.0, assets.walk_sounds())
            }
            AnimationState::Jump => {
                choose_sound(&mut rng, &current_frame, &frame_set.1, assets.jump_sounds())
            }
            AnimationState::Fall => {
                choose_sound(&mut rng, &current_frame, &frame_set.2, assets.fall_sounds())
            }
            _ => None,
        }) else {
            controller.sound_frame = None;
            continue;
        };

        // Play sound
        commands.spawn(sound_effect(sound));
        controller.sound_frame = Some(current_frame);
    }
}

/// Choose a random sound customized via parameters for current frame.
///
/// Returns [`Some`] if current frame is a fall sound frame.
/// Returns [`None`] if current frame is not a fall sound frame or on missing data.
fn choose_sound(
    rng: &mut WyRand,
    current_frame: &usize,
    sound_frames: &Option<Vec<usize>>,
    sounds: &Option<Vec<Handle<AudioSource>>>,
) -> Option<Handle<AudioSource>> {
    // Return `None` if frame data is missing or does not contain current frame
    let Some(sound_frames) = sound_frames else {
        warn_once!("{}", WARN_INCOMPLETE_ANIMATION_DATA);
        return None;
    };
    if !sound_frames.contains(current_frame) {
        return None;
    }
    // Return `None` if asset data is missing
    let Some(sounds) = sounds else {
        warn_once!("{}", WARN_INCOMPLETE_ASSET_DATA);
        return None;
    };

    sounds.choose(rng).cloned()
}

/// Tick [`AnimationTimer`]
fn tick_animation_timer(mut query: Query<&mut AnimationTimer>, time: Res<Time>) {
    for mut timer in &mut query {
        timer.0.tick(time.delta());
    }
}
