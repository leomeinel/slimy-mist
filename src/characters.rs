/*
 * File: characters.rs
 * Author: Leopold Johannes Meinel (leo@meinel.dev)
 * -----
 * Copyright (c) 2026 Leopold Johannes Meinel & contributors
 * SPDX ID: Apache-2.0
 * URL: https://www.apache.org/licenses/LICENSE-2.0
 */

//! Characters

pub(crate) mod attack;
pub(crate) mod health;
pub(crate) mod nav;
pub(crate) mod npc;
pub(crate) mod player;

use std::marker::PhantomData;

use bevy::{prelude::*, reflect::Reflectable};
use bevy_asset_loader::asset_collection::AssetCollection;
use bevy_light_2d::prelude::*;
use bevy_rapier2d::prelude::*;
use bevy_spritesheet_animation::prelude::SpritesheetAnimation;

use crate::{
    AppSystems, animations::Animations, camera::BACKGROUND_Z_DELTA,
    logging::warn::WARN_INCOMPLETE_COLLISION_DATA, screens::Screen,
};

pub(super) fn plugin(app: &mut App) {
    // Add child plugins
    app.add_plugins((
        attack::plugin,
        health::plugin,
        nav::plugin,
        npc::plugin,
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
    fn walk_sounds(&self) -> &Option<Vec<Handle<AudioSource>>>;
    fn jump_sounds(&self) -> &Option<Vec<Handle<AudioSource>>>;
    fn fall_sounds(&self) -> &Option<Vec<Handle<AudioSource>>>;
}
#[macro_export]
macro_rules! impl_character_assets {
    ($type: ty) => {
        impl CharacterAssets for $type {
            fn walk_sounds(&self) -> &Option<Vec<Handle<AudioSource>>> {
                &self.walk_sounds
            }
            fn jump_sounds(&self) -> &Option<Vec<Handle<AudioSource>>> {
                &self.jump_sounds
            }
            fn fall_sounds(&self) -> &Option<Vec<Handle<AudioSource>>> {
                &self.fall_sounds
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
        animation_delay: f32,
        collision_set: &(Option<String>, Option<f32>, Option<f32>),
        pos: Vec2,
    ) -> impl Bundle;

    fn animation_bundle(&self, animations: &Res<Animations<Self>>) -> impl Bundle {
        (
            animations.sprite.clone(),
            SpritesheetAnimation::new(animations.idle.clone()),
        )
    }

    fn shadow_bundle(
        &self,
        collision_set: &(Option<String>, Option<f32>, Option<f32>),
    ) -> impl Bundle {
        let (_, Some(width), Some(height)) = collision_set else {
            // Return default collider if data is not complete
            warn_once!("{}", WARN_INCOMPLETE_COLLISION_DATA);
            return (LightOccluder2d::default(), Transform::default());
        };

        (
            LightOccluder2d {
                // FIXME: Use ellipse.
                //        Wait for: https://github.com/jgayfer/bevy_light_2d/issues/41
                shape: LightOccluder2dShape::Rectangle {
                    // NOTE: We are dividing height by 4 because of 2:1 pixel ratio
                    half_size: Vec2::new(width / 2., height / 4.),
                },
            },
            Transform::from_xyz(0., -height / 2., BACKGROUND_Z_DELTA),
        )
    }

    fn spawn(
        &self,
        commands: &mut Commands,
        collision_set: &(Option<String>, Option<f32>, Option<f32>),
        pos: Vec2,
        animations: &Res<Animations<Self>>,
        animation_delay: f32,
    ) -> Entity {
        let container = commands
            .spawn(self.container_bundle(animation_delay, collision_set, pos))
            .id();
        let animation = commands.spawn(self.animation_bundle(animations)).id();
        let shadow = commands.spawn(self.shadow_bundle(collision_set)).id();
        commands
            .entity(container)
            .add_children(&[animation, shadow]);

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

/// Timer that tracks jumping
#[derive(Component, Debug, Clone, PartialEq, Reflect)]
#[reflect(Component)]
pub(crate) struct JumpTimer(Timer);
impl Default for JumpTimer {
    fn default() -> Self {
        Self(Timer::from_seconds(
            JUMP_DURATION_SECS / 2.,
            TimerMode::Once,
        ))
    }
}

/// [`Collider`] for different shapes
pub(crate) fn character_collider(
    collision_set: &(Option<String>, Option<f32>, Option<f32>),
) -> Collider {
    let (Some(shape), Some(width), Some(height)) = collision_set else {
        warn_once!("{}", WARN_INCOMPLETE_COLLISION_DATA);
        return Collider::ball(0.);
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
        // NOTE: This only checks for desired movement, not actual movement. This is to ensure that
        //       even if a character can't move, it can still change its' facing direction.
        if movement.direction != Vec2::ZERO && movement.direction != movement.old_direction {
            movement.facing = movement.direction.normalize_or_zero();
        }
        movement.old_direction = movement.direction;
    }
}

/// Tick [`JumpTimer`]
fn tick_jump_timer(mut query: Query<&mut JumpTimer>, time: Res<Time>) {
    for mut timer in &mut query {
        timer.0.tick(time.delta());
    }
}
