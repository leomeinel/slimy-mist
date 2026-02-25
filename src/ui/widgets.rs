/*
 * File: widgets.rs
 * Author: Leopold Johannes Meinel (leo@meinel.dev)
 * -----
 * Copyright (c) 2026 Leopold Johannes Meinel & contributors
 * SPDX ID: Apache-2.0
 * URL: https://www.apache.org/licenses/LICENSE-2.0
 * -----
 * Heavily inspired by: https://github.com/TheBevyFlock/bevy_new_2d
 */

//! Helper functions for creating common widgets.

use bevy::{
    ecs::{spawn::SpawnWith, system::IntoObserverSystem},
    prelude::*,
    ui::auto_directional_navigation::AutoDirectionalNavigation,
};

use crate::{
    input::ui::UiNav,
    ui::{
        prelude::*,
        scroll::{AutoScroll, InputScroll},
    },
};

/// Button base marker
#[derive(Component, Reflect)]
#[reflect(Component)]
pub(crate) struct ButtonBase;

/// Button base marker
#[derive(Component, Reflect)]
#[reflect(Component)]
pub(crate) struct ButtonText;

/// A builder for creating [`Button`] [`Bundle`]s with customizable appearance, text, and interaction behavior.
struct ButtonBuilder {
    name: &'static str,
    base_background: Color,
    surface_background: Color,
    hovered_background: Color,
    text: &'static str,
    font: Handle<Font>,
    font_size: f32,
}
impl ButtonBuilder {
    fn new(
        name: &'static str,
        base_background: Color,
        surface_background: Color,
        hovered_background: Color,
        text: &'static str,
        font: Handle<Font>,
        font_size: f32,
    ) -> Self {
        Self {
            name,
            base_background,
            surface_background,
            hovered_background,
            text,
            font,
            font_size,
        }
    }

    /// Builds a [`Button`] [`Bundle`] and attaches an [`Observer`].
    ///
    /// ## Traits
    ///
    /// - `E` must implement [`EntityEvent`].
    /// - `B` must implement [`Bundle`].
    fn build_with<E, B, M>(
        self,
        action: impl IntoObserverSystem<E, B, M>,
        base_bundle: impl Bundle,
        surface_bundle: impl Bundle,
    ) -> impl Bundle
    where
        E: EntityEvent,
        B: Bundle,
    {
        let system = IntoObserverSystem::into_system(action);
        (
            Name::new(self.name),
            Node::default(),
            Children::spawn(SpawnWith(move |parent: &mut ChildSpawner| {
                parent
                    .spawn((
                        Name::new(format!("{} Base", self.name)),
                        ButtonBase,
                        BackgroundColor(self.base_background),
                        base_bundle,
                        ZIndex(0),
                    ))
                    .with_children(|base| {
                        base.spawn((
                            Name::new(format!("{} Surface", self.name)),
                            Button,
                            BackgroundColor(self.surface_background),
                            InteractionPalette {
                                none: self.surface_background,
                                hovered: self.hovered_background,
                                pressed: BUTTON_PRESSED_BACKGROUND,
                            },
                            InteractionOverride::default(),
                            AutoDirectionalNavigation::default(),
                            UiNav,
                            surface_bundle,
                            ZIndex(1),
                            children![(
                                Name::new(format!("{} Text", self.name)),
                                ButtonText,
                                Text(self.text.to_uppercase()),
                                TextFont::from(self.font).with_font_size(self.font_size),
                                TextColor(BUTTON_TEXT.into()),
                                // Don't bubble picking events from the text up to the button.
                                Pickable::IGNORE,
                                ZIndex(2),
                            )],
                        ))
                        .observe(system);
                    });
            })),
        )
    }
}

/// A non-scrolling root UI [`Bundle`] that fills the window and centers its content.
pub(crate) fn ui_root(name: &'static str) -> impl Bundle {
    ui_root_bundle(name, Overflow::DEFAULT)
}

/// An auto-scrolling root UI [`Bundle`] that fills the window and centers its content.
pub(crate) fn ui_root_auto_scroll(name: &'static str) -> impl Bundle {
    (
        ui_root_bundle(name, Overflow::scroll_y()),
        AutoScroll(Vec2::new(0., BODY_FONT_SIZE / 100.)),
        InputScroll(Vec2::new(0., BODY_FONT_SIZE)),
        UiNav,
    )
}

/// A root UI [`Bundle`] that fills the window and centers its content.
fn ui_root_bundle(name: &'static str, overflow: Overflow) -> impl Bundle {
    (
        Name::new(name),
        Node {
            position_type: PositionType::Absolute,
            width: percent(100),
            height: percent(100),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            flex_direction: FlexDirection::Column,
            row_gap: px(20),
            overflow,
            ..default()
        },
        // Don't block picking events for other UI roots.
        Pickable::IGNORE,
    )
}

/// A simple header label. Bigger than [`label`].
pub(crate) fn header(text: impl Into<String>, font: Handle<Font>) -> impl Bundle {
    styled_text("Header", HEADER_TEXT.into(), text, font, HEADER_FONT_SIZE)
}

