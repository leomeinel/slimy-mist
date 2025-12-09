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
use bevy_rapier2d::prelude::*;
use bevy_spritesheet_animation::prelude::*;

use crate::{
    characters::{CharacterAssets, animations::Animations},
    impl_character_assets,
};

pub(super) fn plugin(app: &mut App) {
    // Initialize asset state
    app.init_state::<NpcAssetState>();

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

/// The slime enemy.
pub(crate) fn slime(animations: &Res<Animations<Slime>>) -> impl Bundle {
    (
        Name::new("Slime"),
        Npc,
        Slime,
        animations.sprite.clone(),
        SpritesheetAnimation::new(animations.idle.clone()),
        RigidBody::Dynamic,
        GravityScale(0.),
        Collider::ball(8.),
        KinematicCharacterController::default(),
    )
}
