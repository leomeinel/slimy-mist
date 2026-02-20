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

use std::borrow::Cow;

use bevy::{
    ecs::{spawn::SpawnWith, system::IntoObserverSystem},
    prelude::*,
    ui::auto_directional_navigation::AutoDirectionalNavigation,
};

use crate::ui::{palette::*, prelude::*};

/// Font size for any header.
pub(crate) const HEADER_FONT_SIZE: f32 = 36.;
/// Font size for any body.
pub(crate) const BODY_FONT_SIZE: f32 = 18.;

/// Wrapper for [`Handle<Font>`] for the ui.
#[derive(Resource, Default)]
pub(crate) struct UiFontHandle(pub(crate) Handle<Font>);

/// Offset that stores the offset for a [`Node`].
///
/// Can apply to [`Node::left`] and [`Node::bottom`] according to [`Self::0`].
#[derive(Component, Default)]
pub(crate) struct NodeOffset(pub(crate) IVec2);
/// A root UI node that fills the window and centers its content.
pub(crate) fn ui_root(name: impl Into<Cow<'static, str>>) -> impl Bundle {
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
            ..default()
        },
        // Don't block picking events for other UI roots.
        Pickable::IGNORE,
    )
}

/// A simple header label. Bigger than [`label`].
pub(crate) fn header(text: impl Into<String>, font: Handle<Font>) -> impl Bundle {
    (
        Name::new("Header"),
        Text(text.into()),
        TextFont::from(font).with_font_size(HEADER_FONT_SIZE),
        TextColor(HEADER_TEXT.into()),
    )
}

/// A simple text label.
pub(crate) fn label(text: impl Into<String>, font: Handle<Font>) -> impl Bundle {
    (
        Name::new("Label"),
        Text(text.into()),
        TextFont::from(font).with_font_size(BODY_FONT_SIZE),
        TextColor(LABEL_TEXT.into()),
    )
}

/// A large rounded button with text and an action defined as an [`Observer`].
///
/// ## Traits
///
/// - `E` must implement [`EntityEvent`].
/// - `B` must implement [`Bundle`].
/// - `I` must implement [`IntoObserverSystem<E, B, M>`].
pub(crate) fn button_large<E, B, M, I>(
    text: impl Into<String>,
    font: Handle<Font>,
    action: I,
) -> impl Bundle
where
    E: EntityEvent,
    B: Bundle,
    I: IntoObserverSystem<E, B, M>,
{
    let offset = 8;
    let node = Node {
        width: px(400),
        aspect_ratio: Some(4.5),
        align_items: AlignItems::Center,
        justify_content: JustifyContent::Center,
        border_radius: BorderRadius::all(px(30)),
        ..default()
    };
    button(
        text,
        font,
        action,
        Node {
            overflow: Overflow::visible(),
            ..node.clone()
        },
        (
            Node {
                bottom: px(offset),
                position_type: PositionType::Absolute,
                ..node
            },
            NodeOffset(IVec2::new(0, offset)),
        ),
    )
}

//FIXME: Change cursor to pointer on button hover

/// A small rounded button with text and an action defined as an [`Observer`].
///
/// ## Traits
///
/// - `E` must implement [`EntityEvent`].
/// - `B` must implement [`Bundle`].
/// - `I` must implement [`IntoObserverSystem<E, B, M>`].
pub(crate) fn button_small<E, B, M, I>(
    text: impl Into<String>,
    font: Handle<Font>,
    action: I,
) -> impl Bundle
where
    E: EntityEvent,
    B: Bundle,
    I: IntoObserverSystem<E, B, M>,
{
    let offset = 4;
    let node = Node {
        width: px(30),
        aspect_ratio: Some(1.),
        align_items: AlignItems::Center,
        justify_content: JustifyContent::Center,
        border_radius: BorderRadius::MAX,
        ..default()
    };
    button(
        text,
        font,
        action,
        Node {
            overflow: Overflow::visible(),
            ..node.clone()
        },
        (
            Node {
                bottom: px(offset),
                position_type: PositionType::Absolute,
                ..node
            },
            NodeOffset(IVec2::new(0, offset)),
        ),
    )
}

/// A button with text and an action defined as an [`Observer`].
///
/// ## Traits
///
/// - `E` must implement [`EntityEvent`].
/// - `B` must implement [`Bundle`].
/// - `I` must implement [`IntoObserverSystem<E, B, M>`].
fn button<E, B, M, I>(
    text: impl Into<String>,
    font: Handle<Font>,
    action: I,
    base: impl Bundle,
    surface: impl Bundle,
) -> impl Bundle
where
    E: EntityEvent,
    B: Bundle,
    I: IntoObserverSystem<E, B, M>,
{
    let text = text.into().to_uppercase();
    let action = IntoObserverSystem::into_system(action);
    (
        Name::new("Button"),
        Node::default(),
        Children::spawn(SpawnWith(|parent: &mut ChildSpawner| {
            parent
                .spawn((
                    Name::new("Button Base"),
                    BackgroundColor(BUTTON_BASE_BACKGROUND.into()),
                    base,
                    ZIndex(0),
                ))
                .with_children(|base| {
                    base.spawn((
                        Name::new("Button Surface"),
                        Button,
                        BackgroundColor(BUTTON_BACKGROUND.into()),
                        InteractionPalette {
                            none: BUTTON_BACKGROUND.into(),
                            hovered: BUTTON_HOVERED_BACKGROUND.into(),
                            pressed: BUTTON_PRESSED_BACKGROUND,
                        },
                        InteractionOverride::default(),
                        AutoDirectionalNavigation::default(),
                        surface,
                        ZIndex(1),
                        children![(
                            Name::new("Button Text"),
                            Text(text),
                            TextFont::from(font).with_font_size(HEADER_FONT_SIZE),
                            TextColor(BUTTON_TEXT.into()),
                            // Don't bubble picking events from the text up to the button.
                            Pickable::IGNORE,
                            ZIndex(2),
                        )],
                    ))
                    .observe(action);
                });
        })),
    )
}
