/*
 * Heavily inspired by:
 * - https://github.com/TheBevyFlock/bevy_new_2d
 */

//! The screen state for the main gameplay.

use bevy::prelude::*;

use crate::{
    characters::prelude::*, images::prelude::*, input::prelude::*, levels::prelude::*,
    procgen::prelude::*, render::prelude::*, screens::prelude::*, ui::prelude::*,
};

pub(super) struct GameplayPlugin;
impl Plugin for GameplayPlugin {
    fn build(&self, app: &mut App) {
        app.configure_sets(
            OnEnter(Screen::Gameplay),
            (
                EnterGameplaySystems::Resources,
                EnterGameplaySystems::Images,
                EnterGameplaySystems::Animations,
                EnterGameplaySystems::Levels,
                EnterGameplaySystems::Camera,
                EnterGameplaySystems::NavMesh,
            )
                .chain(),
        );

        app.add_systems(
            OnEnter(Screen::Gameplay),
            insert_resources.in_set(EnterGameplaySystems::Resources),
        );
        app.add_systems(OnExit(Screen::Gameplay), remove_resources);
    }
}

/// A [`SystemSet`] for systems that initialize [`Screen::Gameplay`]
#[derive(SystemSet, Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub(crate) enum EnterGameplaySystems {
    Resources,
    Images,
    Animations,
    Levels,
    Camera,
    NavMesh,
}

/// Insert [`Resource`]s
fn insert_resources(mut commands: Commands) {
    commands.init_resource::<DayTimer>();
    commands.init_resource::<DayUpdateTimer>();
    commands.init_resource::<JoystickMap>();
    commands.init_resource::<MouseDrag>();
    commands.init_resource::<PointerBlockedByUi>();
    commands.init_resource::<PointerStartTimeSecs>();
    commands.init_resource::<ProcGenCache<OverworldProcGen>>();
    commands.init_resource::<ProcGenCache<Slime>>();
    commands.init_resource::<ProcGenCache<StreetLight>>();
    commands.init_resource::<ProcGenCache<StandardMist>>();
    commands.init_resource::<WorldUiHealthBarMap>();
}

/// Remove [`Resource`]s
fn remove_resources(mut commands: Commands) {
    commands.remove_resource::<DayTimer>();
    commands.remove_resource::<DayUpdateTimer>();
    commands.remove_resource::<DisplayLayers<Player>>();
    commands.remove_resource::<DisplayLayers<Slime>>();
    commands.remove_resource::<JoystickMap>();
    commands.remove_resource::<MouseDrag>();
    commands.remove_resource::<PointerBlockedByUi>();
    commands.remove_resource::<PointerStartTimeSecs>();
    commands.remove_resource::<ProcGenCache<OverworldProcGen>>();
    commands.remove_resource::<ProcGenCache<Slime>>();
    commands.remove_resource::<ProcGenCache<StreetLight>>();
    commands.remove_resource::<ProcGenCache<StandardMist>>();
    commands.remove_resource::<WorldUiHealthBarMap>();
}
