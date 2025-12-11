/*
 * File: npc.rs
 * Author: Leopold Johannes Meinel (leo@meinel.dev)
 * -----
 * Copyright (c) 2025 Leopold Johannes Meinel & contributors
 * SPDX ID: Apache-2.0
 * URL: https://www.apache.org/licenses/LICENSE-2.0
 */

//! Npc-specific behavior.

use bevy::prelude::*;
use bevy_asset_loader::prelude::*;
use bevy_common_assets::ron::RonAssetPlugin;
use bevy_rapier2d::prelude::*;
use bevy_spritesheet_animation::prelude::*;

use crate::{
    animations::{AnimationController, AnimationTimer, Animations},
    characters::{CharacterAssets, CollisionData, CollisionHandle, JumpTimer, collider},
    impl_character_assets,
};

pub(super) fn plugin(app: &mut App) {
    // Initialize asset state
    app.init_state::<NpcAssetState>();

    // Add plugin to load ron file
    app.add_plugins((RonAssetPlugin::<CollisionData<Slime>>::new(&[
        "animation.ron",
    ]),));

    // Setup slime
    app.add_systems(Startup, setup_slime);

    // Add loading states via bevy_asset_loader
    app.add_loading_state(
        LoadingState::new(NpcAssetState::AssetLoading)
            .continue_to_state(NpcAssetState::Next)
            .with_dynamic_assets_file::<StandardDynamicAssetCollection>(
                "data/characters/npc/slime.assets.ron",
            )
            .load_collection::<SlimeAssets>(),
    );
}

/// Asset state that tracks asset loading
#[derive(Clone, Eq, PartialEq, Debug, Hash, Default, States)]
pub(crate) enum NpcAssetState {
    #[default]
    AssetLoading,
    Next,
}

/// Assets that are serialized from a ron file
#[derive(AssetCollection, Resource)]
pub(crate) struct SlimeAssets {
    #[asset(key = "slime.step_sounds", collection(typed))]
    pub(crate) step_sounds: Vec<Handle<AudioSource>>,

    #[asset(key = "slime.image")]
    pub(crate) image: Handle<Image>,
}
impl_character_assets!(SlimeAssets);

/// Npc marker
#[derive(Component)]
pub(crate) struct Npc;

/// Slime marker
#[derive(Component, Default, Reflect)]
pub(crate) struct Slime;

/// Deserialize ron file for [`CollisionData`]
fn setup_slime(mut commands: Commands, assets: Res<AssetServer>) {
    let animation_handle =
        CollisionHandle::<Slime>(assets.load("data/characters/npc/slime.collision.ron"));
    commands.insert_resource(animation_handle);
}

/// The slime enemy.
pub(crate) fn slime(
    animations: &Res<Animations<Slime>>,
    collision_data: &Res<Assets<CollisionData<Slime>>>,
    collision_handle: &Res<CollisionHandle<Slime>>,
    animation_delay: f32,
) -> impl Bundle {
    (
        Name::new("Slime"),
        Npc,
        Slime,
        animations.sprite.clone(),
        SpritesheetAnimation::new(animations.idle.clone()),
        RigidBody::KinematicVelocityBased,
        GravityScale(0.),
        collider::<Slime>(collision_data, collision_handle),
        KinematicCharacterController::default(),
        LockedAxes::ROTATION_LOCKED,
        JumpTimer::default(),
        AnimationTimer(Timer::from_seconds(animation_delay, TimerMode::Once)),
        AnimationController::default(),
    )
}
