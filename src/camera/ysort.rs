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
    screens::Screen,
};

pub(super) fn plugin(app: &mut App) {
    // Sort entities with `YSort`
    app.add_systems(
        PostUpdate,
        (y_sort::<Player>, y_sort::<Slime>)
            .before(TransformSystems::Propagate)
            .run_if(in_state(Screen::Gameplay)),
    );
}

/// Factor for [`y_sort`]
///
/// This is required to ensure that we stay within the default Z-levels supported by bevy's camera.
pub(crate) const Y_SORT_FACTOR: f32 = 1e-5;

/// Applies to anything that is y-sorted
pub(crate) trait YSorted
where
    Self: Component + Default + Reflectable,
{
}

/// Sorts entities by their y position.
#[derive(Component, Default, Reflect)]
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

/// Applies the y-sorting to the entities Z position.
///
/// Heavily inspired by: <https://github.com/fishfolk/punchy>
///
/// ## Traits
///
/// - `T` must implement [`YSorted`].
fn y_sort<T>(
    query: Query<(&mut Transform, &YSort, Option<&YSortOffset>), (Changed<Transform>, With<T>)>,
    texture: Res<YSortCache<T>>,
) where
    T: YSorted,
{
    for (mut transform, sort, sort_offset) in query {
        transform.translation.z = (sort.0
            + sort_offset.map_or(0., |offset| offset.0) * Y_SORT_FACTOR)
            - (transform.translation.y - texture.texture_size.y as f32 * 0.5) * Y_SORT_FACTOR;
    }
}
