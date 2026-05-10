/*
 * Heavily inspired by:
 * - https://github.com/TheBevyFlock/bevy_new_2d/tree/main
 * - https://github.com/NiklasEi/bevy_asset_loader
 */

//! A loading screen during which game assets are loaded if necessary.
//! This reduces stuttering, especially for audio on Wasm.

use bevy::{platform::collections::HashMap, prelude::*};
use bevy_asset_loader::prelude::*;
use bevy_common_assets::ron::RonAssetPlugin;
use iyes_progress::ProgressPlugin;

use crate::{
    animations::prelude::*, characters::prelude::*, images::prelude::*, levels::prelude::*,
    log::prelude::*, physics::prelude::*, procgen::prelude::*, render::prelude::*,
    screens::prelude::*, ui::prelude::*,
};

pub(super) struct LoadingPlugin;
impl Plugin for LoadingPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            // NOTE: This advances to `Screen::LoadingCache` to cache data
            ProgressPlugin::<Screen>::new()
                .with_state_transition(Screen::Loading, Screen::LoadingCache),
            RonAssetPlugin::<AnimationData<Player>>::new(&["animation.ron"]),
            RonAssetPlugin::<AnimationData<Slime>>::new(&["animation.ron"]),
            RonAssetPlugin::<CollisionData<Player>>::new(&["collision.ron"]),
            RonAssetPlugin::<CollisionData<Slime>>::new(&["collision.ron"]),
            RonAssetPlugin::<CreditsData>::new(&["credits.ron"]),
            RonAssetPlugin::<LayerData<Player>>::new(&["layers.ron"]),
            RonAssetPlugin::<LayerData<Slime>>::new(&["layers.ron"]),
            RonAssetPlugin::<TileData<OverworldProcGen>>::new(&["tiles.ron"]),
        ));

        app.add_loading_state(
            LoadingState::new(Screen::Loading)
                .load_collection::<InteractionAssets>()
                .load_collection::<SplashAssets>()
                .load_collection::<CreditsAssets>()
                .with_dynamic_assets_file::<StandardDynamicAssetCollection>(
                    "data/levels/overworld.assets.ron",
                )
                .load_collection::<OverworldAssets>()
                .with_dynamic_assets_file::<StandardDynamicAssetCollection>(
                    "data/characters/player/male.assets.ron",
                )
                .load_collection::<PlayerAssets>()
                .with_dynamic_assets_file::<StandardDynamicAssetCollection>(
                    "data/characters/npc/slime.assets.ron",
                )
                .load_collection::<SlimeAssets>(),
        );

        app.add_systems(
            OnEnter(Screen::Loading),
            (
                // After initial `LoadingState<Screen::Loading>` insert additional resources.
                insert_material_handle_resources,
                insert_mesh_handle_resources,
                insert_handle_resources.after(LoadingStateSet(Screen::Loading)),
                spawn_loading_screen,
            )
                .chain(),
        );
        app.add_systems(
            OnEnter(Screen::LoadingCache),
            (
                (
                    cache_animation_data_and_related::<Player>,
                    cache_animation_data_and_related::<Slime>,
                    cache_collision_data_and_related::<Player>,
                    cache_collision_data_and_related::<Slime>,
                    cache_credits_data,
                    cache_layer_data::<Player>,
                    cache_layer_data::<Slime>,
                    cache_tile_data_and_related::<OverworldProcGen>,
                ),
                enter_splash_screen,
            )
                .chain(),
        );
    }
}

/// Display loading screen
fn spawn_loading_screen(mut commands: Commands, font: Res<UiFontHandle>) {
    commands.spawn((
        root_widget("Loading Screen"),
        DespawnOnExit(Screen::Loading),
        children![label_widget("Loading...", font.0.clone())],
    ));
}

/// Insert handle [`Resource`]s for [`Material`]s.
fn insert_material_handle_resources(
    mut commands: Commands,
    mut blood_particle_materials: ResMut<Assets<BloodParticleMaterial>>,
    mut death_particle_materials: ResMut<Assets<DeathParticleMaterial>>,
    mut dust_trail_particle_materials: ResMut<Assets<DustTrailParticleMaterial>>,
    mut melee_particle_materials: ResMut<Assets<MeleeParticleMaterial>>,
) {
    // `Particle2dMaterialHandle`
    commands.insert_resource(Particle2dMaterialHandle::<BloodParticleMaterial>(
        blood_particle_materials.add(BloodParticleMaterial::default()),
    ));
    commands.insert_resource(Particle2dMaterialHandle::<DeathParticleMaterial>(
        death_particle_materials.add(DeathParticleMaterial::default()),
    ));
    commands.insert_resource(Particle2dMaterialHandle::<DustTrailParticleMaterial>(
        dust_trail_particle_materials.add(DustTrailParticleMaterial::default()),
    ));
    commands.insert_resource(Particle2dMaterialHandle::<MeleeParticleMaterial>(
        melee_particle_materials.add(MeleeParticleMaterial::default()),
    ));
}

