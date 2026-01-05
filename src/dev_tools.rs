/*
 * File: dev_tools.rs
 * Author: Leopold Johannes Meinel (leo@meinel.dev)
 * -----
 * Copyright (c) 2026 Leopold Johannes Meinel & contributors
 * SPDX ID: Apache-2.0
 * URL: https://www.apache.org/licenses/LICENSE-2.0
 * -----
 * Heavily inspired by:
 * - https://github.com/TheBevyFlock/bevy_new_2d
 * - https://github.com/vleue/vleue_navigator
 */

//! Development tools for the game. This plugin is only enabled in dev builds.

use bevy::{
    color::palettes::tailwind, dev_tools::states::log_transitions,
    input::common_conditions::input_just_pressed, prelude::*,
};
use bevy_rapier2d::render::{DebugRenderContext, RapierDebugRenderPlugin};
use vleue_navigator::prelude::*;

use crate::{
    procgen::{ProcGenInit, ProcGenState},
    screens::Screen,
};

pub(super) fn plugin(app: &mut App) {
    // Add rapier debug render
    app.add_plugins(RapierDebugRenderPlugin {
        enabled: false,
        ..default()
    });

    // Log `Screen` state transitions.
    app.add_systems(Update, log_transitions::<Screen>);
    app.add_systems(Update, log_transitions::<ProcGenState>);
    app.add_systems(Update, log_transitions::<ProcGenInit>);

    // Set debugging state
    app.init_state::<Debugging>();
    app.add_systems(
        Update,
        toggle_debugging.run_if(input_just_pressed(TOGGLE_KEY)),
    );

    // Toggle debug overlays
    app.add_systems(
        Update,
        (
            toggle_debug_ui,
            toggle_debug_colliders.run_if(in_state(Screen::Gameplay)),
            toggle_debug_navmeshes.run_if(in_state(Screen::Gameplay)),
        )
            .run_if(state_changed::<Debugging>),
    );
    app.add_systems(
        Update,
        display_primitive_obstacles
            .run_if(in_state(Debugging(true)).and(in_state(Screen::Gameplay))),
    );
}

/// Toggle key
const TOGGLE_KEY: KeyCode = KeyCode::Backquote;

/// Whether or not debugging is active
#[derive(States, Copy, Clone, Eq, PartialEq, Hash, Debug, Default)]
struct Debugging(bool);

/// Toggle debugging
fn toggle_debugging(
    mut next_state: ResMut<NextState<Debugging>>,
    debug_state: Res<State<Debugging>>,
) {
    next_state.set(Debugging(!debug_state.0));
}

/// Toggle debug overlay for UI
fn toggle_debug_ui(mut options: ResMut<UiDebugOptions>, debug_state: Res<State<Debugging>>) {
    options.enabled = debug_state.0;
}

/// Toggle debug overlay for rapier colliders
fn toggle_debug_colliders(
    mut render_context: ResMut<DebugRenderContext>,
    debug_state: Res<State<Debugging>>,
) {
    render_context.enabled = debug_state.0;
}

/// Color for the debug navmesh
const DEBUG_NAVMESH_COLOR: Srgba = tailwind::PINK_600;

/// Toggle debug navmeshes
fn toggle_debug_navmeshes(
    debug_query: Query<Entity, With<NavMeshDebug>>,
    query: Query<Entity, With<ManagedNavMesh>>,
    mut commands: Commands,
) {
    // Remove debug navmeshes
    if !debug_query.is_empty() {
        for entity in debug_query {
            commands.entity(entity).remove::<NavMeshDebug>();
        }
        return;
    }

    // Insert debug navmeshes
    for entity in query {
        commands
            .entity(entity)
            .insert(NavMeshDebug(DEBUG_NAVMESH_COLOR.into()));
    }
}

/// Color for the debug obstacle used in the navmesh
const DEBUG_OBSTACLE_COLOR: Srgba = tailwind::CYAN_600;

/// Display [`PrimitiveObstacle`]s
fn display_primitive_obstacles(mut gizmos: Gizmos, query: Query<(&PrimitiveObstacle, &Transform)>) {
    for (prim, transform) in &query {
        match prim {
            PrimitiveObstacle::Rectangle(prim) => {
                gizmos.primitive_2d(
                    prim,
                    Isometry2d::new(
                        transform.translation.xy(),
                        Rot2::radians(transform.rotation.to_axis_angle().1),
                    ),
                    Color::from(DEBUG_OBSTACLE_COLOR),
                );
            }
            PrimitiveObstacle::Circle(prim) => {
                gizmos.primitive_2d(
                    prim,
                    Isometry2d::new(
                        transform.translation.xy(),
                        Rot2::radians(transform.rotation.to_axis_angle().1),
                    ),
                    Color::from(DEBUG_OBSTACLE_COLOR),
                );
            }
            PrimitiveObstacle::Ellipse(prim) => {
                gizmos.primitive_2d(
                    prim,
                    Isometry2d::new(
                        transform.translation.xy(),
                        Rot2::radians(transform.rotation.to_axis_angle().1),
                    ),
                    Color::from(DEBUG_OBSTACLE_COLOR),
                );
            }
            PrimitiveObstacle::CircularSector(prim) => {
                gizmos.primitive_2d(
                    prim,
                    Isometry2d::new(
                        transform.translation.xy(),
                        Rot2::radians(transform.rotation.to_axis_angle().1),
                    ),
                    Color::from(DEBUG_OBSTACLE_COLOR),
                );
            }
            PrimitiveObstacle::CircularSegment(prim) => {
                gizmos.primitive_2d(
                    prim,
                    Isometry2d::new(
                        transform.translation.xy(),
                        Rot2::radians(transform.rotation.to_axis_angle().1),
                    ),
                    Color::from(DEBUG_OBSTACLE_COLOR),
                );
            }
            PrimitiveObstacle::Capsule(prim) => {
                gizmos.primitive_2d(
                    prim,
                    Isometry2d::new(
                        transform.translation.xy(),
                        Rot2::radians(transform.rotation.to_axis_angle().1),
                    ),
                    Color::from(DEBUG_OBSTACLE_COLOR),
                );
            }
            PrimitiveObstacle::RegularPolygon(prim) => {
                gizmos.primitive_2d(
                    prim,
                    Isometry2d::new(
                        transform.translation.xy(),
                        Rot2::radians(transform.rotation.to_axis_angle().1),
                    ),
                    Color::from(DEBUG_OBSTACLE_COLOR),
                );
            }
            PrimitiveObstacle::Rhombus(prim) => {
                gizmos.primitive_2d(
                    prim,
                    Isometry2d::new(
                        transform.translation.xy(),
                        Rot2::radians(transform.rotation.to_axis_angle().1),
                    ),
                    Color::from(DEBUG_OBSTACLE_COLOR),
                );
            }
        }
    }
}
