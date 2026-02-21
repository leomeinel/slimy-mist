/*
 * File: interaction.rs
 * Author: Leopold Johannes Meinel (leo@meinel.dev)
 * -----
 * Copyright (c) 2026 Leopold Johannes Meinel & contributors
 * SPDX ID: Apache-2.0
 * URL: https://www.apache.org/licenses/LICENSE-2.0
 * -----
 * Heavily inspired by: https://github.com/TheBevyFlock/bevy_new_2d
 */

use bevy::{
    prelude::*,
    window::{CursorIcon, PrimaryWindow, SystemCursorIcon},
};
use bevy_asset_loader::prelude::*;

use crate::{audio::sound_effect, ui::prelude::*};

pub(super) fn plugin(app: &mut App) {
    // Insert states
    app.init_state::<OverrideInteraction>();

    // Visualize ui interactions
    app.add_systems(OnEnter(OverrideInteraction(false)), reset_palette);
    app.add_systems(
        Update,
        (
            apply_palette,
            visualize_button_hover,
            visualize_button_pressed,
        ),
    );

    // Reset `CursorIcon`
    app.add_observer(on_remove_button);

    // Play sound effects
    app.add_observer(play_on_hover_sound_effect);
    app.add_observer(play_on_click_sound_effect);
}

/// Tracks whether [`Interaction::None`] is allowed to be overriden by [`InteractionOverride`].
#[derive(States, Copy, Clone, Eq, PartialEq, Hash, Debug, Default)]
pub(crate) struct OverrideInteraction(pub(crate) bool);

/// A custom [`Interaction`] that overrides if [`OverrideInteraction`] is true.
#[derive(Component, Default, PartialEq)]
pub(crate) enum InteractionOverride {
    /// Nothing has happened
    #[default]
    None,
    /// The node has been hovered over
    Hovered,
}

/// Palette for widget interactions. Add this to an entity that supports
/// [`Interaction`]s, such as a button, to change its [`BackgroundColor`] based
/// on the current interaction state.
#[derive(Component, Debug, Reflect)]
#[reflect(Component)]
pub(crate) struct InteractionPalette {
    pub(crate) none: Color,
    pub(crate) hovered: Color,
    pub(crate) pressed: Color,
}

/// Assets for interaction
#[derive(AssetCollection, Resource)]
pub(crate) struct InteractionAssets {
    #[asset(path = "audio/sound-effects/ui/hover.ogg")]
    hover: Handle<AudioSource>,
    #[asset(path = "audio/sound-effects/ui/click.ogg")]
    click: Handle<AudioSource>,
}

/// Reset [`BackgroundColor`] from palette mapped to [`Interaction`].
///
/// This sets the appropriate [`BackgroundColor`] for all [`Interaction::None`].
///
/// This allows [`Interaction`] to override [`OverrideInteraction`] in certain scenarios.
pub(crate) fn reset_palette(
    query: Query<(&Interaction, &InteractionPalette, &mut BackgroundColor)>,
) {
    for (interaction, palette, mut background) in query {
        if *interaction == Interaction::None {
            *background = palette.none.into();
        }
    }
}

/// Apply [`BackgroundColor`] from palette mapped to [`Interaction`] or [`InteractionOverride`].
pub(crate) fn apply_palette(
    query: Query<
        (
            &Interaction,
            &InteractionOverride,
            &InteractionPalette,
            &mut BackgroundColor,
        ),
        Or<(Changed<Interaction>, Changed<InteractionOverride>)>,
    >,
) {
    for (interaction, interaction_override, palette, mut background) in query {
        *background = match interaction {
            Interaction::None => match interaction_override {
                InteractionOverride::Hovered => palette.hovered,
                InteractionOverride::None => palette.none,
            },
            Interaction::Hovered => palette.hovered,
            Interaction::Pressed => palette.pressed,
        }
        .into();
    }
}

/// Set [`CursorIcon`] according to [`Interaction`].
pub(crate) fn visualize_button_hover(
    window: Single<(Entity, Option<&CursorIcon>), With<PrimaryWindow>>,
    query: Query<&Interaction, (Changed<Interaction>, With<Button>)>,
    mut commands: Commands,
) {
    if query.is_empty() {
        return;
    }

    let target_icon = if query.iter().any(|i| *i == Interaction::Hovered) {
        CursorIcon::System(SystemCursorIcon::Pointer)
    } else {
        CursorIcon::default()
    };
    let (entity, icon) = window.into_inner();

    if Some(&target_icon) != icon {
        commands.entity(entity).insert(target_icon);
    }
}

/// Move [`Node`] based on [`NodeOffset`] according to [`Interaction`].
pub(crate) fn visualize_button_pressed(
    query: Query<(&Interaction, &NodeOffset, &mut Node), (Changed<Interaction>, With<Button>)>,
) {
    for (interaction, offset, mut node) in query {
        if *interaction == Interaction::Pressed {
            node.bottom = px(0);
        } else {
            node.bottom = px(offset.0.y);
        }
    }
}

/// Reset [`CursorIcon`].
fn on_remove_button(
    _: On<Remove, Button>,
    window: Single<(Entity, Option<&CursorIcon>), With<PrimaryWindow>>,
    mut commands: Commands,
) {
    let (entity, icon) = window.into_inner();
    let target_icon = CursorIcon::default();
    if Some(&target_icon) != icon {
        commands.entity(entity).insert(target_icon);
    }
}

/// Play sound effect on hover
fn play_on_hover_sound_effect(
    event: On<Pointer<Over>>,
    query: Query<(), Or<(With<Interaction>, With<InteractionOverride>)>>,
    mut commands: Commands,
    interaction_assets: If<Res<InteractionAssets>>,
) {
    if query.contains(event.entity) {
        commands.spawn(sound_effect(interaction_assets.hover.clone()));
    }
}

/// Play sound effect on click
fn play_on_click_sound_effect(
    event: On<Pointer<Click>>,
    query: Query<(), Or<(With<Interaction>, With<InteractionOverride>)>>,
    mut commands: Commands,
    interaction_assets: If<Res<InteractionAssets>>,
) {
    if query.contains(event.entity) {
        commands.spawn(sound_effect(interaction_assets.click.clone()));
    }
}
