//! Characters

mod attack;
mod health;
mod movement;
mod nav;
mod npc;
mod player;

#[allow(unused_imports)]
pub(crate) mod prelude {
    pub(crate) use super::attack::{
        AimDirection, Attack, AttackData, AttackStats, AttackTimer, DelayAttack, InitAttack, punch,
    };
    pub(crate) use super::health::{Damage, Health};
    pub(crate) use super::movement::{
        FacingDirection, JUMP_DURATION_SECS, JumpHeight, JumpTimer, WalkSpeed,
    };
    pub(crate) use super::nav::{NavTarget, Navigator, Path};
    pub(crate) use super::npc::{Npc, Slime, SlimeAssets};
    pub(crate) use super::player::{Player, PlayerAssets};
    pub(crate) use super::{Character, CharacterAssets, SpawnCharacter, impl_character_assets};
}

use std::marker::PhantomData;

use bevy::{prelude::*, reflect::Reflectable};
use bevy_asset_loader::asset_collection::AssetCollection;
use bevy_fast_light::prelude::*;
use bevy_prng::WyRand;
use bevy_rapier2d::prelude::*;
use bevy_spritesheet_animation::prelude::*;
use rand::RngExt as _;

use crate::{
    animations::prelude::*, characters::prelude::*, core::prelude::*, levels::prelude::*,
    physics::prelude::*, procgen::prelude::*, render::prelude::*, screens::prelude::*,
    ui::prelude::*, utils::prelude::*,
};

pub(super) struct CharactersPlugin;
impl Plugin for CharactersPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SpriteAnimations<Slime>>();
        app.init_resource::<SpriteAnimations<Player>>();

        app.add_message::<Attack>();
        app.add_message::<InitAttack>();

        app.add_systems(
            Update,
            (
                (
                    nav::find_path::<OverworldProcGen>,
                    nav::refresh_path::<OverworldProcGen>,
                )
                    .run_if(in_state(DespawnProcGen(false))),
                nav::apply_path.in_set(PausableSystems),
            )
                .run_if(in_state(ProcGenInit(true)).and(in_state(Screen::Gameplay)))
                .chain()
                .in_set(AppSystems::Update),
        );
        app.add_systems(
            Update,
            (
                player::on_init_attack,
                movement::update_facing_direction,
                attack::on_melee_attack::<Player>,
            )
                .run_if(in_state(Screen::Gameplay))
                .chain(),
        );
        app.add_systems(
            Update,
            (
                tick_component_timers::<attack::AttackTimer>,
                tick_component_timers::<movement::JumpTimer>,
            )
                .in_set(AppSystems::TickTimers),
        );
        app.add_systems(
            PostUpdate,
            (
                remove_oneshot_component_timers::<attack::AttackTimer>,
                remove_oneshot_component_timers::<movement::JumpTimer>,
            ),
        );

        app.add_observer(attack::on_delay_attack);
        app.add_observer(health::on_damage);
        app.add_observer(nav::on_stop_nav);
        app.add_observer(on_spawn_character::<Player, Overworld>);
        app.add_observer(on_spawn_character::<Slime, Overworld>);
    }
}

/// Applies to anything that stores character assets
pub(crate) trait CharacterAssets
where
    Self: AssetCollection + Resource + Default + Reflectable,
{
    fn sounds(&self, action: AnimationAction) -> &Option<Vec<Handle<AudioSource>>>;
}
macro_rules! impl_character_assets {
    ($type: ty) => {
        impl CharacterAssets for $type {
            fn sounds(&self, action: AnimationAction) -> &Option<Vec<Handle<AudioSource>>> {
                match action {
                    AnimationAction::Idle => &self.idle_sounds,
                    AnimationAction::Walk => &self.walk_sounds,
                    AnimationAction::Jump => &self.jump_sounds,
                }
            }
        }
    };
}
pub(crate) use impl_character_assets;

/// Applies to any character [`Component`]
pub(crate) trait Character
where
    Self: Component + Default + Reflectable,
{
    fn container_bundle(pos: Vec2, animation_delay: f32, y_offset: f32) -> impl Bundle;

    fn animation_bundle(animation: &SpriteAnimation) -> impl Bundle {
        (
            animation.sprite.clone(),
            SpritesheetAnimation::new(AnimationState::default().animation(animation)),
        )
    }

    fn collider(shape: String, width: f32, height: f32) -> Collider {
        match shape.as_str() {
            "ball" => Collider::ball(width / 2.),
            "capsule" => {
                if width > height {
                    Collider::capsule_x((height - width) / 2., height / 2.)
                } else {
                    Collider::capsule_y((width - height) / 2., width / 2.)
                }
            }
            _ => Collider::cuboid(width / 2., height / 2.),
        }
    }

    fn shadow_bundle<T>(shadow: &MeshLightShadow<T>) -> impl Bundle
    where
        T: Visible,
    {
        (
            MeshOccluder,
            Mesh2d(shadow.mesh.clone()),
            Transform::from_xyz(0., shadow.y_offset, -Y_SORT_OVERRIDE_Z_DELTA),
        )
    }
}

/// [`EntityEvent`] for spawning a [`Character`].
#[derive(EntityEvent)]
pub(crate) struct SpawnCharacter<T, A>
where
    T: Character,
    A: Level,
{
    pub(crate) entity: Entity,
    pub(crate) pos: Vec2,
    pub(crate) _phantom: PhantomData<(T, A)>,
}

/// Spawn a single [`Character`].
fn on_spawn_character<T, A>(
    event: On<SpawnCharacter<T, A>>,
    mut animation_rng: Single<&mut WyRand, With<AnimationRng>>,
    level: Single<Entity, With<A>>,
    mut commands: Commands,
    sprite_animations: Res<SpriteAnimations<T>>,
    collision_data: Res<CollisionDataCache<T>>,
    shadow: Res<MeshLightShadow<T>>,
) where
    T: Character + Visible,
    A: Level,
{
    let animation_delay = animation_rng.random_range(ANIMATION_DELAY_RANGE_SECS);
    let (collider_shape, collider_width, collider_height, collider_y_offset) = (
        collision_data.shape.clone(),
        collision_data.width,
        collision_data.height,
        collision_data.y_offset,
    );

    let entity = commands
        .entity(event.entity)
        .insert((
            T::container_bundle(event.pos, animation_delay, -collider_y_offset),
            T::collider(collider_shape, collider_width, collider_height),
            WorldUiAnchor::HealthBar(Vec2::new(0., collider_height * 2.)),
            children![T::shadow_bundle(&shadow)],
        ))
        .with_children(|commands| {
            let mut animation = commands.spawn((
                T::animation_bundle(&sprite_animations.base),
                Transform::from_xyz(0., -collider_y_offset, 0.),
                AnimationBase,
            ));
            if let Some(floating) = &sprite_animations.floating {
                animation.with_children(|commands| {
                    commands.spawn((
                        T::animation_bundle(floating),
                        Transform::from_xyz(0., 0., LAYER_Z_DELTA),
                    ));
                });
            }
        })
        .id();

    // Add `entity` to level so that level handles despawning
    commands.entity(*level).add_child(entity);
}
