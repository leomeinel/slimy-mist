use std::marker::PhantomData;

use bevy::prelude::*;

use crate::render::prelude::*;

/// [`MeshLight`](bevy_fast_light::prelude::MeshLight) shadow for all type `T`.
///
/// The size of the [`Mesh`] is meant to be derived from [`CollisionDataCache`](crate::physics::prelude::CollisionDataCache).
#[derive(Resource, Default)]
pub(crate) struct MeshLightShadow<T>
where
    T: Visible,
{
    pub(crate) mesh: Handle<Mesh>,
    pub(crate) y_offset: f32,
    pub(crate) _phantom: PhantomData<T>,
}
