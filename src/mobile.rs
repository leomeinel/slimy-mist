/*
 * File: mobile.rs
 * Author: Leopold Johannes Meinel (leo@meinel.dev)
 * -----
 * Copyright (c) 2026 Leopold Johannes Meinel & contributors
 * SPDX ID: Apache-2.0
 * URL: https://www.apache.org/licenses/LICENSE-2.0
 * -----
 * Heavily inspired by:
 * - https://github.com/bevyengine/bevy/tree/main/examples/mobile
 * - https://github.com/SergioRibera/virtual_joystick
 */

#[cfg(target_os = "android")]
mod android;

use bevy::{platform::collections::HashMap, prelude::*, winit::WinitSettings};
use bevy_asset_loader::asset_collection::AssetCollection;
use virtual_joystick::{
    JoystickFixed, NoAction, VirtualJoystickBundle, VirtualJoystickInteractionArea,
    VirtualJoystickNode, VirtualJoystickPlugin, VirtualJoystickUIBackground, VirtualJoystickUIKnob,
};

use crate::{logging::error::*, screens::Screen, ui::prelude::*};

pub(super) fn plugin(app: &mut App) {
    // Add library plugins
    app.add_plugins(VirtualJoystickPlugin::<u8>::default());

    // Add child plugins
    #[cfg(target_os = "android")]
    app.add_plugins(android::plugin);

    // Make the winit loop wait more aggressively when no user input is received
    // This can help reduce cpu usage on mobile devices
    app.insert_resource(WinitSettings::mobile());
}

/// Assets for joystick
#[derive(AssetCollection, Resource)]
pub(crate) struct JoystickAssets {
    #[asset(path = "images/ui/joystick-knob.webp")]
    #[asset(image(sampler(filter = linear)))]
    knob_image: Handle<Image>,
    #[asset(path = "images/ui/joystick-background.webp")]
    #[asset(image(sampler(filter = linear)))]
    background_image: Handle<Image>,
}

/// Enum representation of a joystick ID to have a single source of truth for IDs.
///
/// This can be used as a [`virtual_joystick::VirtualJoystickID`] after casting to [`u8`].
#[repr(u8)]
#[derive(Default)]
pub(crate) enum JoystickID {
    #[default]
    Movement,
}

/// Map of [`JoystickID`]s as [`u8`] mapped to their [`Rect`].
#[derive(Resource, Default)]
pub(crate) struct JoystickRectMap(HashMap<u8, Rect>);
impl JoystickRectMap {
    pub(crate) fn any_intersect_with(&self, point: Vec2) -> bool {
        self.0.iter().any(|(_, v)| v.contains(point))
    }
}

/// Update [`Rect`] representing [`VirtualJoystickInteractionArea`] mapped to `ID` in [`JoystickRectMap`].
///
/// ## Traits
///
/// - `const ID` will be used as [`VirtualJoystickNode::id`].
pub(crate) fn update_joystick_rect_map<const ID: u8>(
    node_query: Query<(&VirtualJoystickNode<u8>, &Children)>,
    interaction_area_query: Query<
        (&ComputedNode, &UiGlobalTransform),
        With<VirtualJoystickInteractionArea>,
    >,
    mut rect_map: ResMut<JoystickRectMap>,
) {
    let children = node_query
        .iter()
        .find(|(n, _)| n.id == ID)
        .map(|(_, c)| c)
        .expect(ERR_INVALID_CHILDREN);

    if let Some((node, transform)) = children
        .iter()
        .find_map(|child| interaction_area_query.get(child).ok())
    {
        let factor = node.inverse_scale_factor;
        let rect = Rect::from_center_size(transform.translation * factor, node.size() * factor);
        rect_map.0.insert(ID, rect);
    }
}

/// Size of the joystick knob in pixels
const JOYSTICK_KNOB_SIZE: Vec2 = Vec2::splat(50.);
/// Size of the joystick background in pixels
const JOYSTICK_BACKGROUND_SIZE: Vec2 = Vec2::splat(100.);

/// Spawn joystick with `ID`.
///
/// ## Traits
///
/// - `const ID` will be used as [`VirtualJoystickNode::id`].
pub(crate) fn spawn_joystick<const ID: u8>(
    mut commands: Commands,
    joystick_assets: Res<JoystickAssets>,
) {
    let style = Node {
        position_type: PositionType::Absolute,
        width: px(JOYSTICK_BACKGROUND_SIZE.x),
        height: px(JOYSTICK_BACKGROUND_SIZE.y),
        left: vmin(10.),
        bottom: vmin(10.),
        ..default()
    };
    commands.spawn((
        VirtualJoystickBundle::new(
            VirtualJoystickNode::default()
                .with_id(ID)
                .with_behavior(JoystickFixed)
                .with_action(NoAction),
        )
        .set_style(style),
        DespawnOnExit(Screen::Gameplay),
        children![
            (
                VirtualJoystickInteractionArea,
                Node {
                    width: percent(100.),
                    height: percent(100.),
                    ..default()
                },
            ),
            (
                VirtualJoystickUIBackground,
                ImageNode {
                    color: JOYSTICK_IMAGE.into(),
                    image: joystick_assets.background_image.clone(),
                    ..default()
                },
                Node {
                    position_type: PositionType::Absolute,
                    width: px(JOYSTICK_BACKGROUND_SIZE.x),
                    height: px(JOYSTICK_BACKGROUND_SIZE.y),
                    ..default()
                },
                ZIndex(0),
            ),
            (
                VirtualJoystickUIKnob,
                ImageNode {
                    color: JOYSTICK_IMAGE.into(),
                    image: joystick_assets.knob_image.clone(),
                    ..default()
                },
                Node {
                    position_type: PositionType::Absolute,
                    width: px(JOYSTICK_KNOB_SIZE.x),
                    height: px(JOYSTICK_KNOB_SIZE.y),
                    ..default()
                },
                ZIndex(1),
            ),
        ],
    ));
}
