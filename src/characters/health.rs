/*
 * File: health.rs
 * Author: Leopold Johannes Meinel (leo@meinel.dev)
 * -----
 * Copyright (c) 2026 Leopold Johannes Meinel & contributors
 * SPDX ID: Apache-2.0
 * URL: https://www.apache.org/licenses/LICENSE-2.0
 */

use bevy::prelude::*;

pub(super) fn plugin(app: &mut App) {
    app.add_observer(on_damage);
}

/// Health that determines if a [`Component`] should be despawned.
#[derive(Component, Default)]
pub(crate) struct Health(pub(crate) f32);

/// Damage applied to [`Health`].
#[derive(Event)]
pub(crate) struct Damage {
    pub(crate) targets: Vec<Entity>,
    pub(crate) damage: f32,
}

/// Apply damage to [`Health`] and handle despawning.
fn on_damage(event: On<Damage>, mut target_query: Query<&mut Health>, mut commands: Commands) {
    for entity in &event.targets {
        let Ok(mut health) = target_query.get_mut(*entity) else {
            continue;
        };
        health.0 -= event.damage;
        if health.0 <= 0. {
            commands.entity(*entity).despawn();
        }
    }
}
