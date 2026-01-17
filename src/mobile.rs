/*
 * File: mobile.rs
 * Author: Leopold Johannes Meinel (leo@meinel.dev)
 * -----
 * Copyright (c) 2026 Leopold Johannes Meinel & contributors
 * SPDX ID: Apache-2.0
 * URL: https://www.apache.org/licenses/LICENSE-2.0
 * -----
 * Heavily inspired by: https://github.com/bevyengine/bevy/tree/main/examples/mobile
 */

#[cfg(target_os = "android")]
mod android;

use bevy::{prelude::*, winit::WinitSettings};
use bevy_asset_loader::asset_collection::AssetCollection;
use virtual_joystick::{
    JoystickFixed, NoAction, VirtualJoystickBundle, VirtualJoystickInteractionArea,
    VirtualJoystickNode, VirtualJoystickPlugin, VirtualJoystickUIBackground, VirtualJoystickUIKnob,
};

use crate::screens::Screen;

pub(super) fn plugin(app: &mut App) {
    // Add library plugins
    app.add_plugins(VirtualJoystickPlugin::<VirtualJoystick>::default());

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

/// Virtual joystick type that is used as [`virtual_joystick::VirtualJoystickID`]
#[derive(Default, Debug, Reflect, Hash, Clone, PartialEq, Eq)]
pub(crate) enum VirtualJoystick {
    #[default]
    Movement,
}

/// Color of the joystick knob
const JOYSTICK_KNOB_COLOR: Color = Color::WHITE;
/// Size of the joystick knob in pixels
const JOYSTICK_KNOB_SIZE: Vec2 = Vec2::new(75., 75.);

/// Color of the joystick background
const JOYSTICK_BACKGROUND_COLOR: Color = Color::WHITE;
/// Size of the joystick background in pixels
const JOYSTICK_BACKGROUND_SIZE: Vec2 = Vec2::new(150., 150.);

/// Spawn default joystick of type [`VirtualJoystick::Movement`]
pub(crate) fn spawn_joystick(mut commands: Commands, joystick_assets: Res<JoystickAssets>) {
    let style = Node {
        position_type: PositionType::Absolute,
        width: Val::Px(JOYSTICK_BACKGROUND_SIZE.x),
        height: Val::Px(JOYSTICK_BACKGROUND_SIZE.y),
        left: Val::VMin(5.),
        bottom: Val::VMin(5.),
        ..default()
    };
    commands.spawn((
        VirtualJoystickBundle::new(
            VirtualJoystickNode::default()
                .with_id(VirtualJoystick::default())
                .with_behavior(JoystickFixed)
                .with_action(NoAction),
        )
        .set_style(style),
        DespawnOnExit(Screen::Gameplay),
        children![
            (
                VirtualJoystickInteractionArea,
                Node {
                    width: Val::Percent(100.),
                    height: Val::Percent(100.),
                    ..default()
                },
            ),
            (
                VirtualJoystickUIBackground,
                ImageNode {
                    color: JOYSTICK_BACKGROUND_COLOR,
                    image: joystick_assets.background_image.clone(),
                    ..default()
                },
                Node {
                    position_type: PositionType::Absolute,
                    width: Val::Px(JOYSTICK_BACKGROUND_SIZE.x),
                    height: Val::Px(JOYSTICK_BACKGROUND_SIZE.y),
                    ..default()
                },
                ZIndex(0),
            ),
            (
                VirtualJoystickUIKnob,
                ImageNode {
                    color: JOYSTICK_KNOB_COLOR,
                    image: joystick_assets.knob_image.clone(),
                    ..default()
                },
                Node {
                    position_type: PositionType::Absolute,
                    width: Val::Px(JOYSTICK_KNOB_SIZE.x),
                    height: Val::Px(JOYSTICK_KNOB_SIZE.y),
                    ..default()
                },
                ZIndex(1),
            ),
        ],
    ));
}
