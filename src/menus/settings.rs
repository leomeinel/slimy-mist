/*
 * File: settings.rs
 * Author: Leopold Johannes Meinel (leo@meinel.dev)
 * -----
 * Copyright (c) 2026 Leopold Johannes Meinel & contributors
 * SPDX ID: Apache-2.0
 * URL: https://www.apache.org/licenses/LICENSE-2.0
 * -----
 * Heavily inspired by: https://github.com/TheBevyFlock/bevy_new_2d
 */

//! The settings menu.
//!
//! Additional settings and accessibility options should go here.

use bevy::{audio::Volume, input::common_conditions::input_just_pressed, prelude::*};

use crate::{
    input::joystick::{JoystickID, JoystickState},
    logging::error::*,
    menus::Menu,
    screens::Screen,
    ui::{
        interaction::apply_palette,
        prelude::*,
        widgets::{ButtonBase, ButtonText},
    },
};

pub(super) fn plugin(app: &mut App) {
    // Open settings menu on state
    app.add_systems(OnEnter(Menu::Settings), spawn_settings_menu);

    // Exit settings menu on pressing Escape
    app.add_systems(
        Update,
        go_back.run_if(in_state(Menu::Settings).and(input_just_pressed(KeyCode::Escape))),
    );

    // Handle changes to settings
    app.add_systems(
        Update,
        (
            update_joystick_button.before(apply_palette),
            update_global_volume_label,
        )
            .run_if(in_state(Menu::Settings)),
    );
}

/// Global volume label marker
#[derive(Component, Reflect)]
#[reflect(Component)]
struct GlobalVolumeLabel;

/// Toggle joystick button marker
#[derive(Component, Reflect)]
#[reflect(Component)]
struct ToggleJoystickButton<const ID: u8>;

/// Spawn settings menu
fn spawn_settings_menu(mut commands: Commands, font: Res<UiFontHandle>) {
    commands.spawn((
        widgets::ui_root("Settings Menu"),
        GlobalZIndex(2),
        DespawnOnExit(Menu::Settings),
        children![
            widgets::header("Settings", font.0.clone()),
            settings_grid(font.0.clone()),
            widgets::button_large("Back", font.0.clone(), go_back_on_click),
        ],
    ));
}

/// Custom settings grid
fn settings_grid(font: Handle<Font>) -> impl Bundle {
    (
        Name::new("Settings Grid"),
        Node {
            display: Display::Grid,
            row_gap: px(10),
            column_gap: px(30),
            grid_template_columns: RepeatedGridTrack::px(2, 400.0),
            ..default()
        },
        children![
            settings_label(font.clone(), "Master Volume"),
            global_volume_widget(font.clone()),
            settings_label(font.clone(), "Joystick"),
            toggle_joystick_widget(font.clone()),
        ],
    )
}

/// Label for configuration in settings.
fn settings_label(font: Handle<Font>, label: &'static str) -> (impl Bundle, Node) {
    (
        widgets::label(label, font.clone()),
        Node {
            justify_self: JustifySelf::End,
            ..default()
        },
    )
}

/// Widget to adjust global volume
fn global_volume_widget(font: Handle<Font>) -> impl Bundle {
    (
        Name::new("Global Volume Widget"),
        Node {
            justify_self: JustifySelf::Start,
            ..default()
        },
        children![
            widgets::button_small("-", font.clone(), lower_global_volume_on_click),
            (
                Name::new("Current Volume"),
                Node {
                    // FIXME: Horizontal alignment is currently incorrect and should not be hardcoded.
                    padding: UiRect::horizontal(px(10)),
                    justify_content: JustifyContent::Center,
                    ..default()
                },
                children![(GlobalVolumeLabel, widgets::label("", font.clone()))],
            ),
            widgets::button_small("+", font.clone(), raise_global_volume_on_click),
        ],
    )
}

/// Minimum global volume
const MIN_VOLUME: f32 = 0.0;
/// Maximum global volume
const MAX_VOLUME: f32 = 3.0;

/// Lower global volume
fn lower_global_volume_on_click(_: On<Pointer<Click>>, mut global_volume: ResMut<GlobalVolume>) {
    let linear = (global_volume.volume.to_linear() - 0.1).max(MIN_VOLUME);
    global_volume.volume = Volume::Linear(linear);
}

