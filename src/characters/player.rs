/*
 * Heavily inspired by:
 * - https://github.com/TheBevyFlock/bevy_new_2d
 * - https://github.com/NiklasEi/bevy_common_assets/tree/main
 * - https://github.com/merwaaan/bevy_spritesheet_animation
 */

//! Player-specific behavior.

use bevy::prelude::*;
use bevy_asset_loader::prelude::*;
use bevy_enhanced_input::prelude::*;
use bevy_rapier2d::prelude::*;

use crate::{
    animations::prelude::*, characters::prelude::*, input::prelude::*, render::prelude::*,
};

/// Walk speed of [`Player`].
const PLAYER_WALK_SPEED: f32 = 60.;

/// Assets that are serialized from a ron file
#[derive(AssetCollection, Resource, Reflect, Default)]
pub(crate) struct PlayerAssets {
    #[asset(key = "male.idle_sounds", collection(typed), optional)]
    pub(crate) idle_sounds: Option<Vec<Handle<AudioSource>>>,

    #[asset(key = "male.walk_sounds", collection(typed), optional)]
    pub(crate) walk_sounds: Option<Vec<Handle<AudioSource>>>,

    #[asset(key = "male.jump_sounds", collection(typed), optional)]
    pub(crate) jump_sounds: Option<Vec<Handle<AudioSource>>>,
}
impl_character_assets!(PlayerAssets);

/// Player marker
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Default, Reflect)]
#[reflect(Component)]
pub(crate) struct Player;
impl Character for Player {
    fn container_bundle(pos: Vec2, animation_delay: f32, y_offset: f32) -> impl Bundle {
        (
            // Identity
            (Name::new("Player")),
            // Positioning/Visibility
            (
                Transform::from_translation(pos.extend(FOREGROUND_Z)),
                YSort(FOREGROUND_Z),
                YSortYOffset(y_offset),
                Visibility::Inherited,
            ),
            // Physics
            (RigidBody::KinematicVelocityBased, GravityScale(0.)),
            // Movement
            (
                player_input(),
                KinematicCharacterController {
                    filter_flags: QueryFilterFlags::EXCLUDE_KINEMATIC,
                    ..default()
                },
                LockedAxes::ROTATION_LOCKED,
                FacingDirection::default(),
                JumpHeight::default(),
                WalkSpeed(PLAYER_WALK_SPEED),
            ),
            // Navigation
            NavTarget(128),
            // Attack
            (
                Health::new(10.),
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
impl Visible for Player {}

/// On [`InitAttack`], set [`AimDirection`] and write [`Attack`].
pub(super) fn on_init_attack(
    mut reader: MessageReader<InitAttack>,
    mut writer: MessageWriter<Attack>,
    aim: Single<&mut Action<Aim>, Changed<Action<Aim>>>,
    mut aim_direction: Single<&mut AimDirection, With<Player>>,
) {
    for attack in reader.read() {
        ***aim_direction = ***aim;
        writer.write(Attack::from(attack));
    }
}
