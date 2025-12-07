/*
 * File: loading.rs
 * Author: Leopold Johannes Meinel (leo@meinel.dev)
 * -----
 * Copyright (c) 2025 Leopold Johannes Meinel & contributors
 * SPDX ID: Apache-2.0
 * URL: https://www.apache.org/licenses/LICENSE-2.0
 * -----
 * Heavily inspired by: https://github.com/TheBevyFlock/bevy_new_2d
 */

//! A loading screen during which game assets are loaded if necessary.
//! This reduces stuttering, especially for audio on Wasm.

// FIXME: This is currently unsupported. After it is, we should implement this: https://github.com/NiklasEi/bevy_asset_loader/blob/main/bevy_asset_loader/examples/progress_tracking.rs
// See: https://github.com/NiklasEi/bevy_asset_loader/pull/259
/*
use bevy::prelude::*;

use crate::{screens::Screen, theme::prelude::*};

pub(super) fn plugin(app: &mut App) {
    app.add_systems(OnEnter(Screen::Loading), spawn_loading_screen);

    app.add_systems(
        Update,
        enter_gameplay_screen.run_if(in_state(Screen::Loading).and(all_assets_loaded)),
    );
}

fn spawn_loading_screen(mut commands: Commands) {
    commands.spawn((
        widgets::common::ui_root("Loading Screen"),
        DespawnOnExit(Screen::Loading),
        children![widgets::common::label("Loading...")],
    ));
}

fn enter_gameplay_screen(mut next_screen: ResMut<NextState<Screen>>) {
    next_screen.set(Screen::Gameplay);
}



fn all_assets_loaded(resource_handles: Res<ResourceHandles>) -> bool {
    resource_handles.is_all_done()
}
*/
