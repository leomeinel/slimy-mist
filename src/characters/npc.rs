/*
 * File: npc.rs
 * Author: Leopold Johannes Meinel (leo@meinel.dev)
 * -----
 * Copyright (c) 2025 Leopold Johannes Meinel & contributors
 * SPDX ID: Apache-2.0
 * URL: https://www.apache.org/licenses/LICENSE-2.0
 * -----
 * Heavily inspired by:
 * - https://github.com/TheBevyFlock/bevy_new_2d
 * - https://github.com/NiklasEi/bevy_common_assets/tree/main
 * - https://github.com/merwaaan/bevy_spritesheet_animation
 */

//! Npc-specific behavior.

use bevy::prelude::*;
use bevy_asset_loader::prelude::*;
use bevy_rapier2d::prelude::*;

use crate::{
    AppSystems, PausableSystems,
    characters::{
        Character, CharacterAssets, JumpTimer, Movement,
        animations::{self, Animations},
        character_collider, setup_shadow,
    },
    impl_character_assets,
    levels::{DEFAULT_Z, DynamicZ},
    screens::Screen,
};

pub(super) fn plugin(app: &mut App) {
    // Insert Animation resource
    app.insert_resource(Animations::<Slime>::default());

    // Setup slime shadow
    app.add_systems(OnEnter(Screen::Gameplay), setup_shadow::<Slime>);

    // Animation setup
    app.add_systems(
        OnEnter(Screen::Gameplay),
        animations::setup_animations::<Slime, SlimeAssets>,
    );

    // Animation updates
    app.add_systems(
        Update,
        (
            animations::update_animations::<Slime>.after(animations::tick_animation_timer),
            animations::update_animation_sounds::<Slime, SlimeAssets>
                .run_if(in_state(Screen::Gameplay)),
        )
            .chain()
            .in_set(AppSystems::Update)
            .in_set(PausableSystems),
    );
}

/// Assets that are serialized from a ron file
#[derive(AssetCollection, Resource, Default, Reflect)]
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
#[derive(Component, Default, Reflect)]
pub(crate) struct Npc;

/// Slime marker
#[derive(Component, Default, Reflect)]
pub(crate) struct Slime;
impl Character for Slime {
    fn container_bundle(
        &self,
        data: &(Option<String>, Option<f32>, Option<f32>),
        pos: Vec3,
    ) -> impl Bundle {
        (
            Name::new("Slime"),
            Npc,
            Self,
            Transform::from_translation(pos),
            character_collider::<Self>(data),
            Visibility::Inherited,
            DynamicZ(DEFAULT_Z),
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
}
