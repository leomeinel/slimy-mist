use bevy::prelude::*;
use bevy_prng::WyRand;
use rand::seq::IndexedRandom as _;

use crate::{images::prelude::*, levels::prelude::*, procgen::prelude::*, render::prelude::*};

/// Number of lights to spawn per chunk.
const LIGHTS_PER_CHUNK: usize = 4;

/// Spawn lights in a chunk.
pub(super) fn spawn_on_procgen_lights<T, A, B>(
    event: On<ProcGen<T>>,
    level: Single<Entity, With<B>>,
    mut procgen_rng: Single<&mut WyRand, With<ProcGenRng>>,
    mut commands: Commands,
    mut object_cache: ResMut<ProcGenCache<T>>,
    mesh: Res<LightMeshHandle<T>>,
    tile_data: Res<TileDataCache<A>>,
) where
    T: LightWrapper + ProcGenerated + Visible,
    A: ProcGenerated,
    B: Level,
{
    let world_pos = event.chunk_pos.as_vec2() * CHUNK_SIZE.as_vec2() * tile_data.tile_size;

    // Choose a number of target chunk tile origins to determine spawn positions
    let target_origins: Vec<(u32, u32)> = (0..CHUNK_SIZE.x)
        .flat_map(|x| (0..CHUNK_SIZE.y).map(move |y| (x, y)))
        .collect();
    let target_origins: Vec<Vec2> = target_origins
        .sample(&mut procgen_rng, LIGHTS_PER_CHUNK)
        .map(|&(x, y)| Vec2::new(x as f32, y as f32))
        .collect();

    for origin in target_origins {
        // Spawn entity in chosen tile and store in `object_cache`
        let target_pos = world_pos + origin * tile_data.tile_size;
        let entity = T::new(mesh.handle.clone()).spawn(&mut commands, target_pos);
        object_cache.chunk_positions.insert(entity, event.chunk_pos);

        // Add entity to level so that level handles despawning
        commands.entity(*level).add_child(entity);
    }
}
