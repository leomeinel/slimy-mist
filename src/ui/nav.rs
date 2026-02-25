/*
 * File: nav.rs
 * Author: Leopold Johannes Meinel (leo@meinel.dev)
 * -----
 * Copyright (c) 2026 Leopold Johannes Meinel & contributors
 * SPDX ID: Apache-2.0
 * URL: https://www.apache.org/licenses/LICENSE-2.0
 * -----
 * Heavily inspired by: https://github.com/bevyengine/bevy/blob/latest/examples/ui/auto_directional_navigation.rs
 */

// FIXME: Suppress warning if InputScroll is present

use core::time::Duration;

use bevy::{
    camera::NormalizedRenderTarget,
    input_focus::{
        InputDispatchPlugin, InputFocus, InputFocusVisible,
        directional_navigation::{AutoNavigationConfig, DirectionalNavigationPlugin},
    },
    math::CompassOctant,
    picking::{
        backend::HitData,
        pointer::{Location, PointerId},
    },
    prelude::*,
    ui::auto_directional_navigation::{AutoDirectionalNavigation, AutoDirectionalNavigator},
};

use crate::{
    input::ui::{UiNavAction, UiNavActionSet},
    logging::warn::*,
    ui::interaction::{InteractionOverride, OverrideInteraction},
    utils::run_conditions::component_is_present,
};

pub(super) fn plugin(app: &mut App) {
    // Library plugins
    app.add_plugins((InputDispatchPlugin, DirectionalNavigationPlugin));

    // Insert resources
    app.insert_resource(InputFocusVisible(true));
    // FIXME: This currently sometimes navigates in weird ways. This is especially visible in the `Settings`
    //        `Menu`. The current `min_alignment_factor` is usable, but still not great.
    app.insert_resource(AutoNavigationConfig {
        min_alignment_factor: 0.01,
        prefer_aligned: true,
        ..default()
    });

    // Process inputs, override `Interaction` and navigate
    app.add_systems(OnEnter(OverrideInteraction(true)), set_input_focus);
    app.add_systems(
        OnEnter(OverrideInteraction(false)),
        reset_interaction_overrides,
    );
    app.add_systems(
        PreUpdate,
        (
            override_interaction.run_if(component_is_present::<AutoDirectionalNavigation>),
            (
                reset_override,
                override_interaction_on_release,
                override_interaction_on_focus,
                navigate,
            )
                .run_if(in_state(OverrideInteraction(true)))
                .chain(),
        )
            .chain(),
    );

    app.add_systems(
        Update,
        (hover_focused, click_focused).run_if(in_state(OverrideInteraction(true))),
    );

    app.add_observer(reset_override_on_remove_nav);
    app.add_observer(override_interaction_on_click);
}

/// Set initial focus to top left-most [`AutoDirectionalNavigation`].
fn set_input_focus(
    query: Query<(Entity, &UiGlobalTransform), With<AutoDirectionalNavigation>>,
    mut input_focus: ResMut<InputFocus>,
) {
    if let Some(button) = query
        .iter()
        .min_by(|(_, transform), (_, other)| {
            transform
                .translation
                .y
                .total_cmp(&other.translation.y)
                .then_with(|| transform.translation.x.total_cmp(&other.translation.x))
        })
        .map(|(e, _)| e)
    {
        input_focus.set(button);
    }
}

/// Reset all [`InteractionOverride`]s.
fn reset_interaction_overrides(
    query: Query<&mut InteractionOverride, With<AutoDirectionalNavigation>>,
) {
    for mut interaction_override in query {
        interaction_override.set_new(Interaction::None);
    }
}

/// Enable [`OverrideInteraction`] if [`UiNavActionSet`] is not empty.
fn override_interaction(
    mut next_state: ResMut<NextState<OverrideInteraction>>,
    action_set: Res<UiNavActionSet>,
) {
    if !action_set.0.is_empty() {
        (*next_state).set_if_neq(OverrideInteraction(true));
    }
}

/// Set [`OverrideInteraction`] to false if any [`Interaction`] with [`AutoDirectionalNavigation`] is not [`Interaction::None`].
fn reset_override(
    query: Query<&Interaction, With<AutoDirectionalNavigation>>,
    mut next_state: ResMut<NextState<OverrideInteraction>>,
) {
    if query.iter().any(|i| *i != Interaction::None) {
        (*next_state).set_if_neq(OverrideInteraction(false));
    }
}

