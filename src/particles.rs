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
    camera::BACKGROUND_Z_DELTA,
    characters::player::Player,
    levels::overworld::spawn_overworld,
    screens::Screen,
    visuals::{TextureInfoCache, Visible},
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
}

/// Marker component for particles
#[derive(Component, Default, Reflect)]
pub(crate) struct Particle;

// FIXME: Add logic to hide, accelerate particles based on movement.
/// Add dust particle for walking.
///
/// ## Traits
///
/// - `T` must implement [`Visible`].
pub(crate) fn add_dust_walking<T>(
    query: Query<Entity, With<T>>,
    mut commands: Commands,
    assets: Res<AssetServer>,
    texture_info: Res<TextureInfoCache<T>>,
) where
    T: Visible,
{
    let texture_offset = texture_info.size.y as f32 / 2.;

    for entity in query {
        let child = commands
            .spawn((
                Particle,
                ParticleSpawner::default(),
                ParticleEffectHandle(assets.load("data/particles/dust-walking.particle.ron")),
                Transform::from_translation(Vec3::new(0., -texture_offset, BACKGROUND_Z_DELTA)),
            ))
            .id();
        commands.entity(entity).add_child(child);
    }
}
