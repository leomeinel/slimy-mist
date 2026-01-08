/*
 * File: particles.rs
 * Author: Leopold Johannes Meinel (leo@meinel.dev)
 * -----
 * Copyright (c) 2026 Leopold Johannes Meinel & contributors
 * SPDX ID: Apache-2.0
 * URL: https://www.apache.org/licenses/LICENSE-2.0
 */

use bevy::prelude::*;
use bevy_enoki::prelude::*;

use crate::{
    characters::{Character, player::Player},
    procgen::ProcGenInit,
};

pub(super) fn plugin(app: &mut App) {
    // Add library plugins
    app.add_plugins(EnokiPlugin);

    // Add particle systems
    app.add_systems(OnEnter(ProcGenInit(true)), add_dust_walking::<Player>);
}

/// Marker component for particles
#[derive(Component, Default, Reflect)]
pub(crate) struct Particle;

/// Add dust particle for walking.
///
/// ## Traits
///
/// - `T` must implement [`Character`].
pub(crate) fn add_dust_walking<T>(
    query: Query<Entity, With<T>>,
    mut commands: Commands,
    assets: Res<AssetServer>,
) where
    T: Character,
{
    for entity in query {
        let particle = commands
            .spawn((
                Particle,
                ParticleSpawner::default(),
                ParticleEffectHandle(assets.load("data/particles/dust-walking.particle.ron")),
                Transform::from_translation(Vec3::ZERO),
            ))
            .id();
        commands.entity(entity).add_child(particle);
    }
}
