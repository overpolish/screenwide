// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The shape annotation: an outline round a box, drawn with a round pen.
//!
//! One form covers every shape: the box's corners are rounded by the style's
//! radius, so a square rounded all the way is a circle and a longer box a
//! pill. The box is held and edited exactly as a redaction's is, through
//! [`super::box_gesture`]; everything else that is a shape's own lives here.

/// Draw-ready geometry, prepared for both backends: the D3D11 one calls this
/// directly, the Metal one through `geometry.h`.
#[cfg(any(target_os = "macos", target_os = "windows", test))]
pub(crate) mod geometry;
/// The grips the native chrome draws.
#[cfg(any(target_os = "macos", target_os = "windows"))]
pub(crate) mod handles;
/// Making one, and the dress a fresh one wears.
pub(crate) mod model;
/// The retained draw record.
#[cfg(any(target_os = "macos", target_os = "windows"))]
pub(crate) mod native;
/// How a hand-drawn stroke strays from its outline.
#[cfg(any(target_os = "macos", target_os = "windows", test))]
mod wander;
