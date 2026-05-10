use std::marker::PhantomData;

use bevy::{color::palettes::tailwind, prelude::*};
use bevy_fast_mist::prelude::*;

use crate::{procgen::prelude::*, render::prelude::*};

/// Wrapper for mist.
pub(crate) trait MistWrapper
where
    Self: Component + Default + Clone,
{
    type Inner: Bundle;
    fn into_inner(self) -> Self::Inner;
    fn new(mesh: Handle<Mesh>) -> Self;
    fn spawn(&self, commands: &mut Commands, pos: Vec2) -> Entity {
        commands
            .spawn((
                self.clone(),
                self.clone().into_inner(),
                Transform::from_translation(pos.extend(OVERLAY_Z)),
            )) //
            .id()
    }
}

/// [`Handle<Mesh>`] for mist `T`.
#[derive(Resource, Default)]
pub(crate) struct MistMeshHandle<T>
where
    T: MistWrapper,
{
    pub(crate) handle: Handle<Mesh>,
    _phantom: PhantomData<T>,
}
impl<T> MistMeshHandle<T>
where
    T: MistWrapper,
{
    pub(crate) fn new(handle: Handle<Mesh>) -> Self {
        Self {
            handle,
            ..default()
        }
    }
}

/// Standard mist.
#[derive(Component, Reflect, Clone, Default)]
pub(crate) struct StandardMist((MeshMist, Mesh2d));
impl StandardMist {
    pub(crate) fn primitive() -> Rectangle {
        Rectangle::new(256., 256.)
    }
}
impl MistWrapper for StandardMist {
    type Inner = (MeshMist, Mesh2d);
    fn new(mesh: Handle<Mesh>) -> Self {
        Self((
            MeshMist {
                color: tailwind::CYAN_50.into(),
                ..default()
            },
            Mesh2d(mesh),
        ))
    }
    fn into_inner(self) -> Self::Inner {
        self.0
    }
}
impl ProcGenerated for StandardMist {}
impl Visible for StandardMist {}
