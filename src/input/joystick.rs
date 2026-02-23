/*
 * File: joystick.rs
 * Author: Leopold Johannes Meinel (leo@meinel.dev)
 * -----
 * Copyright (c) 2026 Leopold Johannes Meinel & contributors
 * SPDX ID: Apache-2.0
 * URL: https://www.apache.org/licenses/LICENSE-2.0
 */

use bevy::{platform::collections::HashMap, prelude::*, window::WindowResized};
use bevy_asset_loader::asset_collection::AssetCollection;
use virtual_joystick::{
    JoystickFixed, NoAction, VirtualJoystickBundle, VirtualJoystickInteractionArea,
    VirtualJoystickNode, VirtualJoystickPlugin, VirtualJoystickUIBackground, VirtualJoystickUIKnob,
};

use crate::{
    screens::{Screen, gameplay::InitGameplaySystems},
    ui::prelude::*,
};

pub(super) fn plugin(app: &mut App) {
    app.init_state::<JoystickState<{ JoystickID::Movement as u8 }>>();

    // Add library plugins
    app.add_plugins(VirtualJoystickPlugin::<u8>::default());

    // Toggle joystick
    app.add_systems(
        OnEnter(Screen::Gameplay),
        spawn_joystick::<{ JoystickID::Movement as u8 }>
            .after(InitGameplaySystems::Finalize)
            .run_if(not(in_state(
                JoystickState::<{ JoystickID::Movement as u8 }>::None,
            ))),
    );
    app.add_systems(
        OnEnter(JoystickState::<{ JoystickID::Movement as u8 }>::Toggled(
            true,
        )),
        spawn_joystick::<{ JoystickID::Movement as u8 }>
            .after(InitGameplaySystems::Finalize)
            .run_if(in_state(Screen::Gameplay)),
    );
    app.add_systems(
        OnEnter(JoystickState::<{ JoystickID::Movement as u8 }>::Toggled(
            false,
        )),
        despawn_joystick::<{ JoystickID::Movement as u8 }>
            .after(InitGameplaySystems::Finalize)
            .run_if(in_state(Screen::Gameplay)),
    );
    // Reset `JoystickRect`
    app.add_systems(
        OnEnter(JoystickState::<{ JoystickID::Movement as u8 }>::None),
        reset_joystick_rect::<{ JoystickID::Movement as u8 }>
            .after(InitGameplaySystems::Finalize)
            .run_if(in_state(Screen::Gameplay)),
    );
    app.add_systems(
        OnExit(Screen::Gameplay),
        reset_joystick_rect::<{ JoystickID::Movement as u8 }>.after(InitGameplaySystems::Finalize),
    );

    // Update `JoystickRect`
    app.add_systems(
        PostUpdate,
        update_joystick_rect::<{ JoystickID::Movement as u8 }>
            .after(InitGameplaySystems::Finalize)
            .after(TransformSystems::Propagate)
            .run_if(
                in_state(Screen::Gameplay)
                    .and(in_state(
                        JoystickState::<{ JoystickID::Movement as u8 }>::Spawned,
                    ))
                    .and(
                        state_changed::<Screen>
                            .or(state_changed::<JoystickState<{ JoystickID::Movement as u8 }>>)
                            .or(on_message::<WindowResized>),
                    ),
            ),
    );
}

/// Tracks the current state of the joystick.
///
/// ## Traits
///
/// - `const ID` represents [`VirtualJoystickNode::id`].
#[derive(States, Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub(crate) enum JoystickState<const ID: u8> {
    None,
    Toggled(bool),
    Spawned,
}
impl<const ID: u8> Default for JoystickState<ID> {
    #[cfg(not(any(target_os = "android", target_os = "ios")))]
    fn default() -> Self {
        Self::None
    }
    #[cfg(any(target_os = "android", target_os = "ios"))]
    fn default() -> Self {
        Self::Toggled(true)
    }
}
impl<const ID: u8> JoystickState<ID> {
    pub(crate) fn is_active(&self) -> bool {
        match self {
            JoystickState::<ID>::Toggled(val) => *val,
            JoystickState::Spawned => true,
            JoystickState::None => false,
        }
    }
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
///
/// ## Traits
///
/// - `const ID` represents [`VirtualJoystickNode::id`].
#[derive(Resource, Default)]
pub(crate) struct JoystickRect<const ID: u8>(Option<Rect>);
impl<const ID: u8> JoystickRect<ID> {
    pub(crate) fn intersects_with(&self, point: Vec2) -> bool {
        self.0.is_some_and(|rect| rect.contains(point))
    }
}

/// Map of [`JoystickID`]s as [`u8`] mapped to their [`Entity`].
#[derive(Resource, Default)]
pub(crate) struct JoystickMap(pub(crate) HashMap<u8, Entity>);

/// Update [`JoystickRect<ID>`] representing [`VirtualJoystickInteractionArea`].
///
/// ## Traits
///
/// - `const ID` represents [`VirtualJoystickNode::id`].
fn update_joystick_rect<const ID: u8>(
    node_query: Query<(&VirtualJoystickNode<u8>, &Children)>,
    interaction_area_query: Query<
        (&ComputedNode, &UiGlobalTransform),
        With<VirtualJoystickInteractionArea>,
    >,
    mut rect: ResMut<JoystickRect<ID>>,
) {
    if node_query.is_empty() || interaction_area_query.is_empty() {
        return;
    }
    let Some(children) = node_query.iter().find(|(n, _)| n.id == ID).map(|(_, c)| c) else {
        return;
    };

    if let Some((node, transform)) = children
        .iter()
        .find_map(|child| interaction_area_query.get(child).ok())
    {
        let factor = node.inverse_scale_factor;
        let new_rect = Rect::from_center_size(transform.translation * factor, node.size() * factor);
        rect.0 = Some(new_rect);
    }
}

/// Reset [`JoystickRect<ID>`].
///
/// ## Traits
///
/// - `const ID` represents [`VirtualJoystickNode::id`].
fn reset_joystick_rect<const ID: u8>(mut rect: ResMut<JoystickRect<ID>>) {
    rect.0 = None;
}

/// Size of the joystick knob in pixels
const JOYSTICK_KNOB_SIZE: Vec2 = Vec2::splat(50.);
/// Size of the joystick background in pixels
const JOYSTICK_BACKGROUND_SIZE: Vec2 = Vec2::splat(100.);

/// Spawn joystick with `ID`.
///
/// ## Traits
///
/// - `const ID` represents [`VirtualJoystickNode::id`].
fn spawn_joystick<const ID: u8>(
    mut commands: Commands,
    mut joystick_map: ResMut<JoystickMap>,
    mut next_state: ResMut<NextState<JoystickState<ID>>>,
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
    let entity = commands
        .spawn((
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
        ))
        .id();
    joystick_map.0.insert(ID, entity);
    (*next_state).set_if_neq(JoystickState::<ID>::Spawned);
}

/// Despawn joystick with `ID`.
///
/// ## Traits
///
/// - `const ID` represents [`VirtualJoystickNode::id`].
fn despawn_joystick<const ID: u8>(
    mut commands: Commands,
    mut joystick_map: ResMut<JoystickMap>,
    mut next_state: ResMut<NextState<JoystickState<ID>>>,
) {
    let Some(entity) = joystick_map.0.get(&ID) else {
        return;
    };
    commands.entity(*entity).despawn();
    joystick_map.0.remove(&ID);
    (*next_state).set_if_neq(JoystickState::<ID>::None);
}
