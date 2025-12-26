/*
 * File: dev_tools.rs
 * Author: Leopold Johannes Meinel (leo@meinel.dev)
 * -----
 * Copyright (c) 2025 Leopold Johannes Meinel & contributors
 * SPDX ID: Apache-2.0
 * URL: https://www.apache.org/licenses/LICENSE-2.0
 * -----
 * Heavily inspired by: https://github.com/TheBevyFlock/bevy_new_2d
 */

//! Development tools for the game. This plugin is only enabled in dev builds.

use bevy::{
    dev_tools::states::log_transitions, input::common_conditions::input_just_pressed, prelude::*,
};
use bevy_northstar::prelude::*;
use bevy_prng::WyRand;
use bevy_rand::{global::GlobalRng, traits::ForkableSeed as _};
use bevy_rapier2d::render::{DebugRenderContext, RapierDebugRenderPlugin};
use rand::Rng;

use crate::{
    characters::{Character, npc::Slime},
    levels::overworld::OverworldProcGen,
    logging::error::{ERR_INVALID_MINIMUM_CHUNK_POS, ERR_LOADING_TILE_DATA},
    procgen::{CHUNK_SIZE, ProcGenController, ProcGenState, ProcGenerated, TileData, TileHandle},
    screens::Screen,
};

pub(super) fn plugin(app: &mut App) {
    // Add rapier debug render
    app.add_plugins(RapierDebugRenderPlugin {
        enabled: false,
        ..default()
    });

    // Add north star debug plugin
    app.add_plugins(NorthstarDebugPlugin::<OrdinalNeighborhood>::default());

    // Setup debug rng
    app.add_systems(Startup, setup_rng);

    // Log `Screen` state transitions.
    app.add_systems(Update, log_transitions::<Screen>);

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
        )
            .run_if(state_changed::<Debugging>),
    );
    app.add_systems(
        OnEnter(Debugging(false)),
        despawn_debug_nav_grid.run_if(in_state(Screen::Gameplay)),
    );
    app.add_systems(
        OnEnter(ProcGenState::RebuildNavGrid),
        (
            spawn_debug_nav_grid::<OverworldProcGen>,
            spawn_debug_path::<Slime>,
        )
            .run_if(in_state(Debugging(true)).and(in_state(Screen::Gameplay))),
    );
}

/// Toggle key
const TOGGLE_KEY: KeyCode = KeyCode::Backquote;

/// Whether or not debugging is active
#[derive(States, Copy, Clone, Eq, PartialEq, Hash, Debug, Default)]
struct Debugging(bool);

/// Rng for debugging
#[derive(Component)]
struct DebugRng;

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

/// Despawn debug nav grid
fn despawn_debug_nav_grid(
    debug_nav_grid: Single<Entity, (With<DebugGrid>, Without<Grid<OrdinalNeighborhood>>)>,
    mut commands: Commands,
) {
    commands.entity(debug_nav_grid.entity()).despawn();
}

/// Spawn debug nav grid
///
/// ## Traits
///
/// - `T` must implement [`ProcGenerated`] and is used as a level's procedurally generated item.
fn spawn_debug_nav_grid<T>(
    debug_nav_grid: Option<
        Single<&mut DebugOffset, (With<DebugGrid>, Without<Grid<OrdinalNeighborhood>>)>,
    >,
    nav_grid: Single<Entity, (With<Grid<OrdinalNeighborhood>>, Without<DebugGrid>)>,
    mut commands: Commands,
    controller: Res<ProcGenController<T>>,
    data: Res<Assets<TileData<T>>>,
    handle: Res<TileHandle<T>>,
) where
    T: ProcGenerated,
{
    // Get data from `TileData` with `TileHandle`
    let data = data.get(handle.0.id()).expect(ERR_LOADING_TILE_DATA);
    let tile_size = Vec2::new(data.tile_height, data.tile_width);

    // Determine world pos from minimum chunk pos
    let min_chunk_pos = controller
        .positions
        .values()
        .min_by_key(|pos| (pos.x, pos.y))
        .expect(ERR_INVALID_MINIMUM_CHUNK_POS);
    let world_pos = Vec2::new(
        min_chunk_pos.x as f32 * CHUNK_SIZE.x as f32 * tile_size.x,
        min_chunk_pos.y as f32 * CHUNK_SIZE.y as f32 * tile_size.y,
    );

    // Return if debug grid is present and update offset
    if let Some(mut offset) = debug_nav_grid {
        offset.0 = world_pos.extend(0.);
        return;
    }

    // Spawn debug grid
    let debug = commands
        .spawn((
            DebugGridBuilder::new(tile_size.x as u32, tile_size.y as u32)
                .enable_chunks()
                .enable_entrances()
                .build(),
            DebugOffset(world_pos.extend(0.)),
        ))
        .id();
    commands.entity(nav_grid.entity()).add_child(debug);
}

/// Spawn debug path for [`Character`]
///
/// ## Traits
///
/// - `T` must implement '[`Character`]'.
fn spawn_debug_path<T>(
    mut debug_rng: Single<&mut WyRand, With<DebugRng>>,
    characters: Query<Entity, (With<T>, With<AgentPos>, Without<DebugPath>)>,
    mut commands: Commands,
) where
    T: Character,
{
    for entity in characters {
        let color = Color::srgb(
            debug_rng.random_range(0.0..=1.),
            debug_rng.random_range(0.0..=1.),
            debug_rng.random_range(0.0..=1.),
        );

        // Insert debug path with random color
        commands.entity(entity).insert(DebugPath::new(color));
    }
}

/// Spawn [`DebugRng`] by forking [`GlobalRng`]
fn setup_rng(mut global: Single<&mut WyRand, With<GlobalRng>>, mut commands: Commands) {
    commands.spawn((DebugRng, global.fork_seed()));
}
