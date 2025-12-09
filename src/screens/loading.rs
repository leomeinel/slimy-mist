/*
 * File: loading.rs
 * Author: Leopold Johannes Meinel (leo@meinel.dev)
 * -----
 * Copyright (c) 2025 Leopold Johannes Meinel & contributors
 * SPDX ID: Apache-2.0
 * URL: https://www.apache.org/licenses/LICENSE-2.0
 */

//! A loading screen during which game assets are loaded if necessary.
//! This reduces stuttering, especially for audio on Wasm.

// FIXME: Previous solution is currently unsupoorted, after it is add loading and gameplay states here.
// See: https://github.com/NiklasEi/bevy_asset_loader/pull/259
// After it is, we should implement this: https://github.com/NiklasEi/bevy_asset_loader/blob/main/bevy_asset_loader/examples/progress_tracking.rs
