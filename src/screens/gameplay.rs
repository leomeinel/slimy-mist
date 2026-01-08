/*
 * File: gameplay.rs
 * Author: Leopold Johannes Meinel (leo@meinel.dev)
 * -----
 * Copyright (c) 2026 Leopold Johannes Meinel & contributors
 * SPDX ID: Apache-2.0
 * URL: https://www.apache.org/licenses/LICENSE-2.0
 * -----
 * Heavily inspired by: https://github.com/TheBevyFlock/bevy_new_2d
 */

//! The screen state for the main gameplay.

use bevy::{input::common_conditions::input_just_pressed, prelude::*};

use crate::{
    Pause,
    camera::center_camera_on_player,
    characters::{
        Character, CollisionData, CollisionDataCache, CollisionHandle, VisualMap,
        animations::{AnimationData, AnimationDataCache, AnimationHandle, setup_animations},
        nav::NavTargetPosMap,
        npc::{Slime, SlimeAssets},
        player::{Player, PlayerAssets},
    },
    levels::overworld::{Overworld, OverworldProcGen, spawn_overworld},
    logging::error::{
        ERR_LOADING_ANIMATION_DATA, ERR_LOADING_COLLISION_DATA, ERR_LOADING_TILE_DATA,
    },
    menus::Menu,
    procgen::{
        CHUNK_SIZE, PROCGEN_DISTANCE, ProcGenController, ProcGenerated, TileData, TileDataCache,
        TileDataRelatedCache, TileHandle, navmesh::spawn_navmesh,
    },
    screens::Screen,
};

pub(super) fn plugin(app: &mut App) {
    // Insert/Remove resources and cache deserialized data in resources
    app.add_systems(
        OnEnter(Screen::Gameplay),
        (
            insert_resources,
            cache_animation_data::<Player>,
            cache_animation_data::<Slime>,
            cache_collision_data::<Player>,
            cache_collision_data::<Slime>,
            cache_tile_data_and_related::<OverworldProcGen>,
        )
            .in_set(PrepareGameplaySystems),
    );
    app.add_systems(OnExit(Screen::Gameplay), remove_resources);

    // Exit pause menu that was used to exit and unpause game
    app.add_systems(OnExit(Screen::Gameplay), (close_menu, unpause));
    // Unpause if in no menu and in gameplay screen
    app.add_systems(
        OnEnter(Menu::None),
        unpause.run_if(in_state(Screen::Gameplay)),
    );

    // Spawn overworld with navmesh and run required systems
    app.add_systems(
        OnEnter(Screen::Gameplay),
        (
            (
                setup_animations::<Player, PlayerAssets>,
                setup_animations::<Slime, SlimeAssets>,
            ),
            spawn_overworld,
            center_camera_on_player,
            spawn_navmesh::<OverworldProcGen, Overworld>,
        )
            .after(PrepareGameplaySystems)
            .chain(),
    );

    // Open pause on pressing P or Escape and pause game
    app.add_systems(
        Update,
        (
            (pause, spawn_pause_overlay, open_pause_menu)
                .run_if(in_state(Menu::None).and(
                    input_just_pressed(KeyCode::KeyP).or(input_just_pressed(KeyCode::Escape)),
                )),
            close_menu.run_if(not(in_state(Menu::None)).and(input_just_pressed(KeyCode::KeyP))),
        )
            .run_if(in_state(Screen::Gameplay)),
    );
}

/// A system set for systems that inserts [`Resource`]s dynamically for [`Screen::Gameplay`]
#[derive(SystemSet, Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub(crate) struct PrepareGameplaySystems;

/// rgba(0, 0, 0, 204)
const BACKGROUND_COLOR: Color = Color::srgba(0.0, 0.0, 0.0, 0.8);

/// Spawn pause overlay
fn spawn_pause_overlay(mut commands: Commands) {
    commands.spawn((
        Name::new("Pause Overlay"),
        Node {
            width: percent(100),
            height: percent(100),
            ..default()
        },
        GlobalZIndex(1),
        BackgroundColor(BACKGROUND_COLOR),
        DespawnOnExit(Pause(true)),
    ));
}

