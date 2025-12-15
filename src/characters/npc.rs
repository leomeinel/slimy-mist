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
    characters::{CharacterAssets, CollisionData, CollisionHandle, JumpTimer, Movement},
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
    #[asset(key = "slime.walk_sounds", collection(typed), optional)]
    pub(crate) walk_sounds: Option<Vec<Handle<AudioSource>>>,

    #[asset(key = "slime.jump_sounds", collection(typed), optional)]
    pub(crate) jump_sounds: Option<Vec<Handle<AudioSource>>>,

    #[asset(key = "slime.fall_sounds", collection(typed), optional)]
    pub(crate) fall_sounds: Option<Vec<Handle<AudioSource>>>,

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
    let handle = CollisionHandle::<Slime>(assets.load("data/characters/npc/slime.collision.ron"));
    commands.insert_resource(handle);
}

/// Slime enemy parent
pub(crate) fn slime() -> impl Bundle {
    (
        Name::new("Slime"),
        Npc,
        Slime,
        RigidBody::KinematicPositionBased,
        GravityScale(0.),
        KinematicCharacterController {
            filter_flags: QueryFilterFlags::EXCLUDE_KINEMATIC,
            ..default()
        },
        LockedAxes::ROTATION_LOCKED,
        Movement::default(),
        JumpTimer::default(),
    )
}

/// Slime enemy child with visual components
pub(crate) fn slime_visual(
    animations: &Res<Animations<Slime>>,
    animation_delay: f32,
) -> impl Bundle {
    (
        animations.sprite.clone(),
        SpritesheetAnimation::new(animations.idle.clone()),
        AnimationController::default(),
        AnimationTimer(Timer::from_seconds(animation_delay, TimerMode::Once)),
    )
}
