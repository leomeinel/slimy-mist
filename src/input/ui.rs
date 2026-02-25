/*
 * File: ui.rs
 * Author: Leopold Johannes Meinel (leo@meinel.dev)
 * -----
 * Copyright (c) 2026 Leopold Johannes Meinel & contributors
 * SPDX ID: Apache-2.0
 * URL: https://www.apache.org/licenses/LICENSE-2.0
 */

use bevy::{
    input::mouse::{MouseScrollUnit, MouseWheel},
    platform::collections::HashSet,
    prelude::*,
};

use crate::{
    AppSystems,
    input::pointer::Swipe as _,
    ui::scroll::{InputScroll, ScrollAction},
    utils::run_conditions::component_is_present,
};

pub(super) fn plugin(app: &mut App) {
    // Insert resources
    app.init_resource::<UiNavActionSet>();

    app.add_systems(
        PreUpdate,
        process_inputs.run_if(component_is_present::<UiNav>),
    );

    app.add_systems(
        Update,
        (
            input_scroll_directional_nav,
            input_scroll_mouse_wheel,
            input_scroll_touch,
        )
            .in_set(AppSystems::RecordInput),
    );
}

/// Marker [`Component`] for directional navigation.
#[derive(Component)]
pub(crate) struct UiNav;

/// Action for UI navigation.
#[derive(Debug, PartialEq, Eq, Hash)]
pub(crate) enum UiNavAction {
    Up,
    Down,
    Left,
    Right,
    Select(bool),
}
impl UiNavAction {
    pub(crate) fn variants() -> Vec<Self> {
        vec![
            UiNavAction::Up,
            UiNavAction::Down,
            UiNavAction::Left,
            UiNavAction::Right,
            UiNavAction::Select(true),
            UiNavAction::Select(false),
        ]
    }
    pub(crate) fn keycode(&self) -> KeyCode {
        match self {
            UiNavAction::Up => KeyCode::ArrowUp,
            UiNavAction::Down => KeyCode::ArrowDown,
            UiNavAction::Left => KeyCode::ArrowLeft,
            UiNavAction::Right => KeyCode::ArrowRight,
            UiNavAction::Select(_) => KeyCode::Enter,
        }
    }
    pub(crate) fn gamepad_button(&self) -> GamepadButton {
        match self {
            UiNavAction::Up => GamepadButton::DPadUp,
            UiNavAction::Down => GamepadButton::DPadDown,
            UiNavAction::Left => GamepadButton::DPadLeft,
            UiNavAction::Right => GamepadButton::DPadRight,
            UiNavAction::Select(_) => GamepadButton::South,
        }
    }
    pub(crate) fn try_from_vec2(vec2: Vec2) -> Option<UiNavAction> {
        if vec2.x.abs() > vec2.y.abs() {
            Some(if vec2.x > 0. { Self::Right } else { Self::Left })
        } else if vec2.y != 0. {
            Some(if vec2.y > 0. { Self::Up } else { Self::Down })
        } else {
            None
        }
    }
}

/// [`HashSet`] containing currently relevant [`UiNavAction`]s.
#[derive(Default, Resource)]
pub(crate) struct UiNavActionSet(pub(crate) HashSet<UiNavAction>);
impl UiNavActionSet {
    pub(crate) fn direction(&self) -> Option<Dir2> {
        let net_east_west =
            self.0.contains(&UiNavAction::Right) as i8 - self.0.contains(&UiNavAction::Left) as i8;
        let net_north_south =
            self.0.contains(&UiNavAction::Up) as i8 - self.0.contains(&UiNavAction::Down) as i8;
        Dir2::from_xy(net_east_west as f32, net_north_south as f32).ok()
    }
}

// FIXME: I'm pretty sure that right stick joystick input is broken. For now I can't further describe how
//        since I only have gamepads that are very broken, but I will test this further.
//        Also for future testing use https://gamepadtest.com
/// Process inputs and insert [`UiNavAction`] into [`UiNavActionSet`].
fn process_inputs(
    gamepad_query: Query<&Gamepad>,
    mut action_set: ResMut<UiNavActionSet>,
    keyboard: Res<ButtonInput<KeyCode>>,
) {
    action_set.0.clear();

    if let Some(action) = gamepad_query
        .iter()
        .find_map(|g| UiNavAction::try_from_vec2(g.right_stick()))
    {
        action_set.0.insert(action);
        return;
    };

    for action in UiNavAction::variants() {
        let on_just_pressed = action != UiNavAction::Select(false);
        let just_pressed = keyboard.just_pressed(action.keycode())
            || gamepad_query
                .iter()
                .any(|g| g.just_pressed(action.gamepad_button()));
        let just_released = keyboard.just_released(action.keycode())
            || gamepad_query
                .iter()
                .any(|g| g.just_released(action.gamepad_button()));

        if (on_just_pressed && just_pressed) || (!on_just_pressed && just_released) {
            action_set.0.insert(action);
        }
    }
}

/// Trigger [`Scroll`] for [`Entity`] with [`InputScroll`] from [`UiNavActionSet`].
///
/// - This assumes that only a single [`InputScroll`] is present.
fn input_scroll_directional_nav(
    scroll: Single<(Entity, &Node, &InputScroll)>,
    mut commands: Commands,
    action_set: Res<UiNavActionSet>,
) {
    let (entity, node, scroll) = *scroll;
    let Some(delta) = action_set.direction().map(|d| -d.as_vec2() * scroll.0) else {
        return;
    };
    if (node.align_items == AlignItems::Center || delta.x == 0.)
        && (node.justify_content == JustifyContent::Center || delta.y == 0.)
    {
        return;
    }

    commands.trigger(ScrollAction { entity, delta });
}

/// Trigger [`Scroll`] for [`Entity`] with [`InputScroll`] from [`Touches`].
///
/// - This assumes that only a single [`InputScroll`] is present.
fn input_scroll_touch(
    scroll: Single<(Entity, &Node, &InputScroll)>,
    mut commands: Commands,
    touches: Res<Touches>,
) {
    for touch in touches.iter() {
        let (entity, node, scroll) = *scroll;
        if ((node.align_items == AlignItems::Center || scroll.0.x == 0.)
            && (node.justify_content == JustifyContent::Center || scroll.0.y == 0.))
            || !touch.is_vertical_swipe()
        {
            continue;
        }
        let delta = -touch.delta();

        commands.trigger(ScrollAction { entity, delta });
    }
}

/// Trigger [`Scroll`] for [`Entity`] with [`InputScroll`] from [`MouseWheel`].
///
/// - This assumes that only a single [`InputScroll`] is present.
fn input_scroll_mouse_wheel(
    mut reader: MessageReader<MouseWheel>,
    scroll: Single<(Entity, &Node, &InputScroll)>,
    mut commands: Commands,
    keyboard: Res<ButtonInput<KeyCode>>,
) {
    for mouse_wheel in reader.read() {
        let (entity, node, scroll) = *scroll;
        if (node.align_items == AlignItems::Center || scroll.0.x == 0.)
            && (node.justify_content == JustifyContent::Center || scroll.0.y == 0.)
        {
            continue;
        }

        let mut delta = -Vec2::new(mouse_wheel.x, mouse_wheel.y);
        if mouse_wheel.unit == MouseScrollUnit::Line {
            delta *= scroll.0;
        }
        if keyboard.any_pressed([KeyCode::ControlLeft, KeyCode::ControlRight]) {
            std::mem::swap(&mut delta.x, &mut delta.y);
        }

        commands.trigger(ScrollAction { entity, delta });
    }
}