/// Set correct [`InteractionOverride`] for focused [`AutoDirectionalNavigation`]s.
fn override_interaction_on_focus(
    query: Query<(Entity, &mut InteractionOverride), With<AutoDirectionalNavigation>>,
    input_focus: Res<InputFocus>,
    input_focus_visible: Res<InputFocusVisible>,
) {
    if !input_focus_visible.0 {
        return;
    }
    for (entity, mut interaction_override) in query {
        if input_focus.0 == Some(entity) {
            interaction_override.set_new_if_current(Interaction::None, Interaction::Hovered);
        } else {
            interaction_override.set_new_if_current(Interaction::Hovered, Interaction::None);
        }
    }
}

/// Set correct [`InteractionOverride`] for selected [`AutoDirectionalNavigation`]s on press.
fn override_interaction_on_click(
    event: On<Pointer<Click>>,
    mut query: Query<&mut InteractionOverride, With<AutoDirectionalNavigation>>,
    input_focus_visible: Res<InputFocusVisible>,
) {
    if !input_focus_visible.0 {
        return;
    }
    if let Ok(mut interaction_override) = query.get_mut(event.entity) {
        interaction_override.set_new_if_current(Interaction::Hovered, Interaction::Pressed);
    };
}

/// Set correct [`InteractionOverride`] for selected [`AutoDirectionalNavigation`]s on release.
fn override_interaction_on_release(
    query: Query<&mut InteractionOverride, With<AutoDirectionalNavigation>>,
    action_set: Res<UiNavActionSet>,
    input_focus_visible: Res<InputFocusVisible>,
) {
    if !input_focus_visible.0 || !action_set.0.contains(&UiNavAction::Select(false)) {
        return;
    }
    for mut interaction_override in query {
        interaction_override.set_new_if_current(Interaction::Pressed, Interaction::None);
    }
}

/// Navigate to [`UiNavActionSet::direction()`].
fn navigate(mut navigator: AutoDirectionalNavigator, action_set: Res<UiNavActionSet>) {
    // Navigate to `maybe_direction`.
    let direction = action_set.direction().map(CompassOctant::from);
    if let Some(direction) = direction
        && let Err(_e) = navigator.navigate(direction)
    {
        warn!("{}", WARN_INVALID_UI_NAV);
    }
}

/// Trigger [`Pointer<Over>`] on focused [`Entity`]s.
fn hover_focused(
    mut commands: Commands,
    input_focus: Res<InputFocus>,
    mut last_entity: Local<Option<Entity>>,
) {
    if input_focus.0 != *last_entity
        && let Some(entity) = input_focus.0
    {
        // NOTE: Since we only need to trigger the pointer hover for the entity,
        //       we are mostly using placeholder values.
        commands.trigger(Pointer::<Over> {
            entity,
            pointer_id: PointerId::Mouse,
            pointer_location: Location {
                target: NormalizedRenderTarget::None {
                    width: 0,
                    height: 0,
                },
                position: Vec2::ZERO,
            },
            event: Over {
                hit: HitData {
                    camera: Entity::PLACEHOLDER,
                    depth: 0.0,
                    position: None,
                    normal: None,
                },
            },
        });
        *last_entity = Some(entity);
    }
}

/// Trigger [`Pointer<Click>`] on focused [`Entity`]s mapped to [`UiNavAction::Select`] in [`UiNavActionSet`].
fn click_focused(
    mut commands: Commands,
    action_set: Res<UiNavActionSet>,
    input_focus: Res<InputFocus>,
) {
    if action_set.0.contains(&UiNavAction::Select(true))
        && let Some(entity) = input_focus.0
    {
        // NOTE: Since we only need to trigger the pointer click for the entity,
        //       we are mostly using placeholder values.
        commands.trigger(Pointer::<Click> {
            entity,
            pointer_id: PointerId::Mouse,
            pointer_location: Location {
                target: NormalizedRenderTarget::None {
                    width: 0,
                    height: 0,
                },
                position: Vec2::ZERO,
            },
            event: Click {
                button: PointerButton::Primary,
                hit: HitData {
                    camera: Entity::PLACEHOLDER,
                    depth: 0.0,
                    position: None,
                    normal: None,
                },
                duration: Duration::from_secs_f32(0.1),
            },
        });
    }
}

/// Set [`OverrideInteraction`] to false.
fn reset_override_on_remove_nav(
    _: On<Remove, AutoDirectionalNavigation>,
    mut next_state: ResMut<NextState<OverrideInteraction>>,
) {
    (*next_state).set_if_neq(OverrideInteraction(false));
}
