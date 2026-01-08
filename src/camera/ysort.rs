/*
 * File: ysort.rs
 * Author: Leopold Johannes Meinel (leo@meinel.dev)
 * -----
 * Copyright (c) 2026 Leopold Johannes Meinel & contributors
 * SPDX ID: Apache-2.0
 * URL: https://www.apache.org/licenses/LICENSE-2.0
 */

use std::marker::PhantomData;

use bevy::{prelude::*, reflect::Reflectable};

use crate::{
    characters::{npc::Slime, player::Player},
    levels::overworld::OverworldProcGen,
    procgen::{ProcGenController, ProcGenInit, ProcGenerated, TileDataRelatedCache},
    screens::Screen,
};

pub(super) fn plugin(app: &mut App) {
    // Sort entities with `YSort`
    app.add_systems(
        PostUpdate,
        (
            y_sort::<Player, OverworldProcGen>,
            y_sort::<Slime, OverworldProcGen>,
        )
            .before(TransformSystems::Propagate)
            .run_if(in_state(ProcGenInit(true)).and(in_state(Screen::Gameplay))),
    );
}

/// Applies to anything that is y-sorted
pub(crate) trait YSorted
where
    Self: Component + Default + Reflectable,
{
}

/// Sorts entities by their y position.
#[derive(Component, Default, Reflect, Debug)]
#[reflect(Component)]
pub(crate) struct YSort(pub(crate) f32);

/// Applies an offset to the [`YSort`].
///
/// The offset is expected to be in px.
#[derive(Component, Default, Reflect)]
#[reflect(Component)]
pub(crate) struct YSortOffset(pub(crate) f32);

/// Cache for [`YSort`]
///
/// ## Traits
///
/// - `T` must implement [`YSorted`].
#[derive(Resource, Default)]
pub(crate) struct YSortCache<T>
where
    T: YSorted,
{
    pub(crate) texture_size: UVec2,
    pub(crate) _phantom: PhantomData<T>,
}

// FIXME: We currently can't use Changed<Transform> because we always need to update z-level based on relative position.
/// Applies the y-sorting to the entities Z position.
///
/// Heavily inspired by: <https://github.com/fishfolk/punchy>
///
/// ## Traits
///
/// - `T` must implement [`YSorted`].
/// - `A` must implement [`ProcGenerated`]' and is used as the procedurally generated level.
fn y_sort<T, A>(
    query: Query<(&mut Transform, &YSort, Option<&YSortOffset>), With<T>>,
    controller: Res<ProcGenController<A>>,
    texture: Res<YSortCache<T>>,
    tile_data_related: Res<TileDataRelatedCache<A>>,
) where
    T: YSorted,
    A: ProcGenerated,
{
    let min_world_y = controller.min_chunk_pos().y as f32 * tile_data_related.chunk_size_px.y;

    for (mut transform, sort, sort_offset) in query {
        let sort_offset = sort_offset.map_or(0., |offset| offset.0);
        let relative_y = transform.translation.y - min_world_y;
        let texture_offset = texture.texture_size.y as f32 / 2.;

        // NOTE: We could also just divide by `world_height`, but multiplying `world_height` by 2 ensures that we never
        //       add more than 1 to `sort.0`
        transform.translation.z = sort.0
            + (sort_offset - (relative_y - texture_offset)) / (tile_data_related.world_height * 2.);
    }
}
