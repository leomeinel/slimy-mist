/*
 * File: nav.rs
 * Author: Leopold Johannes Meinel (leo@meinel.dev)
 * -----
 * Copyright (c) 2026 Leopold Johannes Meinel & contributors
 * SPDX ID: Apache-2.0
 * URL: https://www.apache.org/licenses/LICENSE-2.0
 */

// FIXME: We currently have a few bugs with this:
//        - When transitioning to a new chunks, AgentPos does not seem to update in time, therefore characters move a chunk up.
//        - When goal walks, pathfinding is delayed. This is due to `find_path` reinserting NextPos, which prevents this from running.
//        - Characters tend to walk very long paths when adjusting for changes in the goal pos. Sometimes moving to the far edges of the
//          map even though the goal is in an adjacent chunk.
//        - Most of these and a lot more issues are most likely due to scheduling
//        - The TrackGoalTimer should trigger an update if it finishes even if goal is not moving.
//        - Also sometimes when characters have already been despawned, we are still trying to apply pathfinding which causes a panic!()
//        - This also needs a lot of performance and requires optimization.
//          For 100 characters, I am barely dipping below 60fps in debug builds, as far as I know this is not too much of a concern.
//          However, the current behavior seems quite unstable. Sometimes even dipping to 50 or below when everything is idle.
//          CPU: AMD Ryzen 7 5700U (16) @ 4.37 GHz; GPU: AMD Lucienne [Integrated]

use std::ops::Range;

use bevy::prelude::*;
use bevy_northstar::prelude::*;
use bevy_prng::WyRand;
use bevy_rand::{global::GlobalRng, traits::ForkableSeed as _};
use bevy_rapier2d::prelude::*;
use rand::Rng as _;

use crate::{
    AppSystems,
    characters::{
        Character, Movement, MovementSpeed, VisualMap,
        animations::{AnimationController, AnimationState},
        npc::Slime,
        player::Player,
    },
    levels::overworld::OverworldProcGen,
    logging::error::{ERR_INVALID_VISUAL_MAP, ERR_LOADING_TILE_DATA},
    procgen::{
        CHUNK_SIZE, ProcGenController, ProcGenInit, ProcGenState, ProcGenerated, TileData,
        TileHandle,
    },
    screens::Screen,
};

pub(super) fn plugin(app: &mut App) {
    // Add rng for navigation
    app.add_systems(Startup, setup_rng);

    // Trigger position refresh when new chunks are generated
    app.add_systems(
        OnEnter(ProcGenState::UpdateNav),
        (refresh_pos::<Player>, refresh_pos::<Slime>).run_if(in_state(ProcGenInit(true))),
    );

    // Update pathfinding
    app.add_systems(
        Update,
        (
            (
                update_pos::<Player, OverworldProcGen, true>,
                update_pos::<Slime, OverworldProcGen, false>,
            )
                .before(PathingSet),
            find_path::<Slime, Player>,
            apply_path::<Slime, OverworldProcGen>.after(PathingSet),
        )
            .run_if(in_state(ProcGenInit(true)).and(in_state(Screen::Gameplay))),
    );

    // Tick timers
    app.add_systems(Update, tick_track_goal_timer.in_set(AppSystems::TickTimers));

    // Handle goal movement
    app.add_observer(on_goal_moved::<Slime>);
}

/// Current state of navigation
#[derive(Event)]
pub(crate) struct GoalMoved;

/// Current state of navigation per [`Entity`]
#[derive(Clone, Copy, PartialEq, Debug, Default)]
pub(crate) enum NavState {
    #[default]
    UpdatePos,
    FindPath,
    ApplyPath,
    None,
}

/// Controller for navigation
#[derive(Component, Default)]
pub(crate) struct NavController {
    pub(crate) state: NavState,
    pub(crate) track_goal_done: bool,
}

/// Timer for goal tracking
#[derive(Component, Debug, Clone, PartialEq, Reflect)]
#[reflect(Component)]
pub(crate) struct TrackGoalTimer(pub(crate) Timer);

/// Rng for navigation
#[derive(Component)]
pub(crate) struct NavRng;

