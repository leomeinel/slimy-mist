/*
 * File: characters.rs
 * Author: Leopold Johannes Meinel (leo@meinel.dev)
 * -----
 * Copyright (c) 2026 Leopold Johannes Meinel & contributors
 * SPDX ID: Apache-2.0
 * URL: https://www.apache.org/licenses/LICENSE-2.0
 */

//! Characters

pub(crate) mod animations;
pub(crate) mod combat;
pub(crate) mod nav;
pub(crate) mod npc;
pub(crate) mod player;

use std::marker::PhantomData;

use bevy::{platform::collections::HashMap, prelude::*, reflect::Reflectable};
use bevy_asset_loader::asset_collection::AssetCollection;
use bevy_rapier2d::prelude::*;
use bevy_spritesheet_animation::prelude::SpritesheetAnimation;

use crate::{
    AppSystems,
    characters::animations::{AnimationController, AnimationTimer, Animations},
    logging::warn::WARN_INCOMPLETE_COLLISION_DATA_FALLBACK,
    screens::Screen,
    utils::math::NearEq,
};

pub(super) fn plugin(app: &mut App) {
    // Add child plugins
    app.add_plugins((
        animations::plugin,
        combat::plugin,
        npc::plugin,
        nav::plugin,
        player::plugin,
    ));

    // Tick timers
    app.add_systems(Update, tick_jump_timer.in_set(AppSystems::TickTimers));

    // Update movement facing
    app.add_systems(
        PostUpdate,
        update_movement_facing.run_if(in_state(Screen::Gameplay)),
    );
}

/// Jumping duration in seconds
pub(crate) const JUMP_DURATION_SECS: f32 = 1.;

/// Applies to anything that stores character assets
pub(crate) trait CharacterAssets
where
    Self: AssetCollection + Resource + Default + Reflectable,
{
    fn get_walk_sounds(&self) -> &Option<Vec<Handle<AudioSource>>>;
    fn get_jump_sounds(&self) -> &Option<Vec<Handle<AudioSource>>>;
    fn get_fall_sounds(&self) -> &Option<Vec<Handle<AudioSource>>>;
    fn get_image(&self) -> &Handle<Image>;
}
#[macro_export]
macro_rules! impl_character_assets {
    ($type: ty) => {
        impl CharacterAssets for $type {
            fn get_walk_sounds(&self) -> &Option<Vec<Handle<AudioSource>>> {
                &self.walk_sounds
            }
            fn get_jump_sounds(&self) -> &Option<Vec<Handle<AudioSource>>> {
                &self.jump_sounds
            }
            fn get_fall_sounds(&self) -> &Option<Vec<Handle<AudioSource>>> {
                &self.fall_sounds
            }
            fn get_image(&self) -> &Handle<Image> {
                &self.image
            }
        }
    };
}

/// Applies to any character [`Component`]
pub(crate) trait Character
where
    Self: Component + Default + Reflectable,
{
    fn container_bundle(
        &self,
        collision_set: &(Option<String>, Option<f32>, Option<f32>),
        pos: Vec2,
    ) -> impl Bundle;

    fn visual_bundle(
        &self,
        animations: &Res<Animations<Self>>,
        animation_delay: f32,
    ) -> impl Bundle {
        (
            animations.sprite.clone(),
            SpritesheetAnimation::new(animations.idle.clone()),
            AnimationController::default(),
            AnimationTimer(Timer::from_seconds(animation_delay, TimerMode::Once)),
        )
    }

    fn spawn(
        commands: &mut Commands,
        visual_map: &mut ResMut<VisualMap>,
        collision_set: &(Option<String>, Option<f32>, Option<f32>),
        pos: Vec2,
        animations: &Res<Animations<Self>>,
        animation_delay: f32,
    ) -> Entity {
        let character = Self::default();
        let container = commands
            .spawn(character.container_bundle(collision_set, pos))
            .id();

        let visual = commands
            .spawn(character.visual_bundle(animations, animation_delay))
            .id();
        commands.entity(container).add_child(visual);
        visual_map.0.insert(container, visual);

        container
    }
}

