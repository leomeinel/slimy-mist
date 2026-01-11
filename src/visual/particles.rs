/*
 * File: particles.rs
 * Author: Leopold Johannes Meinel (leo@meinel.dev)
 * -----
 * Copyright (c) 2026 Leopold Johannes Meinel & contributors
 * SPDX ID: Apache-2.0
 * URL: https://www.apache.org/licenses/LICENSE-2.0
 */

use std::marker::PhantomData;

use bevy::{platform::collections::HashMap, prelude::*, reflect::Reflectable};
use bevy_enoki::prelude::*;
use bevy_spritesheet_animation::prelude::SpritesheetAnimation;

use crate::{
    camera::BACKGROUND_Z_DELTA,
    characters::{
        Character, VisualMap,
        animations::{AnimationController, AnimationDataCache, AnimationState},
        player::Player,
    },
    levels::overworld::spawn_overworld,
    logging::{
        error::{ERR_INVALID_PARTICLE_MAP, ERR_INVALID_VISUAL_MAP},
        warn::WARN_INCOMPLETE_ANIMATION_DATA,
    },
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
        update_character::<Player, ParticleDustWalking>
            .after(spawn_overworld)
            .run_if(in_state(Screen::Gameplay)),
    );
}

pub(crate) trait Particle
where
    Self: Component + Default + Reflectable,
{
}

/// Marker component for dust walking particles
#[derive(Component, Default, Reflect)]
pub(crate) struct ParticleDustWalking;
impl Particle for ParticleDustWalking {}

/// Map of characters to their particles
#[derive(Resource, Default)]
pub(crate) struct ParticleMap<T>
where
    T: Particle,
{
    pub(crate) map: HashMap<Entity, Entity>,
    _phantom: PhantomData<T>,
}

/// Controller for [Particle]s
#[derive(Component, Default)]
pub(crate) struct ParticleController {
    /// Used to determine if we should send particle again
    pub(crate) frame: Option<usize>,
}

/// Add dust particle for walking.
///
/// ## Traits
///
/// - `T` must implement [`Visible`].
pub(crate) fn add_dust_walking<T>(
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
                ParticleDustWalking,
                ParticleController::default(),
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

// FIXME: Generics are currently somewhat useless here
// FIXME: Refactor for readability, this is ugly.
/// Update particle effect for [`Character`]s
///
/// ## Traits
///
/// - `T` must implement [`Character`] and [`Visible`].
pub(crate) fn update_character<T, A>(
    parent_query: Query<Entity, With<T>>,
    mut child_particle_query: Query<
        (&mut ParticleController, &mut ParticleSpawnerState),
        (With<A>, Without<T>),
    >,
    mut child_animation_query: Query<
        (&mut AnimationController, &mut SpritesheetAnimation),
        Without<T>,
    >,
    animation_data: Res<AnimationDataCache<T>>,
    particle_map: Res<ParticleMap<A>>,
    visual_map: Res<VisualMap>,
) where
    T: Character + Visible,
    A: Particle,
{
    let frame_set = (
        animation_data.walk_sound_frames.clone(),
        animation_data.jump_sound_frames.clone(),
        animation_data.fall_sound_frames.clone(),
    );
    let Some(frames) = frame_set.0 else {
        warn_once!("{}", WARN_INCOMPLETE_ANIMATION_DATA);
        return;
    };

    for container in parent_query {
        // Extract `animation_controller` from `child_query`
        let particle = particle_map
            .map
            .get(&container)
            .expect(ERR_INVALID_PARTICLE_MAP);
        let (mut particle_controller, mut state) = child_particle_query
            .get_mut(*particle)
            .expect(ERR_INVALID_PARTICLE_MAP);
        let visual = visual_map.0.get(&container).expect(ERR_INVALID_VISUAL_MAP);
        let (animation_controller, animation) = child_animation_query
            .get_mut(*visual)
            .expect(ERR_INVALID_VISUAL_MAP);
        let current_frame = animation.progress.frame;

        // Continue if particle has already been sent
        if let Some(frame) = particle_controller.frame
            && frame == current_frame
        {
            continue;
        }

        if animation_controller.state == AnimationState::Walk {
            if !frames.contains(&current_frame) {
                continue;
            }
            state.active = true;
            particle_controller.frame = Some(current_frame);
        } else {
            state.active = false;
            particle_controller.frame = None;
        }
    }
}
