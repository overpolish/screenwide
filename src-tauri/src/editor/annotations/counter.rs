// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The counter annotation: a numbered disc with a pin's curved tail.
//!
//! Everything that is a counter's own - how one is numbered and aimed, what
//! its tail does, how it is packed for the native side, how it is drawn and
//! picked - lives here. [`super::shape`] is the list of what a kind has to
//! answer, and every answer of the counter's is one function in this module.

/// Where the numbers sit in the atlas both backends rasterise them into.
#[cfg(any(target_os = "macos", target_os = "windows", test))]
pub(crate) mod atlas;
/// The atlas layout as the Metal compositor reaches it.
#[cfg(target_os = "macos")]
pub(crate) mod atlas_ffi;
/// Draw-ready geometry, prepared for both backends: the D3D11 one calls this
/// directly, the Metal one through `geometry.h`.
#[cfg(any(target_os = "macos", target_os = "windows", test))]
pub(crate) mod geometry;
/// What moving a counter's grips does to it.
pub(crate) mod gesture;
/// The grips the native chrome draws.
#[cfg(any(target_os = "macos", target_os = "windows"))]
pub(crate) mod handles;
/// Making one, and the dress a fresh one wears.
pub(crate) mod model;
/// The retained draw record.
#[cfg(any(target_os = "macos", target_os = "windows"))]
pub(crate) mod native;
/// How a counter arrives and leaves: it grows into place rather than being
/// drawn along a path.
pub(crate) mod reveal;
/// The shape a counter is drawn and picked as, and where its tail ends.
pub(crate) mod silhouette;
/// Where a fresh counter lands, and what a counter offers the snap engine.
pub(crate) mod snap;

pub(crate) use model::{new_counter, next_counter_value};
