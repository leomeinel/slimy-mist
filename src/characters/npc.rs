/*
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
    animations::prelude::*, characters::prelude::*, procgen::prelude::*, render::prelude::*,
};

/// Assets that are serialized from a ron file
#[derive(AssetCollection, Resource, Default, Reflect)]
pub(crate) struct SlimeAssets {
    #[asset(key = "slime.idle_sounds", collection(typed), optional)]
    pub(crate) idle_sounds: Option<Vec<Handle<AudioSource>>>,

    #[asset(key = "slime.walk_sounds", collection(typed), optional)]
    pub(crate) walk_sounds: Option<Vec<Handle<AudioSource>>>,

    #[asset(key = "slime.jump_sounds", collection(typed), optional)]
    pub(crate) jump_sounds: Option<Vec<Handle<AudioSource>>>,
}
impl_character_assets!(SlimeAssets);

/// Npc marker.
#[derive(Component, Default, Reflect)]
pub(crate) struct Npc;

/// Walk speed of a [`Slime`].
const SLIME_WALK_SPEED: f32 = 60.;

/// Slime marker
#[derive(Component, Default, Reflect)]
pub(crate) struct Slime;
impl Character for Slime {
    fn container_bundle(pos: Vec2, animation_delay: f32, y_offset: f32) -> impl Bundle {
        (
            // Identity
            (Name::new("Slime"), Npc),
            // Positioning/Visibility
            (
                Transform::from_translation(pos.extend(FOREGROUND_Z)),
                YSort(FOREGROUND_Z),
                YSortYOffset(y_offset),
                Visibility::Inherited,
            ),
            // Physics
            (RigidBody::KinematicPositionBased, GravityScale(0.)),
            // Movement
            (
                KinematicCharacterController::default(),
                LockedAxes::ROTATION_LOCKED,
                FacingDirection::default(),
                WalkSpeed(SLIME_WALK_SPEED),
            ),
            // Navigation
            Navigator,
            // Attack
            (
                Health::new(5.),
                AimDirection::default(),
                AttackStats {
                    _attacks: vec![punch()],
                    damage_factor: 1.,
                    melee: Some(punch()),
                    _ranged: None,
                },
            ),
            // Animations
            (
                AnimationAudioIndex::default(),
                AnimationState::default(),
                LastAnimationAction::default(),
                AnimationYOffset::default(),
                AnimationTimer(Timer::from_seconds(animation_delay, TimerMode::Once)),
            ),
        )
    }
}
impl ProcGenerated for Slime {}
impl Visible for Slime {}
