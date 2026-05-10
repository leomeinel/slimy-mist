// FIXME: Find a way to avoid rounding errors by f32 causing problems with the infinite world.
//        Currently `1 << 20` is the limit where there is almost no visible rounding errors.
//        This was determined after a visual inspection at that position, no other testing has been done.
//        Above that, movement becomes more and more imprecise and if you go further, tiles will
//        scale incorrectly.
//        As far as I know, until I switch to f64 this will not be fixable.
//        A workaround is preventing movement beyond that point and adding a visible border or not moving
//        the player on the world, but scrolling the world below the player.
//        The second solution would require a lot of changes and I can imagine at least a few drawbacks of
//        that. Especially pathfinding, storing game data etc. would be a lot harder/impossible.
//        I'm pretty sure that even for my plans with the infinite open world, a limit of `1 << 20`
//        will be just fine. That is a world of 4096x4096 chunks with 16 16x16 tiles.
//        Also see: https://github.com/bevyengine/bevy/issues/1680

mod characters;
mod chunks;
mod common;
mod lights;
mod mist;

pub(crate) mod prelude {
    pub(crate) use super::{
        CHUNK_SIZE, DespawnProcGen, PROCGEN_DISTANCE, ProcGen, ProcGenCache, ProcGenInit,
        ProcGenRng, ProcGenState, ProcGenerated,
    };
}

use std::marker::PhantomData;

use bevy::{
    platform::collections::{HashMap, HashSet},
    prelude::*,
    reflect::Reflectable,
};

use crate::{
    characters::prelude::*, core::prelude::*, levels::prelude::*, log::prelude::*,
    render::prelude::*, screens::prelude::*, utils::prelude::*,
};

pub(super) struct ProcGenPlugin;
impl Plugin for ProcGenPlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<ProcGenState>();
        app.init_state::<ProcGenInit>();
        app.init_state::<DespawnProcGen>();

        app.add_systems(
            OnEnter(ProcGenState::Spawn),
            (
                chunks::spawn_chunks::<OverworldProcGen, Overworld>,
                (
                    common::spawn_objects::<Slime, OverworldProcGen, Overworld>,
                    common::spawn_objects::<StreetLight, OverworldProcGen, Overworld>,
                    common::spawn_objects::<StandardMist, OverworldProcGen, Overworld>,
                ),
            )
                .run_if(in_state(Screen::Gameplay))
                .chain(),
        );
        app.add_systems(
            OnExit(Screen::Gameplay),
            (
                reset_procgen_state,
                reset_procgen_init,
                reset_procgen_despawning,
            ),
        );
        app.add_systems(Startup, setup_rng::<ProcGenRng>);
        app.add_systems(
            Update,
            (
                common::collect_to_despawn::<OverworldProcGen, OverworldProcGen, true>,
                common::collect_to_despawn::<Slime, OverworldProcGen, false>,
                common::collect_to_despawn::<StreetLight, OverworldProcGen, false>,
                common::collect_to_despawn::<StandardMist, OverworldProcGen, false>,
            )
                .run_if(in_state(ProcGenState::Despawn).and(in_state(Screen::Gameplay)))
                .in_set(AppSystems::Update)
                .in_set(PausableSystems),
        );
        app.add_systems(
            PostUpdate,
            (
                (
                    common::set_despawning::<OverworldProcGen>,
                    common::set_despawning::<Slime>,
                    common::set_despawning::<StreetLight>,
                    common::set_despawning::<StandardMist>,
                ),
                (
                    common::despawn::<Slime>,
                    common::despawn::<StreetLight>,
                    common::despawn::<StandardMist>,
                    common::despawn::<OverworldProcGen>,
                )
                    .run_if(in_state(DespawnProcGen(true)))
                    .chain(),
            )
                .run_if(in_state(Screen::Gameplay)),
        );

        app.add_observer(
            chunks::spawn_on_procgen_chunks::<OverworldProcGen, OverworldAssets, Overworld>,
        );
        app.add_observer(
            characters::spawn_on_procgen_characters::<Slime, OverworldProcGen, Overworld>,
        );
        app.add_observer(
            lights::spawn_on_procgen_lights::<StreetLight, OverworldProcGen, Overworld>,
        );
        app.add_observer(mist::spawn_on_procgen_mist::<StandardMist, OverworldProcGen, Overworld>);
    }
}

/// Size of a single chunk
pub(crate) const CHUNK_SIZE: UVec2 = UVec2 { x: 16, y: 16 };

/// Maximum distance for procedural generation
pub(crate) const PROCGEN_DISTANCE: i32 = 2;

/// Tracks the current proc gen task
#[derive(States, Copy, Clone, Eq, PartialEq, Hash, Debug, Default)]
pub(crate) enum ProcGenState {
    #[default]
    Despawn,
    Spawn,
    MoveNavMesh,
}

/// Tracks whether we are currently despawning
#[derive(States, Copy, Clone, Eq, PartialEq, Hash, Debug, Default)]
pub(crate) struct DespawnProcGen(pub(crate) bool);

/// Tracks the proc gen has been initialized fully at least once
#[derive(States, Copy, Clone, Eq, PartialEq, Hash, Debug, Default)]
pub(crate) struct ProcGenInit(pub(crate) bool);

/// Applies to anything that is generated by procgen
pub(crate) trait ProcGenerated
where
    Self: Component + Default + Reflectable,
{
}

#[derive(EntityEvent)]
pub(crate) struct ProcGen<T>
where
    T: ProcGenerated,
{
    entity: Entity,
    chunk_pos: IVec2,
    _phantom: PhantomData<T>,
}

/// Cache that maps entities to their positions.
///
/// We are also storing entities to avoid duplicate spawning for a chunk. This allows us to check
/// if a chunk has already been spawned to, even for entities that have left the chunk.
#[derive(Default, Debug, Resource)]
pub(crate) struct ProcGenCache<T>
where
    T: ProcGenerated,
{
    pub(crate) camera_chunk_pos: IVec2,
    pub(crate) chunk_positions: HashMap<Entity, IVec2>,
    pub(crate) to_despawn: HashSet<Entity>,
    _phantom: PhantomData<T>,
}
impl<T> ProcGenCache<T>
where
    T: ProcGenerated,
{
    /// Minimum chunk position stored in [`ProcGenCache`]
    pub(crate) fn min_chunk_pos(&self) -> &IVec2 {
        self.chunk_positions
            .values()
            .min_by_key(|pos| (pos.x, pos.y))
            .expect(ERR_INVALID_CHUNK_POSITIONS)
    }
}

/// Rng for procedural generation
#[derive(Component, Default)]
pub(crate) struct ProcGenRng;
impl ForkedRng for ProcGenRng {}

/// Reset [`ProcGenState`]
fn reset_procgen_state(mut next_state: ResMut<NextState<ProcGenState>>) {
    (*next_state).set_if_neq(ProcGenState::default());
}

/// Reset [`ProcGenInit`]
fn reset_procgen_init(mut next_state: ResMut<NextState<ProcGenInit>>) {
    (*next_state).set_if_neq(ProcGenInit::default());
}

/// Reset [`DespawnProcGen`]
fn reset_procgen_despawning(mut next_state: ResMut<NextState<DespawnProcGen>>) {
    (*next_state).set_if_neq(DespawnProcGen::default());
}
