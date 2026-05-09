use bevy::prelude::*;
use bevy_rapier2d::{parry::shape, prelude::*};

use crate::{characters::prelude::*, log::prelude::*, physics::prelude::*, render::prelude::*};

/// Direction the [`Character`] is aiming.
#[derive(Component, Deref, DerefMut)]
pub(crate) struct AimDirection(pub(crate) Vec2);
impl Default for AimDirection {
    fn default() -> Self {
        Self(Vec2::ZERO)
    }
}

/// Relevant data for an attack.
pub(crate) struct AttackData {
    pub(crate) _name: &'static str,
    pub(crate) damage: f32,
    /// Attack range in pixels.
    ///
    /// First value is width, second is height.
    pub(crate) range: Vec2,
    /// Cooldown in seconds after attack is done
    pub(crate) cooldown_secs: f32,
}

/// [`Message`] that is written if the source [`Entity`] has attacked.
#[derive(Message)]
pub(crate) enum Attack {
    Melee(Entity),
    // TODO: Implement this.
    #[allow(dead_code)]
    Ranged(Entity),
}
impl From<&InitAttack> for Attack {
    fn from(attack: &InitAttack) -> Self {
        match attack {
            InitAttack::Melee(entity) => Self::Melee(*entity),
            InitAttack::Ranged(entity) => Self::Ranged(*entity),
        }
    }
}

/// [`Message`] that is written if the source [`Entity`] has attacked.
///
/// This initiates [`Attack`].
#[derive(Message)]
pub(crate) enum InitAttack {
    Melee(Entity),
    // TODO: Implement this.
    #[allow(dead_code)]
    Ranged(Entity),
}

/// [`EntityEvent`] that is triggered if the contained [`Entity`]'s next [`Attack`] should be delayed.
#[derive(EntityEvent)]
pub(crate) struct DelayAttack {
    pub(crate) entity: Entity,
    pub(crate) cooldown_secs: f32,
}

/// Stats for [`Attack`]
#[derive(Component, Default)]
pub(crate) struct AttackStats {
    pub(crate) _attacks: Vec<AttackData>,
    pub(crate) damage_factor: f32,
    pub(crate) melee: Option<AttackData>,
    pub(crate) _ranged: Option<AttackData>,
}

/// Timer that tracks [`Attack`]s
#[derive(Component, Debug, Clone, PartialEq, Reflect, Deref, DerefMut)]
#[reflect(Component)]
pub(crate) struct AttackTimer(pub(crate) Timer);

/// Simple punch [`Attack`] with short range
pub(crate) fn punch() -> AttackData {
    AttackData {
        _name: "punch",
        damage: 1.,
        range: Vec2::new(8., 16.),
        cooldown_secs: 0.5,
    }
}

/// On [`Attack::Melee`], trigger [`Damage`] on [Entity]s within range.
pub(super) fn on_melee_attack<T>(
    mut reader: MessageReader<Attack>,
    target_query: Query<&Health>,
    origin_query: Query<(&Transform, &AimDirection, &AttackStats), With<T>>,
    mut commands: Commands,
    collision_data: Res<CollisionDataCache<T>>,
    rapier_context: ReadRapierContext,
    particle: Res<ParticleHandle<MeleeParticle>>,
) where
    T: Visible,
{
    for attack in reader.read() {
        let Attack::Melee(entity) = attack else {
            continue;
        };

        let rapier_context = rapier_context.single().expect(ERR_INVALID_RAPIER_CONTEXT);
        let (transform, direction, stats) = origin_query.get(*entity).expect(ERR_INVALID_ATTACKER);
        let Some(melee) = &stats.melee else {
            warn_once!("{}", WARN_INVALID_ATTACK_DATA);
            return;
        };

        // Cast ray to determine boundary of `Collider`
        // NOTE: We have to add an offset to max_toi to ensure that the ray reaches the boundary.
        let max_toi = (collision_data.width / 2.).max(collision_data.height / 2.) + 1.;
        // Filter for the source itself
        let filter = &|e| e == *entity;
        let filter = QueryFilter::exclude_dynamic()
            .exclude_sensors()
            .predicate(filter);
        let pos = transform.translation.xy();
        let Some((_, extent)) = rapier_context.cast_ray(pos, direction.0, max_toi, false, filter)
        else {
            return;
        };

        // Collect all entities within attack range
        let shape_half_size = melee.range / 2.;
        let offset = direction.0 * (extent + shape_half_size.x);
        let shape_pos = pos + offset;
        let shape_rot = direction.0.to_angle();
        let shape = shape::Cuboid::new(shape_half_size.into());
        // Filter for anything that is not the source
        let filter = QueryFilter::exclude_dynamic()
            .exclude_sensors()
            .exclude_rigid_body(*entity);
        let mut targets = Vec::new();
        rapier_context.intersect_shape(shape_pos, shape_rot, &shape, filter, |e| {
            if target_query.contains(e) {
                targets.push(e);
            }
            true
        });

        // Apply attack
        let damage = stats.damage_factor * melee.damage;
        commands.trigger(Damage { targets, damage });
        commands.trigger(SpawnChildParticleOnce::<MeleeParticle>::new(
            *entity,
            offset.extend(BASE_Z_DELTA),
            particle.handle.clone(),
        ));
    }
}

/// Insert [`AttackTimer`] to delay [`Attack`]s.
pub(super) fn on_delay_attack(event: On<DelayAttack>, mut commands: Commands) {
    // NOTE: Using try here is necessary since the entity might have been despawned elsewhere.
    commands
        .entity(event.entity)
        .try_insert(AttackTimer(Timer::from_seconds(
            event.cooldown_secs,
            TimerMode::Once,
        )));
}