/// Update navigation [`Grid`] position one [`Character`] at a time.
///
/// ## Traits
///
/// - `T` must implement [`Character`].
/// - `A` must implement [`ProcGenerated`] and is used as a level's procedurally generated item.
/// - `const IS_GOAL` defines whether [`GoalMoved`] will be triggered and proceeds to [`NavState::None`].
fn update_pos<T, A, const IS_GOAL: bool>(
    grid: Single<Entity, With<Grid<OrdinalNeighborhood>>>,
    mut character_query: Query<
        (
            Entity,
            &mut NavController,
            &Transform,
            Option<&mut AgentPos>,
        ),
        With<T>,
    >,
    mut commands: Commands,
    mut next_state: ResMut<NextState<ProcGenState>>,
    data: Res<Assets<TileData<A>>>,
    handle: Res<TileHandle<A>>,
    procgen_controller: Res<ProcGenController<A>>,
    state: Res<State<ProcGenState>>,
    mut tile_size: Local<Option<Vec2>>,
) where
    T: Character,
    A: ProcGenerated,
{
    // Find first entity matching state
    let Some((entity, mut controller, transform, mut agent_pos)) = character_query
        .iter_mut()
        .find(|(_, c, _, _)| c.state == NavState::UpdatePos)
    else {
        return;
    };

    // Init local values
    let tile_size = tile_size.unwrap_or_else(|| {
        let data = data.get(handle.0.id()).expect(ERR_LOADING_TILE_DATA);
        let value = Vec2::new(data.tile_height, data.tile_width);
        *tile_size = Some(value);
        value
    });

    // Calculate `target_agent_pos` by converting translation to agent_pos and subtracting `min_chunk_pos`
    let target_agent_pos = (transform.translation.xy() / tile_size
        - procgen_controller.min_chunk_pos().as_vec2() * CHUNK_SIZE.as_vec2())
    .floor()
    .as_uvec2();
    // Set agent_pos
    if let Some(agent_pos) = agent_pos.as_mut() {
        if agent_pos.0 != target_agent_pos.extend(0) {
            agent_pos.0 = target_agent_pos.extend(0);
        }
    } else {
        commands.entity(entity).insert((
            AgentPos(target_agent_pos.extend(0)),
            AgentOfGrid(grid.entity()),
        ));
    };

    // Proceed to next `NavState`/`ProcGenState`
    if IS_GOAL {
        commands.trigger(GoalMoved);
        controller.state = NavState::None;
    } else {
        controller.state = NavState::FindPath;
    }
    if state.get() == &ProcGenState::None {
        next_state.set(ProcGenState::Despawn);
    }
}

/// Insert [`Pathfind`] to one [`Character`] at a time.
///
/// ## Traits
///
/// - `T` must implement [`Character`] and is used as the follower.
/// - `A` must implement [`Character`] and is used as the goal.
fn find_path<T, A>(
    goal: Single<&AgentPos, (With<A>, Without<T>)>,
    mut character_query: Query<(Entity, &mut NavController), With<T>>,
    mut commands: Commands,
) where
    T: Character,
    A: Character,
{
    // Find first entity matching state
    let Some((entity, mut controller)) = character_query
        .iter_mut()
        .find(|(_, c)| c.state == NavState::FindPath)
    else {
        return;
    };

    // Insert `Pathfind`
    commands
        .entity(entity)
        .insert(Pathfind::new(goal.0).mode(PathfindMode::Waypoints));

    controller.state = NavState::ApplyPath;
}

/// Maximum distance to goal in tiles
const MAX_GOAL_TILE_DIST: f32 = 1.;

