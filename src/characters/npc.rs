/*
 * File: npc.rs
 * Author: Leopold Johannes Meinel (leo@meinel.dev)
 * -----
 * Copyright (c) 2026 Leopold Johannes Meinel & contributors
 * SPDX ID: Apache-2.0
 * URL: https://www.apache.org/licenses/LICENSE-2.0
 * -----
 * Heavily inspired by:
 * - https://github.com/TheBevyFlock/bevy_new_2d
 * - https://github.com/NiklasEi/bevy_common_assets/tree/main
 * - https://github.com/merwaaan/bevy_spritesheet_animation
 */

//! Npc-specific behavior.

use bevy::{platform::collections::HashSet, prelude::*};
use bevy_asset_loader::prelude::*;
use bevy_rapier2d::prelude::*;

use crate::{
    animations::{AnimationCache, AnimationTimer, Animations},
    camera::{FOREGROUND_Z, ysort::YSort},
    characters::{
        Character, CharacterAssets, Movement,
        attack::{AttackStats, punch},
        health::Health,
        nav::Navigator,
    },
    impl_character_assets,
    procgen::ProcGenerated,
    visual::Visible,
};

pub(super) fn plugin(app: &mut App) {
    // Insert resources
    app.init_resource::<Animations<Slime>>();
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
}
impl_character_assets!(SlimeAssets);

/// Npc marker
#[derive(Component, Default, Reflect)]
pub(crate) struct Npc;

/// Walk speed of a [`Slime`]
const SLIME_WALK_SPEED: f32 = 80.;

/// Slime marker
#[derive(Component, Default, Reflect)]
pub(crate) struct Slime;
impl Character for Slime {
    fn container_bundle(&self, animation_delay: f32, pos: Vec2) -> impl Bundle {
        (
            // Identity
            (Name::new("Slime"), Npc),
            // Positioning/Visibility
            (
                Transform::from_translation(pos.extend(FOREGROUND_Z)),
                YSort(FOREGROUND_Z),
                Visibility::Inherited,
            ),
            // Physics
            (RigidBody::KinematicPositionBased, GravityScale(0.)),
            // Movement
            (
                KinematicCharacterController::default(),
                LockedAxes::ROTATION_LOCKED,
                Movement::default(),
                Navigator(SLIME_WALK_SPEED),
            ),
            // Attack
            (
                Health(5.),
                AttackStats {
                    _attacks: HashSet::from([punch()]),
                    damage_factor: 1.,
                    melee: Some(punch()),
                    _ranged: None,
                },
            ),
            // Animations
            (
                AnimationCache::default(),
                AnimationTimer(Timer::from_seconds(animation_delay, TimerMode::Once)),
            ),
        )
    }
}
impl ProcGenerated for Slime {}
impl Visible for Slime {}
