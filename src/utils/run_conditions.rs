/*
 * File: run_conditions.rs
 * Author: Leopold Johannes Meinel (leo@meinel.dev)
 * -----
 * Copyright (c) 2026 Leopold Johannes Meinel & contributors
 * SPDX ID: Apache-2.0
 * URL: https://www.apache.org/licenses/LICENSE-2.0
 */

use bevy::{prelude::*, window::WindowFocused};

/// Run condition that is active if any [`WindowFocused::focused`] has been sent with false.
///
/// This indicates that any window has lost focus.
pub(crate) fn window_unfocused(mut reader: MessageReader<WindowFocused>) -> bool {
    reader.read().any(|w| !w.focused)
}
