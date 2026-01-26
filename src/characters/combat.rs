/*
 * File: combat.rs
 * Author: Leopold Johannes Meinel (leo@meinel.dev)
 * -----
 * Copyright (c) 2026 Leopold Johannes Meinel & contributors
 * SPDX ID: Apache-2.0
 * URL: https://www.apache.org/licenses/LICENSE-2.0
 */

use bevy::{platform::collections::HashSet, prelude::*};
use bevy_rapier2d::{parry::shape, prelude::*};
use ordered_float::OrderedFloat;

use crate::{
    AppSystems,
    camera::OVERLAY_Z,
    characters::{Character, CollisionDataCache, Health, Movement, player::Player},
    logging::{
        error::{ERR_INVALID_ATTACKER, ERR_INVALID_RAPIER_CONTEXT},
        warn::{WARN_INCOMPLETE_COLLISION_DATA, WARN_INVALID_ATTACK},
    },
    visual::particles::{ParticleCombatHit, ParticleHandle, spawn_particle_once},
};

pub(super) fn plugin(app: &mut App) {
    // Tick timers
    app.add_systems(Update, tick_attack_timer.in_set(AppSystems::TickTimers));

    // Apply combat in observers
    app.add_observer(apply_melee::<Player>);
}

/// Relevant data for an attack.
#[derive(Default, PartialEq, Eq, Hash)]
pub(crate) struct Attack {
    pub(crate) name: String,
    pub(crate) damage: OrderedFloat<f32>,
    /// Attack range in pixels.
    ///
    /// First value is width, second is height.
    pub(crate) range: (OrderedFloat<f32>, OrderedFloat<f32>),
    /// Cooldown in seconds after attack is done
    pub(crate) cooldown_secs: OrderedFloat<f32>,
}

/// [`EntityEvent`] that is triggered if the contained [`Entity`] has attacked.
#[derive(EntityEvent)]
pub(crate) struct Attacked {
    pub(crate) entity: Entity,
    pub(crate) direction: Vec2,
}

/// Controller for combat
#[derive(Component, Default)]
pub(crate) struct CombatController {
    pub(crate) _attacks: HashSet<Attack>,
    pub(crate) damage_factor: f32,
    pub(crate) melee: Option<Attack>,
    pub(crate) _ranged: Option<Attack>,
}

/// Timer that tracks [`Attack`]s
#[derive(Component, Debug, Clone, PartialEq, Reflect)]
#[reflect(Component)]
pub(crate) struct AttackTimer(pub(crate) Timer);

/// Simple punch [`Attack`] with short range
pub(crate) fn punch() -> Attack {
    Attack {
        name: "punch".to_string(),
        damage: OrderedFloat(1.),
        range: (OrderedFloat(8.), OrderedFloat(16.)),
        cooldown_secs: OrderedFloat(0.5),
    }
}

/// On a triggered [`Attacked`], apply melee damage to [Entity]s within range.
///
/// ## Traits
///
/// - `T` must implement [`Character`] and is used as the character associated with a [`CombatController`].
fn apply_melee<T>(
    event: On<Attacked>,
    mut target_query: Query<&mut Health>,
    origin_query: Query<(&Transform, &Movement, &CombatController), With<T>>,
    mut commands: Commands,
    collision_data: Res<CollisionDataCache<T>>,
    rapier_context: ReadRapierContext,
    particle_handle: Res<ParticleHandle<ParticleCombatHit>>,
) where
    T: Character,
{
    let rapier_context = rapier_context.single().expect(ERR_INVALID_RAPIER_CONTEXT);
    let (Some(width), Some(height)) = (collision_data.width, collision_data.height) else {
        warn_once!("{}", WARN_INCOMPLETE_COLLISION_DATA);
        return;
    };

    let (origin, event_direction) = (event.entity, event.direction);
    let (transform, movement, controller) = origin_query.get(origin).expect(ERR_INVALID_ATTACKER);
    let Some(melee) = &controller.melee else {
        warn_once!("{}", WARN_INVALID_ATTACK);
        return;
    };
    let direction = if event_direction == Vec2::ZERO {
        movement.facing
    } else {
        event_direction
    };

    // Cast ray to determine boundary of `Collider`
    // NOTE: We have to add an offset to max_toi to ensure that the ray reaches the boundary.
    let max_toi = (width / 2.).max(height / 2.) + 1.;
    // Filter for `origin` itself
    let filter = &|e| e == origin;
    let filter = QueryFilter::exclude_dynamic()
        .exclude_sensors()
        .predicate(filter);
    let pos = transform.translation.xy();
    let Some((_, extent)) = rapier_context.cast_ray(pos, direction, max_toi, false, filter) else {
        return;
    };

    // Collect all entities within attack range
    let shape_half_size = Vec2::new(melee.range.0.into_inner(), melee.range.1.into_inner()) / 2.;
    let offset = extent + shape_half_size.x;
    let shape_pos = pos + direction * offset;
    let shape_rot = direction.to_angle();
    let shape = shape::Cuboid::new(shape_half_size.into());
    // Filter for anything that is not `origin`
    let filter = QueryFilter::exclude_dynamic()
        .exclude_sensors()
        .exclude_rigid_body(origin);
    let mut targets = Vec::new();
    rapier_context.intersect_shape(shape_pos, shape_rot, &shape, filter, |e| {
        if target_query.contains(e) {
            targets.push(e);
        }
        true
    });

    // Apply attack
    let damage = controller.damage_factor * melee.damage.into_inner();
    let cooldown_secs = melee.cooldown_secs.into_inner();
    apply_attack(
        &mut target_query,
        &mut commands,
        origin,
        targets,
        damage,
        cooldown_secs,
    );
    spawn_particle_once(
        &mut commands,
        shape_pos.extend(OVERLAY_Z),
        &*particle_handle,
    );
}

/// Apply damage to [`Health`], handle despawning and insert [`AttackTimer`].
fn apply_attack(
    target_query: &mut Query<&mut Health>,
    commands: &mut Commands,
    origin: Entity,
    targets: Vec<Entity>,
    damage: f32,
    cooldown_secs: f32,
) {
    // Apply damage to health and handle despawning.
    for entity in targets {
        let Ok(mut health) = target_query.get_mut(entity) else {
            continue;
        };
        health.0 -= damage;
        if health.0 <= 0. {
            commands.entity(entity).despawn();
        }
    }

    // Insert `AttackTimer`.
    commands
        .entity(origin)
        .insert(AttackTimer(Timer::from_seconds(
            cooldown_secs,
            TimerMode::Once,
        )));
}

/// Tick [`AttackTimer`]
fn tick_attack_timer(mut query: Query<&mut AttackTimer>, time: Res<Time>) {
    for mut timer in &mut query {
        timer.0.tick(time.delta());
    }
}
