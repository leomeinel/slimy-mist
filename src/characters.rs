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
pub(crate) mod nav;
pub(crate) mod npc;
pub(crate) mod player;

use std::marker::PhantomData;

use bevy::{platform::collections::HashMap, prelude::*, reflect::Reflectable};
use bevy_asset_loader::asset_collection::AssetCollection;
use bevy_rapier2d::prelude::*;
use bevy_spritesheet_animation::prelude::SpritesheetAnimation;
use vleue_navigator::prelude::*;

use crate::{
    AppSystems,
    characters::animations::{AnimationController, AnimationTimer, Animations},
    logging::warn::WARN_INCOMPLETE_COLLISION_DATA_FALLBACK,
    screens::{ResInsertGameplay, Screen},
};

pub(super) fn plugin(app: &mut App) {
    // Insert/Remove resources
    app.add_systems(
        OnEnter(Screen::Gameplay),
        insert_resources.in_set(ResInsertGameplay),
    );
    app.add_systems(OnExit(Screen::Gameplay), remove_resources);

    // Add child plugins
    app.add_plugins((animations::plugin, npc::plugin, nav::plugin, player::plugin));

    // Tick timers
    app.add_systems(Update, tick_jump_timer.in_set(AppSystems::TickTimers));
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

/// Animation data deserialized from a ron file as a generic
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

/// Current data about movement
#[derive(Component, Default)]
pub(crate) struct Movement {
    pub(crate) direction: Vec2,
    jump_height: f32,
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

/// [`PrimitiveObstacle`] for different shapes
pub(crate) fn character_obstacle(
    collision_set: &(Option<String>, Option<f32>, Option<f32>),
) -> PrimitiveObstacle {
    let (Some(shape), Some(width), Some(height)) = collision_set else {
        // Return default collider if data is not complete
        warn_once!("{}", WARN_INCOMPLETE_COLLISION_DATA_FALLBACK);
        return PrimitiveObstacle::Circle(Circle::new(FALLBACK_BALL_COLLIDER_RADIUS));
    };

    // Set correct collider for each shape
    // NOTE: For capsules, we just assume that the values are correct, meaning that for x: `width < height` and for y: `width > height`
    match shape.as_str() {
        "ball" => PrimitiveObstacle::Circle(Circle::new(width / 2.)),
        "capsule_x" => PrimitiveObstacle::Rectangle(Rectangle::new(*width, *height)),
        "capsule_y" => PrimitiveObstacle::Rectangle(Rectangle::new(*width, *height)),
        _ => PrimitiveObstacle::Rectangle(Rectangle::new(*width, *height)),
    }
}

/// Tick jump timer
fn tick_jump_timer(mut query: Query<&mut JumpTimer>, time: Res<Time>) {
    for mut timer in &mut query {
        timer.0.tick(time.delta());
    }
}

/// Insert resources
fn insert_resources(mut commands: Commands) {
    commands.init_resource::<VisualMap>();
}

/// Remove resources
fn remove_resources(mut commands: Commands) {
    commands.remove_resource::<VisualMap>();
}
