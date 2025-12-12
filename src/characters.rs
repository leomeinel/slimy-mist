/*
 * File: characters.rs
 * Author: Leopold Johannes Meinel (leo@meinel.dev)
 * -----
 * Copyright (c) 2025 Leopold Johannes Meinel & contributors
 * SPDX ID: Apache-2.0
 * URL: https://www.apache.org/licenses/LICENSE-2.0
 */

//! Characters

pub(crate) mod npc;
pub(crate) mod player;

use std::marker::PhantomData;

use bevy::{platform::collections::HashMap, prelude::*, reflect::Reflectable};
use bevy_rapier2d::prelude::Collider;

use crate::AppSystems;

pub(super) fn plugin(app: &mut App) {
    // Insert `VisualMap`
    app.insert_resource(VisualMap::default());

    // Add child plugins
    app.add_plugins((npc::plugin, player::plugin));

    // Tick jump timer
    app.add_systems(Update, tick_jump_timer.in_set(AppSystems::TickTimers));
}

/// Applies to anything that stores character assets
pub(crate) trait CharacterAssets {
    fn get_step_sounds(&self) -> &Vec<Handle<AudioSource>>;
    fn get_image(&self) -> &Handle<Image>;
}
#[macro_export]
macro_rules! impl_character_assets {
    ($type: ty) => {
        impl CharacterAssets for $type {
            fn get_step_sounds(&self) -> &Vec<Handle<AudioSource>> {
                &self.step_sounds
            }
            fn get_image(&self) -> &Handle<Image> {
                &self.image
            }
        }
    };
}

/// Animation data deserialized from a ron file as a generic
#[derive(serde::Deserialize, Asset, TypePath)]
pub(crate) struct CollisionData<T>
where
    T: Reflectable,
{
    shape: String,
    width: f32,
    height: f32,
    #[serde(skip)]
    _phantom: PhantomData<T>,
}

/// Handle for [`CollisionData`] as a generic
#[derive(Resource)]
pub(crate) struct CollisionHandle<T>(Handle<CollisionData<T>>)
where
    T: Reflectable;

/// Current data about movement
#[derive(Component, Default)]
pub(crate) struct Movement {
    pub(crate) target: Vec2,
    jump_height: f32,
}

/// Jumping duration in seconds
const JUMP_DURATION_SECS: f32 = 1.;

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

/// Collider for different shapes
pub(crate) fn collider<T>(
    data: &Res<Assets<CollisionData<T>>>,
    handle: &Res<CollisionHandle<T>>,
) -> Collider
where
    T: Component + Default + Reflectable,
{
    // Get data from `CollisionData` with `CollisionHandle`
    let data = data.get(handle.0.id()).unwrap();

    let (width, height) = (data.width, data.height);
    // Set correct collider for each shape
    // NOTE: For capsules, we just assume that the values are correct, meaning that for x: `width < height` and for y: `width > height`
    match data.shape.as_str() {
        "ball" => Collider::ball(width / 2.),
        "capsule_x" => Collider::capsule_x((height - width) / 2., height / 2.),
        "capsule_y" => Collider::capsule_y((width - height) / 2., width / 2.),
        _ => Collider::cuboid(width / 2., height / 2.),
    }
}

/// Tick jump timer
fn tick_jump_timer(mut query: Query<&mut JumpTimer>, time: Res<Time>) {
    for mut timer in &mut query {
        timer.0.tick(time.delta());
    }
}
