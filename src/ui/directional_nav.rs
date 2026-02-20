/*
 * File: directional_nav.rs
 * Author: Leopold Johannes Meinel (leo@meinel.dev)
 * -----
 * Copyright (c) 2026 Leopold Johannes Meinel & contributors
 * SPDX ID: Apache-2.0
 * URL: https://www.apache.org/licenses/LICENSE-2.0
 * -----
 * Heavily inspired by: https://github.com/bevyengine/bevy/blob/latest/examples/ui/auto_directional_navigation.rs
 */

use core::time::Duration;

use bevy::{
    camera::NormalizedRenderTarget,
    input_focus::{
        InputDispatchPlugin, InputFocus, InputFocusVisible,
        directional_navigation::{AutoNavigationConfig, DirectionalNavigationPlugin},
    },
    math::{CompassOctant, Dir2},
    picking::{
        backend::HitData,
        pointer::{Location, PointerId},
    },
    platform::collections::HashSet,
    prelude::*,
    ui::auto_directional_navigation::{AutoDirectionalNavigation, AutoDirectionalNavigator},
};

use crate::{
    logging::warn::WARN_INVALID_UI_NAV,
    ui::interaction::{InteractionOverride, OverrideInteraction},
    utils::run_conditions::component_is_present,
};

pub(super) fn plugin(app: &mut App) {
    // Library plugins
    app.add_plugins((InputDispatchPlugin, DirectionalNavigationPlugin));

    // Insert resources
    app.init_resource::<DirectionalNavActionSet>();
    app.insert_resource(InputFocusVisible(true));
    app.insert_resource(AutoNavigationConfig {
        min_alignment_factor: 0.1,
        prefer_aligned: true,
        ..default()
    });

    // Process inputs, override `Interaction` and navigate
    app.add_systems(OnEnter(OverrideInteraction(true)), set_input_focus);
    app.add_systems(
        PreUpdate,
        (
            process_inputs.run_if(component_is_present::<AutoDirectionalNavigation>),
            (override_interaction_on_focus, navigate)
                .run_if(in_state(OverrideInteraction(true)))
                .chain(),
        )
            .chain(),
    );

    app.add_systems(
        Update,
        (hover_focused, click_focused).run_if(in_state(OverrideInteraction(true))),
    );

    // Set `OverrideInteraction` to false
    app.add_observer(on_remove_button);
}

/// Action for directional navigation.
#[derive(Debug, PartialEq, Eq, Hash)]
pub(crate) enum DirectionalNavAction {
    Up,
    Down,
    Left,
    Right,
    Select,
}
impl DirectionalNavAction {
    fn variants() -> Vec<Self> {
        vec![
            DirectionalNavAction::Up,
            DirectionalNavAction::Down,
            DirectionalNavAction::Left,
            DirectionalNavAction::Right,
            DirectionalNavAction::Select,
        ]
    }
    fn keycode(&self) -> KeyCode {
        match self {
            DirectionalNavAction::Up => KeyCode::ArrowUp,
            DirectionalNavAction::Down => KeyCode::ArrowDown,
            DirectionalNavAction::Left => KeyCode::ArrowLeft,
            DirectionalNavAction::Right => KeyCode::ArrowRight,
            DirectionalNavAction::Select => KeyCode::Enter,
        }
    }
    fn gamepad_button(&self) -> GamepadButton {
        match self {
            DirectionalNavAction::Up => GamepadButton::DPadUp,
            DirectionalNavAction::Down => GamepadButton::DPadDown,
            DirectionalNavAction::Left => GamepadButton::DPadLeft,
            DirectionalNavAction::Right => GamepadButton::DPadRight,
            DirectionalNavAction::Select => GamepadButton::South,
        }
    }
}

/// [`HashSet`] containing currently relevant [`DirectionalNavAction`]s.
#[derive(Default, Resource)]
struct DirectionalNavActionSet(HashSet<DirectionalNavAction>);

/// Process inputs and add correct [`DirectionalNavAction`] to [`DirectionalNavActionSet`].
///
/// This also sets [`OverrideInteraction`] to true if any input has been pressed.
fn process_inputs(
    gamepad_input: Query<&Gamepad>,
    mut next_state: ResMut<NextState<OverrideInteraction>>,
    mut action_set: ResMut<DirectionalNavActionSet>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    state: Res<State<OverrideInteraction>>,
) {
    action_set.0.clear();

    let mut any_pressed = false;
    for action in DirectionalNavAction::variants() {
        if keyboard_input.just_pressed(action.keycode())
            || gamepad_input
                .iter()
                .any(|g| g.just_pressed(action.gamepad_button()))
        {
            action_set.0.insert(action);
            any_pressed = true;
        }
    }

    if *state != OverrideInteraction(true) && any_pressed {
        next_state.set(OverrideInteraction(true));
    }
}

/// Set correct [`InteractionOverride`] for [`AutoDirectionalNavigation`]s.
fn override_interaction_on_focus(
    query: Query<(Entity, &mut InteractionOverride), With<AutoDirectionalNavigation>>,
    input_focus: Res<InputFocus>,
    input_focus_visible: Res<InputFocusVisible>,
) {
    for (entity, mut interaction_override) in query {
        if input_focus.0 == Some(entity) && input_focus_visible.0 {
            interaction_override.set_if_neq(InteractionOverride::Hovered);
        } else {
            interaction_override.set_if_neq(InteractionOverride::None);
        }
    }
}

/// Determine direction from directional fields of [`DirectionalNavActionSet`] and navigate.
fn navigate(mut navigator: AutoDirectionalNavigator, action_set: Res<DirectionalNavActionSet>) {
    // Use `bool` values as i8 and determine expected `maybe_direction`.
    let net_east_west = action_set.0.contains(&DirectionalNavAction::Right) as i8
        - action_set.0.contains(&DirectionalNavAction::Left) as i8;
    let net_north_south = action_set.0.contains(&DirectionalNavAction::Up) as i8
        - action_set.0.contains(&DirectionalNavAction::Down) as i8;
    let maybe_direction = Dir2::from_xy(net_east_west as f32, net_north_south as f32)
        .ok()
        .map(CompassOctant::from);

    // Navigate to `maybe_direction`.
    if let Some(direction) = maybe_direction
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

/// Trigger [`Pointer<Click>`] on focused [`Entity`]s mapped to [`DirectionalNavAction::Select`] in [`DirectionalNavActionSet`].
fn click_focused(
    mut commands: Commands,
    action_set: Res<DirectionalNavActionSet>,
    input_focus: Res<InputFocus>,
) {
    if action_set.0.contains(&DirectionalNavAction::Select)
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

/// Set [`OverrideInteraction`] to false if no [`AutoDirectionalNavigation`] is present.
fn on_remove_button(
    _: On<Remove, AutoDirectionalNavigation>,
    mut next_state: ResMut<NextState<OverrideInteraction>>,
    state: Res<State<OverrideInteraction>>,
) {
    if *state != OverrideInteraction(false) {
        next_state.set(OverrideInteraction(false));
    }
}
