/*
 * File: particles.rs
 * Author: Leopold Johannes Meinel (leo@meinel.dev)
 * -----
 * Copyright (c) 2026 Leopold Johannes Meinel & contributors
 * SPDX ID: Apache-2.0
 * URL: https://www.apache.org/licenses/LICENSE-2.0
 */

use std::marker::PhantomData;

use bevy::{platform::collections::HashMap, prelude::*};
use bevy_enoki::prelude::*;

use crate::{
    AppSystems,
    camera::BACKGROUND_Z_DELTA,
    characters::{
        Character, VisualMap,
        animations::{AnimationController, AnimationState},
        player::Player,
    },
    levels::overworld::spawn_overworld,
    logging::error::{ERR_INVALID_PARTICLE_MAP, ERR_INVALID_VISUAL_MAP},
    screens::Screen,
    visual::{TextureInfoCache, Visible},
};

pub(super) fn plugin(app: &mut App) {
    // Add library plugins
    app.add_plugins(EnokiPlugin);

    // Add particle systems
    // FIXME: Think of using a SystemSet here instead that includes spawn_overworld
    app.add_systems(
        OnEnter(Screen::Gameplay),
        add_dust_walking::<Player>.after(spawn_overworld),
    );

    // Update particles for character
    app.add_systems(
        Update,
        update_character_particles::<Player, ParticleDustWalking>
            .after(spawn_overworld)
            .run_if(in_state(Screen::Gameplay)),
    );

    // Tick timers
    app.add_systems(Update, tick_particle_timer.in_set(AppSystems::TickTimers));
}

/// Applies to anything that is considered a particle.
pub(crate) trait Particle
where
    Self: Component + Default,
{
    fn is_active(&self, state: AnimationState) -> bool;
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

/// Marker component for dust walking particles
#[derive(Component, Default)]
pub(crate) struct ParticleDustWalking(AnimationState);
impl Particle for ParticleDustWalking {
    fn is_active(&self, animation_state: AnimationState) -> bool {
        self.0 == animation_state
    }
}

/// Map of characters to their particles
#[derive(Resource, Default)]
pub(crate) struct ParticleMap<T>
where
    T: Particle,
{
    pub(crate) map: HashMap<Entity, Entity>,
    _phantom: PhantomData<T>,
}

/// Timer that tracks particles
#[derive(Component, Debug, Clone, PartialEq, Reflect)]
#[reflect(Component)]
struct ParticleTimer(Timer);

/// Interval for [`ParticleDustWalking`].
const DUST_WALKING_INTERVAL_SECS: f32 = 0.5;

/// Add [`ParticleDustWalking`].
///
/// ## Traits
///
/// - `T` must implement [`Visible`].
fn add_dust_walking<T>(
    query: Query<Entity, With<T>>,
    mut commands: Commands,
    mut particle_map: ResMut<ParticleMap<ParticleDustWalking>>,
    assets: Res<AssetServer>,
    texture_info: Res<TextureInfoCache<T>>,
) where
    T: Visible,
{
    let texture_offset = texture_info.size.y as f32 / 2.;

    for container in query {
        let particle = commands
            .spawn((
                ParticleDustWalking(AnimationState::Walk),
                ParticleTimer(Timer::from_seconds(
                    DUST_WALKING_INTERVAL_SECS,
                    TimerMode::Repeating,
                )),
                ParticleSpawner::default(),
                NoAutoAabb,
                ParticleSpawnerState {
                    active: false,
                    ..default()
                },
                ParticleEffectHandle(assets.load("data/particles/dust-walking.particle.ron")),
                Transform::from_translation(Vec3::new(0., -texture_offset, BACKGROUND_Z_DELTA)),
            ))
            .id();
        commands.entity(container).add_child(particle);
        particle_map.map.insert(container, particle);
    }
}

/// Update particle for [`Character`]s
///
/// ## Traits
///
/// - `T` must implement [`Character`] and [`Visible`].
/// - `A` must implement [`Particle`].
fn update_character_particles<T, A>(
    parent_query: Query<Entity, With<T>>,
    mut child_particle_query: Query<
        (
            &ParticleDustWalking,
            &ParticleTimer,
            &mut ParticleSpawnerState,
        ),
        (With<A>, Without<T>),
    >,
    mut child_visual_query: Query<&mut AnimationController, Without<T>>,
    particle_map: Res<ParticleMap<A>>,
    visual_map: Res<VisualMap>,
) where
    T: Character + Visible,
    A: Particle,
{
    for container in parent_query {
        let particle = particle_map
            .map
            .get(&container)
            .expect(ERR_INVALID_PARTICLE_MAP);
        let (particle, timer, mut state) = child_particle_query
            .get_mut(*particle)
            .expect(ERR_INVALID_PARTICLE_MAP);

        // Continue if timer has not finished
        if !timer.0.just_finished() {
            continue;
        }

        let visual = visual_map.0.get(&container).expect(ERR_INVALID_VISUAL_MAP);
        let animation_controller = child_visual_query
            .get_mut(*visual)
            .expect(ERR_INVALID_VISUAL_MAP);

        state.set_new_active(particle.is_active(animation_controller.state));
    }
}

/// Tick [`ParticleTimer`]
fn tick_particle_timer(mut query: Query<&mut ParticleTimer>, time: Res<Time>) {
    for mut timer in &mut query {
        timer.0.tick(time.delta());
    }
}
