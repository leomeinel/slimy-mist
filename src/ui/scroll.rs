/*
 * File: scroll.rs
 * Author: Leopold Johannes Meinel (leo@meinel.dev)
 * -----
 * Copyright (c) 2026 Leopold Johannes Meinel & contributors
 * SPDX ID: Apache-2.0
 * URL: https://www.apache.org/licenses/LICENSE-2.0
 * -----
 * Heavily inspired by: https://github.com/bevyengine/bevy/blob/main/examples/ui/scroll_and_overflow/scroll.rs
 */

use bevy::{prelude::*, ui::UiSystems};

pub(super) fn plugin(app: &mut App) {
    // Note: We are running this in `FixedUpdate` to ensure consistent scrolling.
    app.add_systems(FixedUpdate, auto_scroll_hovered);

    app.add_systems(PostUpdate, reset_auto_scroll_nodes.after(UiSystems::Layout));

    app.add_observer(on_scroll);
}

/// UI scrolling event.
#[derive(EntityEvent, Debug)]
#[entity_event(propagate, auto_propagate)]
struct Scroll {
    entity: Entity,
    /// Scroll delta in logical coordinates.
    delta: Vec2,
}

/// Marker [`Component`] for auto scrolling.
#[derive(Component)]
pub(crate) struct AutoScroll(pub(crate) Vec2);

/// Trigger [`Scroll`] automatically for hovered [`Entity`]s with [`AutoScroll`].
fn auto_scroll_hovered(query: Query<(Entity, &Node, &AutoScroll)>, mut commands: Commands) {
    for (entity, node, scroll) in query {
        // Continue if content is centered
        if node.justify_content == JustifyContent::Center && node.align_items == AlignItems::Center
        {
            continue;
        }

        commands.trigger(Scroll {
            entity,
            delta: scroll.0,
        });
    }
}

/// Reset [`Node`]s with [`AutoScroll`].
///
/// This sets [`Node::justify_content`] and [`Node::align_items`] according to if there is any overflow.
fn reset_auto_scroll_nodes(query: Query<(&mut Node, &ComputedNode), With<AutoScroll>>) {
    for (mut node, computed) in query {
        let delta = computed.content_size() - computed.size();

        if node.overflow.x == OverflowAxis::Scroll {
            if delta.x < 0. {
                node.align_items = AlignItems::Center;
            } else {
                node.align_items = AlignItems::FlexStart;
            }
        }

        if node.overflow.y == OverflowAxis::Scroll {
            if delta.y < 0. {
                node.justify_content = JustifyContent::Center;
            } else {
                node.justify_content = JustifyContent::FlexStart;
            }
        }
    }
}

/// Scroll [`Entity`] from [`Scroll`].
fn on_scroll(
    mut scroll: On<Scroll>,
    mut query: Query<(&mut ScrollPosition, &Node, &ComputedNode)>,
) {
    let Ok((mut scroll_position, node, computed)) = query.get_mut(scroll.entity) else {
        return;
    };

    let max_offset = (computed.content_size() - computed.size()) * computed.inverse_scale_factor();

    let delta = &mut scroll.delta;
    if node.overflow.x == OverflowAxis::Scroll && delta.x != 0. {
        // Is this node already scrolled all the way in the direction of the scroll?
        let max = if delta.x > 0. {
            scroll_position.x >= max_offset.x
        } else {
            scroll_position.x <= 0.
        };

        if !max {
            scroll_position.x += delta.x;
            // Consume the X portion of the scroll delta.
            delta.x = 0.;
        }
    }

    if node.overflow.y == OverflowAxis::Scroll && delta.y != 0. {
        // Is this node already scrolled all the way in the direction of the scroll?
        let max = if delta.y > 0. {
            scroll_position.y >= max_offset.y
        } else {
            scroll_position.y <= 0.
        };

        if !max {
            scroll_position.y += delta.y;
            // Consume the Y portion of the scroll delta.
            delta.y = 0.;
        }
    }

    // Stop propagating when the delta is fully consumed.
    if *delta == Vec2::ZERO {
        scroll.propagate(false);
    }
}