/// Open pause menu
fn open_pause_menu(mut next_state: ResMut<NextState<Menu>>) {
    next_state.set(Menu::Pause);
}

/// Close pause menu
fn close_menu(mut next_state: ResMut<NextState<Menu>>) {
    next_state.set(Menu::None);
}

/// Unpause the game
fn unpause(mut next_state: ResMut<NextState<Pause>>) {
    next_state.set(Pause(false));
}

/// Pause the game
fn pause(mut next_state: ResMut<NextState<Pause>>) {
    next_state.set(Pause(true));
}

/// Insert resources for [`crate::procgen`]
fn insert_resources(mut commands: Commands) {
    commands.init_resource::<NavTargetPosMap>();
    commands.init_resource::<ProcGenController<OverworldProcGen>>();
    commands.init_resource::<ProcGenController<Slime>>();
    commands.init_resource::<VisualMap>();
}

/// Remove resources for [`crate::procgen`]
fn remove_resources(mut commands: Commands) {
    commands.remove_resource::<NavTargetPosMap>();
    commands.remove_resource::<ProcGenController<OverworldProcGen>>();
    commands.remove_resource::<ProcGenController<Slime>>();
    commands.remove_resource::<VisualMap>();
}

/// Cache data from [`TileData`] in [`TileDataCache`] and related data in [`TileDataRelatedCache`]
fn cache_tile_data_and_related<T>(
    mut commands: Commands,
    mut data: ResMut<Assets<TileData<T>>>,
    handle: Res<TileHandle<T>>,
) where
    T: ProcGenerated,
{
    let data = data.remove(handle.0.id()).expect(ERR_LOADING_TILE_DATA);
    // FIXME: Add missing fields from `TileData`
    let tile_size = data.tile_size;
    commands.insert_resource(TileDataCache::<T> {
        tile_size,
        ..default()
    });
    let chunk_size_px = CHUNK_SIZE.as_vec2() * tile_size;
    let world_height = PROCGEN_DISTANCE as f32 * 2. + 1. * chunk_size_px.y;
    commands.insert_resource(TileDataRelatedCache::<T> {
        chunk_size_px,
        world_height,
        ..default()
    });

    commands.remove_resource::<TileHandle<T>>();
}

/// Cache data from [`CollisionData`] in [`CollisionDataCache`]
fn cache_collision_data<T>(
    mut commands: Commands,
    mut data: ResMut<Assets<CollisionData<T>>>,
    handle: Res<CollisionHandle<T>>,
) where
    T: Character,
{
    let data = data
        .remove(handle.0.id())
        .expect(ERR_LOADING_COLLISION_DATA);
    commands.insert_resource(CollisionDataCache::<T> {
        shape: data.shape,
        width: data.width,
        height: data.height,
        ..default()
    });

    commands.remove_resource::<CollisionHandle<T>>();
}

/// Cache data from [`AnimationData`] in [`AnimationDataCache`]
fn cache_animation_data<T>(
    mut commands: Commands,
    mut data: ResMut<Assets<AnimationData<T>>>,
    handle: Res<AnimationHandle<T>>,
) where
    T: Character,
{
    let data = data
        .remove(handle.0.id())
        .expect(ERR_LOADING_ANIMATION_DATA);
    commands.insert_resource(AnimationDataCache::<T> {
        atlas_columns: data.atlas_columns,
        atlas_rows: data.atlas_rows,
        idle_row: data.idle_row,
        idle_frames: data.idle_frames,
        idle_interval_ms: data.idle_interval_ms,
        walk_row: data.walk_row,
        walk_frames: data.walk_frames,
        walk_interval_ms: data.walk_interval_ms,
        walk_sound_frames: data.walk_sound_frames,
        jump_row: data.jump_row,
        jump_frames: data.jump_frames,
        jump_sound_frames: data.jump_sound_frames,
        fall_row: data.fall_row,
        fall_frames: data.fall_frames,
        fall_sound_frames: data.fall_sound_frames,
        ..default()
    });

    commands.remove_resource::<AnimationHandle<T>>();
}