/// A simple text label.
pub(crate) fn label(text: impl Into<String>, font: Handle<Font>) -> impl Bundle {
    styled_text("Label", LABEL_TEXT.into(), text, font, BODY_FONT_SIZE)
}

/// A simple styled text
fn styled_text(
    name: &'static str,
    color: Color,
    text: impl Into<String>,
    font: Handle<Font>,
    font_size: f32,
) -> impl Bundle {
    (
        Name::new(name),
        Text(text.into()),
        TextFont::from(font).with_font_size(font_size),
        TextColor(color),
    )
}

/// A large rounded [`Button`] with text and an action defined as an [`Observer`].
///
/// ## Traits
///
/// - `E` must implement [`EntityEvent`].
/// - `B` must implement [`Bundle`].
pub(crate) fn button_large<E, B, M>(
    text: &'static str,
    font: Handle<Font>,
    action: impl IntoObserverSystem<E, B, M>,
) -> impl Bundle
where
    E: EntityEvent,
    B: Bundle,
{
    button(
        text,
        font,
        action,
        px(400),
        Some(4.5),
        BorderRadius::all(px(30)),
        8,
    )
}

/// A medium rounded switch [`Button`] with text and an action defined as an [`Observer`].
///
/// ## Traits
///
/// - `E` must implement [`EntityEvent`].
/// - `B` must implement [`Bundle`].
pub(crate) fn switch_medium<E, B, M>(
    text: &'static str,
    font: Handle<Font>,
    action: impl IntoObserverSystem<E, B, M>,
) -> impl Bundle
where
    E: EntityEvent,
    B: Bundle,
{
    switch(
        text,
        font,
        action,
        px(60),
        Some(2.),
        BorderRadius::all(px(30)),
        4,
    )
}

/// A small circle [`Button`] with text and an action defined as an [`Observer`].
///
/// ## Traits
///
/// - `E` must implement [`EntityEvent`].
/// - `B` must implement [`Bundle`].
pub(crate) fn button_small<E, B, M>(
    text: &'static str,
    font: Handle<Font>,
    action: impl IntoObserverSystem<E, B, M>,
) -> impl Bundle
where
    E: EntityEvent,
    B: Bundle,
{
    button(text, font, action, px(30), Some(1.), BorderRadius::MAX, 4)
}

/// A [`Button`] with text and an action defined as an [`Observer`].
///
/// ## Traits
///
/// - `E` must implement [`EntityEvent`].
/// - `B` must implement [`Bundle`].
fn button<E, B, M>(
    text: &'static str,
    font: Handle<Font>,
    action: impl IntoObserverSystem<E, B, M>,
    width: Val,
    aspect_ratio: Option<f32>,
    border_radius: BorderRadius,
    offset: i32,
) -> impl Bundle
where
    E: EntityEvent,
    B: Bundle,
{
    let builder = ButtonBuilder::new(
        "Button",
        BUTTON_BASE_BACKGROUND.into(),
        BUTTON_BACKGROUND.into(),
        BUTTON_HOVERED_BACKGROUND.into(),
        text,
        font,
        HEADER_FONT_SIZE,
    );
    let (base_bundle, surface_bundle) = button_bundles(width, aspect_ratio, border_radius, offset);
    builder.build_with(action, base_bundle, surface_bundle)
}

/// A switch [`Button`] with text and an action defined as an [`Observer`].
///
/// ## Traits
///
/// - `E` must implement [`EntityEvent`].
/// - `B` must implement [`Bundle`].
fn switch<E, B, M>(
    text: &'static str,
    font: Handle<Font>,
    action: impl IntoObserverSystem<E, B, M>,
    width: Val,
    aspect_ratio: Option<f32>,
    border_radius: BorderRadius,
    offset: i32,
) -> impl Bundle
where
    E: EntityEvent,
    B: Bundle,
{
    let builder = ButtonBuilder::new(
        "Switch",
        SWITCH_BASE_OFF_BACKGROUND.into(),
        SWITCH_OFF_BACKGROUND.into(),
        SWITCH_OFF_HOVERED_BACKGROUND.into(),
        text,
        font,
        BODY_FONT_SIZE,
    );
    let (base_bundle, surface_bundle) = button_bundles(width, aspect_ratio, border_radius, offset);
    builder.build_with(action, base_bundle, surface_bundle)
}

/// Tuples meant to be used as [`Bundle`]s for [`Button`].
fn button_bundles(
    width: Val,
    aspect_ratio: Option<f32>,
    border_radius: BorderRadius,
    offset: i32,
) -> (Node, (Node, NodeOffset)) {
    let common_node = Node {
        width,
        aspect_ratio,
        align_items: AlignItems::Center,
        justify_content: JustifyContent::Center,
        border_radius,
        ..default()
    };

    (
        Node {
            overflow: Overflow::visible(),
            ..common_node.clone()
        },
        (
            Node {
                bottom: px(offset),
                position_type: PositionType::Absolute,
                ..common_node
            },
            NodeOffset(IVec2::new(0, offset)),
        ),
    )
}