/// Insert handle [`Resource`]s for [`Mesh`]s.
fn insert_mesh_handle_resources(mut commands: Commands, mut meshes: ResMut<Assets<Mesh>>) {
    // `LightMeshHandle`
    commands.insert_resource(LightMeshHandle::<StreetLight>::new(
        meshes.add(StreetLight::primitive()),
    ));
    // `MistMeshHandle`
    commands.insert_resource(MistMeshHandle::<StandardMist>::new(
        meshes.add(StandardMist::primitive()),
    ));
}

/// Insert handle [`Resource`]s for deserialized data.
///
/// These serve as handles for the actual data.
fn insert_handle_resources(mut commands: Commands, asset_server: Res<AssetServer>) {
    // `AnimationData`
    commands.insert_resource(AnimationHandle::<Player>(
        asset_server.load("data/characters/human/male.animation.ron"),
    ));
    commands.insert_resource(AnimationHandle::<Slime>(
        asset_server.load("data/characters/npc/slime.animation.ron"),
    ));

    // `CollisionData`
    commands.insert_resource(CollisionHandle::<Player>(
        asset_server.load("data/characters/human/male.collision.ron"),
    ));
    commands.insert_resource(CollisionHandle::<Slime>(
        asset_server.load("data/characters/npc/slime.collision.ron"),
    ));

    commands.insert_resource(LayerHandle::<Player>(
        asset_server.load("data/characters/player/male.layers.ron"),
    ));
    commands.insert_resource(LayerHandle::<Slime>(
        asset_server.load("data/characters/npc/slime.layers.ron"),
    ));

    // `CreditsData`
    commands.insert_resource(CreditsHandle(asset_server.load("data/menus/credits.ron")));

    // `TileData`
    commands.insert_resource(TileHandle::<OverworldProcGen>(
        asset_server.load("data/levels/overworld.tiles.ron"),
    ));

    // `ParticleHandle` not needing a custom data struct
    commands.insert_resource(ParticleHandle::<BloodParticle> {
        handle: asset_server.load("data/particles/blood.particle.ron"),
        ..default()
    });
    commands.insert_resource(ParticleHandle::<DeathParticle> {
        handle: asset_server.load("data/particles/death.particle.ron"),
        ..default()
    });
    commands.insert_resource(ParticleHandle::<DustTrailParticle> {
        handle: asset_server.load("data/particles/dust-trail.particle.ron"),
        ..default()
    });
    commands.insert_resource(ParticleHandle::<MeleeParticle> {
        handle: asset_server.load("data/particles/melee.particle.ron"),
        ..default()
    });

    // `UiFontHandle` not needing a custom data struct
    commands.insert_resource(UiFontHandle(
        asset_server.load("fonts/Pixeloid/PixeloidSans.ttf"),
    ));
}

/// Cache data from [`AnimationData`] in [`AnimationDataCache`].
fn cache_animation_data_and_related<T>(
    mut commands: Commands,
    mut data: ResMut<Assets<AnimationData<T>>>,
    handle: Res<AnimationHandle<T>>,
) where
    T: Visible,
{
    let data = data
        .remove(handle.0.id())
        .expect(ERR_LOADING_ANIMATION_DATA);
    commands.insert_resource(AnimationDataCache::<T> {
        atlas_columns: data.atlas_columns,
        atlas_rows: data.atlas_rows,
        idle_clips: data.idle_clips.clone(),
        walk_clips: data.walk_clips.clone(),
        _run_clips: data.run_clips,
        jump_clips: data.jump_clips.clone(),
        ..default()
    });

    let mut audio_map = HashMap::new();
    for clip in data.idle_clips.as_ref() {
        audio_map.insert(clip.state, clip.audio_indexes.clone());
    }
    if let Some(walk_clips) = data.walk_clips.as_ref() {
        for clip in walk_clips {
            audio_map.insert(clip.state, clip.audio_indexes.clone());
        }
    }
    if let Some(jump_clips) = data.jump_clips.as_ref() {
        for clip in jump_clips {
            audio_map.insert(clip.state, clip.audio_indexes.clone());
        }
    }
    commands.insert_resource(AnimationAudioMap::<T> {
        map: audio_map,
        ..default()
    });

    // Remove handle after caching since it is no longer needed
    commands.remove_resource::<AnimationHandle<T>>();
}

