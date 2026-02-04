/*
 * File: player.rs
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

//! Player-specific behavior.

use bevy::{platform::collections::HashSet, prelude::*};
use bevy_asset_loader::prelude::*;
use bevy_rapier2d::prelude::*;
use bevy_spritesheet_animation::prelude::*;

use crate::{
    AppSystems,
    animations::{AnimationCache, AnimationState, AnimationTimer, Animations},
    camera::{FOREGROUND_Z, ysort::YSort},
    characters::{
        Character, CharacterAssets, JumpTimer, Movement,
        attack::{AttackStats, punch},
        character_collider,
        health::Health,
        nav::NavTarget,
    },
    impl_character_assets,
    input::player_input,
    logging::error::ERR_INVALID_CHILDREN,
    visual::Visible,
};

pub(super) fn plugin(app: &mut App) {
    // Insert resources
    app.init_resource::<Animations<Player>>();

    // Jump or stop jump depending on timer
    app.add_systems(
        Update,
        (apply_jump.before(PhysicsSet::SyncBackend), limit_jump)
            .chain()
            .in_set(AppSystems::Update),
    );
}

/// Assets that are serialized from a ron file
#[derive(AssetCollection, Resource, Reflect, Default)]
pub(crate) struct PlayerAssets {
    #[asset(key = "male.walk_sounds", collection(typed), optional)]
    pub(crate) walk_sounds: Option<Vec<Handle<AudioSource>>>,

    #[asset(key = "male.jump_sounds", collection(typed), optional)]
    pub(crate) jump_sounds: Option<Vec<Handle<AudioSource>>>,

    #[asset(key = "male.fall_sounds", collection(typed), optional)]
    pub(crate) fall_sounds: Option<Vec<Handle<AudioSource>>>,
}
impl_character_assets!(PlayerAssets);

/// Player marker
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Default, Reflect)]
#[reflect(Component)]
pub(crate) struct Player;
impl Character for Player {
    fn container_bundle(
        &self,
        animation_delay: f32,
        collision_set: &(Option<String>, Option<f32>, Option<f32>),
        pos: Vec2,
    ) -> impl Bundle {
        (
            // Identity
            (Name::new("Player"), Self),
            // Positioning/Visibility
            (
                Transform::from_translation(pos.extend(FOREGROUND_Z)),
                YSort(FOREGROUND_Z),
                Visibility::Inherited,
            ),
            // Physics
            (
                character_collider(collision_set),
                RigidBody::KinematicVelocityBased,
                GravityScale(0.),
            ),
            // Movement
            (
                player_input(),
                KinematicCharacterController {
                    filter_flags: QueryFilterFlags::EXCLUDE_KINEMATIC,
                    ..default()
                },
                LockedAxes::ROTATION_LOCKED,
                Movement::default(),
                NavTarget(128),
            ),
            // Attack
            (
                Health(10.),
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
impl Visible for Player {}

/// Jump height
const JUMP_HEIGHT: f32 = 12.;

/// Apply jump
fn apply_jump(
    player: Single<(&AnimationCache, &mut Movement, &JumpTimer, &Children), With<Player>>,
    mut transform_query: Query<&mut Transform, With<SpritesheetAnimation>>,
) {
    let (cache, mut movement, timer, children) = player.into_inner();

    // Return if we are not jumping or falling
    let state = cache.state;
    if !matches!(state, AnimationState::Jump | AnimationState::Fall) {
        return;
    }

    // Apply visual jump or fall
    let factor = if state == AnimationState::Jump {
        1.0f32
    } else {
        -1.0f32
    };
    let eased_time = EasingCurve::new(0., 1., EaseFunction::QuadraticOut);
    let eased_time = eased_time.sample_clamped(timer.0.fraction());
    let target = JUMP_HEIGHT * factor * eased_time;

    let child = children
        .iter()
        .find(|e| transform_query.contains(*e))
        .expect(ERR_INVALID_CHILDREN);
    let mut transform = transform_query.get_mut(child).expect(ERR_INVALID_CHILDREN);
    transform.translation.y += target - movement.jump_height;
    movement.jump_height = target;
}

/// Limit jump by setting fall after specific time and then switching to walk
fn limit_jump(
    player: Single<(Entity, &mut AnimationCache, &mut Movement, &JumpTimer), With<Player>>,
    mut commands: Commands,
) {
    let (entity, mut cache, mut movement, timer) = player.into_inner();

    // Return if timer has not finished
    if !timer.0.just_finished() {
        return;
    }

    // Reset jump height
    movement.jump_height = 0.;

    // Set animation states
    match cache.state {
        AnimationState::Jump => {
            commands.entity(entity).insert(JumpTimer::default());
            cache.state = AnimationState::Fall;
        }
        AnimationState::Fall => cache.state = AnimationState::Idle,
        _ => (),
    }
}