/// Apply path from [`NextPos`] and [`Pathfind`] via [`KinematicCharacterController`].
///
/// This applies to all [`Character`]s at once.
///
/// ## Traits
///
/// - `T` must implement [`Character`].
/// - `A` must implement [`ProcGenerated`] and is used as a level's procedurally generated item.
fn apply_path<T, A>(
    parent_query: Query<
        (
            Entity,
            &mut KinematicCharacterController,
            &mut NavController,
            &Transform,
            &mut Movement,
            &MovementSpeed,
            &mut AgentPos,
            &NextPos,
            &Pathfind,
        ),
        With<T>,
    >,
    mut child_query: Query<&mut AnimationController, Without<T>>,
    mut commands: Commands,
    procgen_controller: Res<ProcGenController<A>>,
    data: Res<Assets<TileData<A>>>,
    handle: Res<TileHandle<A>>,
    time: Res<Time>,
    visual_map: Res<VisualMap>,
    mut tile_size: Local<Option<Vec2>>,
) where
    T: Character,
    A: ProcGenerated,
{
    // Init local values
    let tile_size = tile_size.unwrap_or_else(|| {
        let data = data.get(handle.0.id()).expect(ERR_LOADING_TILE_DATA);
        let value = Vec2::new(data.tile_height, data.tile_width);
        *tile_size = Some(value);
        value
    });

    let world_pos = procgen_controller.min_chunk_pos().as_vec2() * CHUNK_SIZE.as_vec2() * tile_size;

    for (
        entity,
        mut character_controller,
        mut nav_controller,
        transform,
        mut movement,
        movement_speed,
        mut agent_pos,
        next_pos,
        path_find,
    ) in parent_query
    {
        // Extract `animation_controller` from `child_query`
        let visual = visual_map.0.get(&entity).expect(ERR_INVALID_VISUAL_MAP);
        let mut animation_controller = child_query.get_mut(*visual).expect(ERR_INVALID_VISUAL_MAP);

        // Continue if goal has been reached and set to idle
        let goal_world_pos = path_find.goal.xy().as_vec2() * tile_size + world_pos;
        let direction = goal_world_pos - transform.translation.xy();
        let dist_squared = direction.length_squared();
        if dist_squared <= MAX_GOAL_TILE_DIST * tile_size.x * MAX_GOAL_TILE_DIST * tile_size.x {
            agent_pos.0 = next_pos.0;
            commands.entity(entity).remove::<NextPos>();
            nav_controller.state = NavState::None;
            animation_controller.state = AnimationState::Idle;
            // DEBUG: Remove this debug info.
            // info!(
            //     "NavState::ApplyPath: Reached goal - entity: {}, distance: {}, direction: {}, pos: {}, goal_world_pos: {}",
            //     entity,
            //     dist_squared,
            //     direction,
            //     transform.translation.xy(),
            //     goal_world_pos
            // );
            continue;
        }

        let next_world_pos = next_pos.0.xy().as_vec2() * tile_size + world_pos;
        let direction = next_world_pos - transform.translation.xy();
        let dist_squared = direction.length_squared();

        // Set default direction to normalized vector of `direction / distance`
        let movement_dist = movement_speed.0 * time.delta_secs();
        movement.direction = movement_dist * direction.normalize_or_zero();

        if dist_squared < movement_dist * movement_dist {
            // Would overshoot, therefore apply direction and set/remove next_pos
            character_controller.translation = Some(direction);
            agent_pos.0 = next_pos.0;
            commands.entity(entity).remove::<NextPos>();
            // DEBUG: Remove this debug info.
            // info!(
            //     "NavState::ApplyPath: Reached next_pos - entity: {}, distance: {}, direction: {}, pos: {}, goal_world_pos: {}",
            //     entity,
            //     dist_squared,
            //     direction,
            //     transform.translation.xy(),
            //     next_world_pos
            // );
        } else {
            character_controller.translation = Some(movement.direction);
        }

        // Set animation state
        let state = animation_controller.state;
        if state != AnimationState::Jump || state != AnimationState::Fall {
            animation_controller.state = AnimationState::Walk;
        }
    }
}

/// Set all controller states to [`NavState::UpdatePos`]
///
/// ## Traits
///
/// - `T` must implement [`Character`].
fn refresh_pos<T>(mut character_query: Query<&mut NavController, With<T>>)
where
    T: Character,
{
    for mut controller in &mut character_query {
        if controller.state != NavState::UpdatePos {
            controller.state = NavState::UpdatePos;
        }
    }
}

/// Range for goal tracking interval
const TRACK_GOAL_INTERVAL_RANGE: Range<f32> = 0.5..1.0;

/// Track goal on [`GoalMoved`] for one [`Character`] at a time.
///
/// ## Traits
///
/// - `T` must implement [`Character`].
fn on_goal_moved<T>(
    _: On<GoalMoved>,
    mut rng: Single<&mut WyRand, With<NavRng>>,
    mut character_query: Query<(Entity, &mut NavController, Option<&TrackGoalTimer>), With<T>>,
    mut commands: Commands,
) where
    T: Character,
{
    // If all elements have `track_goal_done` set, reset.
    if character_query.iter().all(|(_, c, _)| c.track_goal_done) {
        character_query
            .iter_mut()
            .for_each(|(_, mut c, _)| c.track_goal_done = false);
    }

    // Find first entity matching states and without `track_goal_done`
    let Some((entity, mut controller, timer)) = character_query.iter_mut().find(|(_, c, _)| {
        (c.state == NavState::None || c.state == NavState::ApplyPath) && !c.track_goal_done
    }) else {
        return;
    };
    // Set `track_goal_done`
    controller.track_goal_done = true;

    // Insert/Remove random timer and return if it has not finished
    let Some(timer) = timer else {
        commands
            .entity(entity)
            .insert(TrackGoalTimer(Timer::from_seconds(
                rng.random_range(TRACK_GOAL_INTERVAL_RANGE),
                TimerMode::Once,
            )));
        return;
    };
    if !timer.0.is_finished() {
        return;
    }
    commands.entity(entity).remove::<TrackGoalTimer>();

    // Set controller state according to current state
    match controller.state {
        NavState::None => controller.state = NavState::UpdatePos,
        NavState::ApplyPath => controller.state = NavState::FindPath,
        _ => (),
    }
}

/// Tick [`TrackGoalTimer`]
fn tick_track_goal_timer(mut query: Query<&mut TrackGoalTimer>, time: Res<Time>) {
    for mut timer in &mut query {
        timer.0.tick(time.delta());
    }
}

/// Spawn [`NavRng`] by forking [`GlobalRng`]
fn setup_rng(mut global: Single<&mut WyRand, With<GlobalRng>>, mut commands: Commands) {
    commands.spawn((NavRng, global.fork_seed()));
}