/// Cache data from [`CollisionData`] in [`CollisionDataCache`].
fn cache_collision_data_and_related<T>(
    mut commands: Commands,
    mut data: ResMut<Assets<CollisionData<T>>>,
    mut meshes: ResMut<Assets<Mesh>>,
    handle: Res<CollisionHandle<T>>,
) where
    T: Visible,
{
    let data = data
        .remove(handle.0.id())
        .expect(ERR_LOADING_COLLISION_DATA);
    let (shape, width, height, y_offset) = (
        data.shape.clone().unwrap_or("ball".to_string()),
        data.width.unwrap_or(0.),
        data.height.unwrap_or(0.),
        data.y_offset.unwrap_or(0.),
    );
    if data.shape.is_none() || data.width.is_none() || data.height.is_none() {
        warn_once!("{}", WARN_INCOMPLETE_COLLISION_DATA);
    };
    commands.insert_resource(CollisionDataCache::<T> {
        shape,
        width,
        height,
        y_offset,
        ..default()
    });
    commands.insert_resource(MeshLightShadow::<T> {
        mesh: meshes.add(Ellipse::new(width / 2., height / 2.)),
        // NOTE: Move to the bottom but exclude the outline.
        y_offset: -height / 2. + 1.,
        ..default()
    });

    // Remove handle after caching since it is no longer needed
    commands.remove_resource::<CollisionHandle<T>>();
}

/// Cache data from [`CreditsData`] in [`CreditsDataCache`].
fn cache_credits_data(
    mut commands: Commands,
    mut data: ResMut<Assets<CreditsData>>,
    handle: Res<CreditsHandle>,
) {
    let data = data.remove(handle.0.id()).expect(ERR_LOADING_CREDITS_DATA);
    commands.insert_resource(CreditsDataCache {
        created_by: data.created_by,
        assets: data.assets,
        code: data.code,
    });

    // Remove handle after caching since it is no longer needed
    commands.remove_resource::<CreditsHandle>();
}

/// Cache data from [`LayerData`] in [`LayerDataCache`].
fn cache_layer_data<T>(
    mut commands: Commands,
    mut data: ResMut<Assets<LayerData<T>>>,
    asset_server: Res<AssetServer>,
    handle: Res<LayerHandle<T>>,
) where
    T: Visible,
{
    let data = data.remove(handle.0.id()).expect(ERR_LOADING_LAYER_DATA);
    let base = data.base.iter().map(|l| asset_server.load(l)).collect();
    let floating = data
        .floating
        .map(|layers| layers.iter().map(|l| asset_server.load(l)).collect());
    commands.insert_resource(LayerDataCache::<T> {
        floating,
        base,
        ..default()
    });

    // Remove handle after caching since it is no longer needed
    commands.remove_resource::<LayerHandle<T>>();
}

/// Cache data from [`TileData`] in [`TileDataCache`] and [`LevelDimensions`].
fn cache_tile_data_and_related<T>(
    mut commands: Commands,
    mut data: ResMut<Assets<TileData<T>>>,
    handle: Res<TileHandle<T>>,
) where
    T: ProcGenerated,
{
    let data = data.remove(handle.0.id()).expect(ERR_LOADING_TILE_DATA);
    commands.insert_resource(TileDataCache::<T> {
        tile_size: data.tile_size,
        _full_dirt: data.full_dirt,
        _full_grass: data.full_grass,
        _corner_outer_grass_to_dirt: data.corner_outer_grass_to_dirt,
        _corner_outer_dirt_to_grass: data.corner_outer_dirt_to_grass,
        _side_dirt_and_grass: data.side_dirt_and_grass,
        _diag_stripe_grass_in_dirt: data.diag_stripe_grass_in_dirt,
        ..default()
    });
    let chunk_size_px = CHUNK_SIZE.as_vec2() * data.tile_size;
    let world_height = PROCGEN_DISTANCE as f32 * 2. + 1. * chunk_size_px.y;
    commands.insert_resource(LevelDimensions::<T> {
        chunk_size_px,
        world_height,
        ..default()
    });

    // Remove handle after caching since it is no longer needed
    commands.remove_resource::<TileHandle<T>>();
}