/// Collision data deserialized from a ron file as a generic
///
/// ## Traits
///
/// - `T` must implement [`Character`].
#[derive(serde::Deserialize, Asset, TypePath, Default)]
pub(crate) struct CollisionData<T>
where
    T: Character,
{
    #[serde(default)]
    pub(crate) shape: Option<String>,
    #[serde(default)]
    pub(crate) width: Option<f32>,
    #[serde(default)]
    pub(crate) height: Option<f32>,
    #[serde(skip)]
    _phantom: PhantomData<T>,
}

/// Handle for [`CollisionData`] as a generic
///
/// ## Traits
///
/// - `T` must implement [`Character`].
#[derive(Resource)]
pub(crate) struct CollisionHandle<T>(pub(crate) Handle<CollisionData<T>>)
where
    T: Character;

/// Cache for [`CollisionData`]
///
/// This is to allow easier access.
///
/// ## Traits
///
/// - `T` must implement [`Character`].
#[derive(Resource, Default)]
pub(crate) struct CollisionDataCache<T>
where
    T: Character,
{
    pub(crate) shape: Option<String>,
    pub(crate) width: Option<f32>,
    pub(crate) height: Option<f32>,
    pub(crate) _phantom: PhantomData<T>,
}

/// Current data about movement
#[derive(Component)]
pub(crate) struct Movement {
    pub(crate) direction: Vec2,
    old_direction: Vec2,
    pub(crate) facing: Vec2,
    jump_height: f32,
}
impl Default for Movement {
    fn default() -> Self {
        Self {
            direction: Vec2::default(),
            old_direction: Vec2::default(),
            facing: Vec2::X,
            jump_height: f32::default(),
        }
    }
}

/// Current data about movement
#[derive(Component, Copy, Clone)]
pub(crate) struct MovementSpeed(f32);
impl Default for MovementSpeed {
    fn default() -> Self {
        Self(80.)
    }
}

/// Timer that tracks jumping
#[derive(Component, Debug, Clone, PartialEq, Reflect)]
#[reflect(Component)]
struct JumpTimer(Timer);
impl Default for JumpTimer {
    fn default() -> Self {
        Self(Timer::from_seconds(
            JUMP_DURATION_SECS / 2.,
            TimerMode::Once,
        ))
    }
}

/// Health that determines if a [`Character`] is living.
#[derive(Component, Default)]
pub(crate) struct Health(f32);

/// Map of characters to their visual representations
#[derive(Resource, Default)]
pub(crate) struct VisualMap(pub(crate) HashMap<Entity, Entity>);

/// Radius of the fallback ball collider
const FALLBACK_BALL_COLLIDER_RADIUS: f32 = 8.;

/// [`Collider`] for different shapes
pub(crate) fn character_collider(
    collision_set: &(Option<String>, Option<f32>, Option<f32>),
) -> Collider {
    let (Some(shape), Some(width), Some(height)) = collision_set else {
        // Return default collider if data is not complete
        warn_once!("{}", WARN_INCOMPLETE_COLLISION_DATA_FALLBACK);
        return Collider::ball(FALLBACK_BALL_COLLIDER_RADIUS);
    };

    // Set correct collider for each shape
    // NOTE: For capsules, we just assume that the values are correct, meaning that for x: `width < height` and for y: `width > height`
    match shape.as_str() {
        "ball" => Collider::ball(width / 2.),
        "capsule_x" => Collider::capsule_x((height - width) / 2., height / 2.),
        "capsule_y" => Collider::capsule_y((width - height) / 2., width / 2.),
        _ => Collider::cuboid(width / 2., height / 2.),
    }
}

/// Update [`Movement::facing`] from [`Movement::direction`].
///
/// This will only set a new [`Movement::facing`] if [`Movement::direction`] is not near zero.
/// This also updates [`Movement::old_direction`] from [`Movement::direction`].
fn update_movement_facing(query: Query<&mut Movement>) {
    for mut movement in query {
        let epsilon = 0.1;
        // NOTE: This only checks for desired movement, not actual movement. This is to ensure that
        //       even if a character can't move, it can still change its' facing direction.
        if !movement.direction.is_near(movement.old_direction, epsilon)
            && !movement.direction.is_near_zero(0.1)
        {
            movement.facing = movement.direction.normalize_or_zero();
        }
        movement.old_direction = movement.direction;
    }
}

/// Tick jump timer
fn tick_jump_timer(mut query: Query<&mut JumpTimer>, time: Res<Time>) {
    for mut timer in &mut query {
        timer.0.tick(time.delta());
    }
}