/// Raise global volume
fn raise_global_volume_on_click(_: On<Pointer<Click>>, mut global_volume: ResMut<GlobalVolume>) {
    let linear = (global_volume.volume.to_linear() + 0.1).min(MAX_VOLUME);
    global_volume.volume = Volume::Linear(linear);
}

/// Update global volume label that displays volume
fn update_global_volume_label(
    mut label: Single<&mut Text, With<GlobalVolumeLabel>>,
    global_volume: Res<GlobalVolume>,
) {
    let percent = 100.0 * global_volume.volume.to_linear();
    label.0 = format!("{percent:3.0}%");
}

/// Widget to toggle movement joystick
fn toggle_joystick_widget(font: Handle<Font>) -> impl Bundle {
    (
        Name::new("Toggle Joystick Widget"),
        Node {
            justify_self: JustifySelf::Start,
            ..default()
        },
        children![(
            Name::new("Toggle Joystick Button"),
            Node {
                // FIXME: Horizontal alignment is currently incorrect and should not be hardcoded.
                padding: UiRect::horizontal(px(40)),
                justify_content: JustifyContent::Center,
                ..default()
            },
            children![(
                ToggleJoystickButton::<{ JoystickID::Movement as u8 }>,
                widgets::switch_medium("", font.clone(), toggle_joystick_on_click),
            )]
        )],
    )
}

/// Toggle [`JoystickState<ID>`].
fn toggle_joystick_on_click(
    _: On<Pointer<Click>>,
    mut next_state: ResMut<NextState<JoystickState<{ JoystickID::Movement as u8 }>>>,
    state: Res<State<JoystickState<{ JoystickID::Movement as u8 }>>>,
) {
    (*next_state).set_if_neq(JoystickState::<{ JoystickID::Movement as u8 }>::Toggled(
        !state.is_active(),
    ));
}

// FIXME: When using keyboard and thus not triggering an Interaction update on select, the hover color does not change.
/// Update global volume label that displays volume
fn update_joystick_button(
    children: Single<&Children, With<ToggleJoystickButton<{ JoystickID::Movement as u8 }>>>,
    mut base_query: Query<(&mut BackgroundColor, &Children), (With<ButtonBase>, Without<Button>)>,
    mut surface_query: Query<
        (&mut InteractionPalette, &Children),
        (With<Button>, Without<ButtonBase>),
    >,
    mut text_query: Query<&mut Text, With<ButtonText>>,
    state: Res<State<JoystickState<{ JoystickID::Movement as u8 }>>>,
) {
    let (base_color, surface_color, hover_color, new_text) = if state.is_active() {
        (
            SWITCH_BASE_ON_BACKGROUND,
            SWITCH_ON_BACKGROUND,
            SWITCH_ON_HOVERED_BACKGROUND,
            "On",
        )
    } else {
        (
            SWITCH_BASE_OFF_BACKGROUND,
            SWITCH_OFF_BACKGROUND,
            SWITCH_OFF_HOVERED_BACKGROUND,
            "Off",
        )
    };

    // `ButtonBase`
    let child = children
        .iter()
        .find(|e| base_query.contains(*e))
        .expect(ERR_INVALID_CHILDREN);
    let (mut background, children) = base_query.get_mut(child).expect(ERR_INVALID_CHILDREN);
    background.0 = base_color.into();

    // `Button`
    let child = children
        .iter()
        .find(|e| surface_query.contains(*e))
        .expect(ERR_INVALID_CHILDREN);
    let (mut palette, children) = surface_query.get_mut(child).expect(ERR_INVALID_CHILDREN);
    palette.none = surface_color.into();
    palette.hovered = hover_color.into();

    // `ButtonText`
    let child = children
        .iter()
        .find(|e| text_query.contains(*e))
        .expect(ERR_INVALID_CHILDREN);
    let mut text = text_query.get_mut(child).expect(ERR_INVALID_CHILDREN);
    text.0 = new_text.to_uppercase();
}

/// Call [`go_back`] on pointer click.
fn go_back_on_click(
    _: On<Pointer<Click>>,
    next_state: ResMut<NextState<Menu>>,
    state: Res<State<Screen>>,
) {
    go_back(next_state, state);
}

/// Go back.
fn go_back(mut next_state: ResMut<NextState<Menu>>, state: Res<State<Screen>>) {
    (*next_state).set_if_neq(state.back_menu());
}
