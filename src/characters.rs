/*
 * File: characters.rs
 * Author: Leopold Johannes Meinel (leo@meinel.dev)
 * -----
 * Copyright (c) 2025 Leopold Johannes Meinel & contributors
 * SPDX ID: Apache-2.0
 * URL: https://www.apache.org/licenses/LICENSE-2.0
 */

//! Characters

pub(crate) mod animations;
pub(crate) mod npc;
pub(crate) mod player;

use std::marker::PhantomData;

use bevy::{
    color::palettes::tailwind, platform::collections::HashMap, prelude::*, reflect::Reflectable,
};
use bevy_asset_loader::asset_collection::AssetCollection;
use bevy_rapier2d::prelude::*;
use bevy_spritesheet_animation::prelude::SpritesheetAnimation;

use crate::{
    AppSystems,
    characters::animations::{AnimationController, AnimationTimer, Animations},
    levels::{DEFAULT_Z, SHADOW_Z, YSort},
    logging::{error::ERR_LOADING_COLLISION_DATA, warn::WARN_INCOMPLETE_COLLISION_DATA_FALLBACK},
};

pub(super) fn plugin(app: &mut App) {
    // Insert `VisualMap`
    app.insert_resource(VisualMap::default());

    // Add child plugins
    app.add_plugins((animations::plugin, npc::plugin, player::plugin));

    // Tick jump timer
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
        data: &(Option<String>, Option<f32>, Option<f32>),
        pos: Vec2,
    ) -> impl Bundle;

    fn visual_bundle(
        &self,
        animations: &Res<Animations<Self>>,
        animation_delay: f32,
    ) -> impl Bundle {
        (
            YSort(DEFAULT_Z),
            animations.sprite.clone(),
            SpritesheetAnimation::new(animations.idle.clone()),
            AnimationController::default(),
            AnimationTimer(Timer::from_seconds(animation_delay, TimerMode::Once)),
        )
    }

    fn shadow_bundle(&self, shadow: &Res<Shadow<Self>>, width: f32) -> impl Bundle {
        (
            YSort(SHADOW_Z),
            Transform::from_xyz(0., -width / 2., SHADOW_Z),
            Mesh2d(shadow.mesh.clone()),
            MeshMaterial2d(shadow.material.clone()),
        )
    }

    fn spawn(
        commands: &mut Commands,
        visual_map: &mut ResMut<VisualMap>,
        data: &(Option<String>, Option<f32>, Option<f32>),
        pos: Vec2,
        animations: &Res<Animations<Self>>,
        shadow: &Res<Shadow<Self>>,
        animation_delay: f32,
    ) -> Entity {
        let character = Self::default();
        let container = commands.spawn(character.container_bundle(data, pos)).id();

        let visual = commands
            .spawn(character.visual_bundle(animations, animation_delay))
            .id();
        commands.entity(container).add_child(visual);
        visual_map.0.insert(container, visual);

        let width = data.1.unwrap_or_else(|| {
            warn_once!("{}", WARN_INCOMPLETE_COLLISION_DATA_FALLBACK);
            24.
        });
        let shadow = commands.spawn(character.shadow_bundle(shadow, width)).id();
        commands.entity(container).add_child(shadow);

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
    pub(crate) target: Vec2,
    jump_height: f32,
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

/// Shadow data for characters
///
/// ## Traits
///
/// - `T` must implement [`Character`].
#[derive(Resource, Default, Debug)]
pub(crate) struct Shadow<T>
where
    T: Character,
{
    mesh: Handle<Mesh>,
    material: Handle<ColorMaterial>,
    _phantom: PhantomData<T>,
}

/// Color for cast shadows
const SHADOW_COLOR: Srgba = tailwind::GRAY_700;

/// Setup [`Shadow`]
///
/// ## Traits
///
/// - `T` must implement [`Character`].
pub(crate) fn setup_shadow<T>(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    data: Res<Assets<CollisionData<T>>>,
    handle: Res<CollisionHandle<T>>,
) where
    T: Character,
{
    // Get animation from `AnimationData` with `CollisionHandle`
    let data = data.get(handle.0.id()).expect(ERR_LOADING_COLLISION_DATA);
    let width = data.width.unwrap_or_else(|| {
        warn_once!("{}", WARN_INCOMPLETE_COLLISION_DATA_FALLBACK);
        24.
    });

    let resource = Shadow::<T> {
        mesh: meshes.add(Circle::new(-width / 4.)),
        material: materials.add(Color::from(SHADOW_COLOR.with_alpha(0.25))),
        ..default()
    };

    commands.insert_resource(resource);
}

/// Collider for different shapes
pub(crate) fn character_collider(data: &(Option<String>, Option<f32>, Option<f32>)) -> Collider {
    let (Some(shape), Some(width), Some(height)) = data else {
        // Return default collider if data is not complete
        warn_once!("{}", WARN_INCOMPLETE_COLLISION_DATA_FALLBACK);
        return Collider::ball(12.);
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

/// Tick jump timer
fn tick_jump_timer(mut query: Query<&mut JumpTimer>, time: Res<Time>) {
    for mut timer in &mut query {
        timer.0.tick(time.delta());
    }
}
