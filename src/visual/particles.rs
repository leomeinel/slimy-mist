/*
 * File: particles.rs
 * Author: Leopold Johannes Meinel (leo@meinel.dev)
 * -----
 * Copyright (c) 2026 Leopold Johannes Meinel & contributors
 * SPDX ID: Apache-2.0
 * URL: https://www.apache.org/licenses/LICENSE-2.0
 */

use std::marker::PhantomData;

use bevy::prelude::*;
use bevy_enoki::prelude::*;

use crate::{
    AppSystems,
    animations::{AnimationCache, AnimationState},
    camera::BACKGROUND_Z_DELTA,
    characters::{Character, player::Player},
    logging::error::ERR_INVALID_CHILDREN,
    screens::{Screen, gameplay::InitGameplaySystems},
    visual::{TextureInfoCache, Visible},
};

pub(super) fn plugin(app: &mut App) {
    // Add library plugins
    app.add_plugins(EnokiPlugin);

    // Add particle systems
    app.add_systems(
        OnEnter(Screen::Gameplay),
        add_walking_dust::<Player>.after(InitGameplaySystems::Finalize),
    );

    // Update particles for character
    app.add_systems(
        Update,
        update_character_particles::<Player, ParticleWalkingDust>
            .after(InitGameplaySystems::Finalize)
            .run_if(in_state(Screen::Gameplay)),
    );

    // Tick timers
    app.add_systems(Update, tick_particle_timer.in_set(AppSystems::TickTimers));

    // Add observers spawning particles
    app.add_observer(on_spawn_particle_once::<ParticleMeleeAttack>);
}

/// Applies to anything that is considered a particle.
pub(crate) trait Particle
where
    Self: Component + Default,
{
    fn is_active(&self, _state: AnimationState) -> bool {
        true
    }
}

trait ParticleSpawnerExt {
    fn set_new_active(&mut self, new_active: bool);
}
impl ParticleSpawnerExt for ParticleSpawnerState {
    fn set_new_active(&mut self, new_active: bool) {
        if self.active != new_active {
            self.active = new_active;
        }
    }
}

/// Marker component for walking dust particles
#[derive(Component, Default)]
pub(crate) struct ParticleWalkingDust(AnimationState);
impl Particle for ParticleWalkingDust {
    fn is_active(&self, animation_state: AnimationState) -> bool {
        self.0 == animation_state
    }
}

/// Marker component for [`crate::characters::attack::MeleeAttack`] particles
#[derive(Component, Default)]
pub(crate) struct ParticleMeleeAttack;
impl Particle for ParticleMeleeAttack {}

#[derive(Event)]
pub(crate) struct SpawnParticleOnce {
    pub(crate) pos: Vec3,
    pub(crate) handle: Handle<Particle2dEffect>,
}

/// Handle for [`Particle2dEffect`] as a generic.
///
/// ## Traits
///
/// - `T` must implement [`Particle`] and is used as the associated particle type.
#[derive(Resource, Default)]
pub(crate) struct ParticleHandle<T>
where
    T: Particle,
{
    pub(crate) handle: Handle<Particle2dEffect>,
    pub(crate) _phantom: PhantomData<T>,
}

/// Timer that tracks particles
#[derive(Component, Debug, Clone, PartialEq, Reflect)]
#[reflect(Component)]
struct ParticleTimer(Timer);

/// Spawn and despawn a particle once.
///
/// ## Traits
///
/// - `T` must implement [`Particle`] and is used as the associated particle type.
pub(crate) fn on_spawn_particle_once<T>(event: On<SpawnParticleOnce>, mut commands: Commands)
where
    T: Particle,
{
    commands.spawn((
        T::default(),
        OneShot::Despawn,
        ParticleSpawner::default(),
        NoAutoAabb,
        Transform::from_translation(event.pos),
        ParticleEffectHandle(event.handle.clone()),
    ));
}

/// Interval for [`ParticleWalkingDust`].
const WALKING_DUST_INTERVAL_SECS: f32 = 0.5;

/// Add [`ParticleWalkingDust`].
///
/// ## Traits
///
/// - `T` must implement [`Visible`].
fn add_walking_dust<T>(
    query: Query<Entity, With<T>>,
    mut commands: Commands,
    texture_info: Res<TextureInfoCache<T>>,
    handle: Res<ParticleHandle<ParticleWalkingDust>>,
) where
    T: Visible,
{
    let texture_offset = texture_info.size.y as f32 / 2.;

    for container in query {
        let particle = commands
            .spawn((
                ParticleWalkingDust(AnimationState::Walk),
                ParticleTimer(Timer::from_seconds(
                    WALKING_DUST_INTERVAL_SECS,
                    TimerMode::Repeating,
                )),
                ParticleSpawner::default(),
                NoAutoAabb,
                ParticleSpawnerState {
                    active: false,
                    ..default()
                },
                ParticleEffectHandle(handle.handle.clone()),
                Transform::from_translation(Vec3::new(0., -texture_offset, BACKGROUND_Z_DELTA)),
            ))
            .id();
        commands.entity(container).add_child(particle);
    }
}

/// Update particle for [`Character`]s
///
/// ## Traits
///
/// - `T` must implement [`Character`] and [`Visible`].
/// - `A` must implement [`Particle`].
fn update_character_particles<T, A>(
    character_query: Query<(&mut AnimationCache, &Children), With<T>>,
    mut particle_query: Query<
        (
            &ParticleWalkingDust,
            &ParticleTimer,
            &mut ParticleSpawnerState,
        ),
        With<A>,
    >,
) where
    T: Character + Visible,
    A: Particle,
{
    for (cache, children) in character_query {
        let child = children
            .iter()
            .find(|e| particle_query.contains(*e))
            .expect(ERR_INVALID_CHILDREN);
        let (particle, timer, mut state) =
            particle_query.get_mut(child).expect(ERR_INVALID_CHILDREN);

        // Continue if timer has not finished
        if !timer.0.just_finished() {
            continue;
        }

        state.set_new_active(particle.is_active(cache.state));
    }
}

/// Tick [`ParticleTimer`]
fn tick_particle_timer(mut query: Query<&mut ParticleTimer>, time: Res<Time>) {
    for mut timer in &mut query {
        timer.0.tick(time.delta());
    }
}
