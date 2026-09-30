// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The magnifier: a loupe showing a zoom area enlarged.
//!
//! The loupe is the zoom area's own shape scaled up, so it always shows
//! exactly what the zoom area covers, and one line joins the two. The loupe
//! enlarges the picture as the compositor holds it, after the redactions and
//! the spotlights' blur were applied to it, so nothing hidden is shown
//! larger; it reads nothing drawn over the picture. Enlarged pixels stay
//! pixels: each texel is drawn as a crisp square with only its edges
//! smoothed, since a smoothly scaled capture reads as a blurred one.
//!
//! The zoom area is drawn out, held and resized as a spotlight's box is,
//! through [`super::box_gesture`]; the loupe is carried by its own inside and
//! sized by one grip on its rim, and its zoom is how much bigger that makes
//! it than the zoom area.

/// The C entry points the Metal compositor and the macOS chrome prepare and
/// pick a magnifier through.
#[cfg(target_os = "macos")]
mod ffi;
/// Draw-ready geometry and the distances that pick it, for both backends.
#[cfg(any(target_os = "macos", target_os = "windows", test))]
pub(crate) mod geometry;
/// Carrying and sizing the loupe, and pulling out a fresh zoom area.
#[cfg(any(target_os = "macos", target_os = "windows", test))]
pub(crate) mod gesture;
/// The grips the native chrome draws.
#[cfg(any(target_os = "macos", target_os = "windows"))]
pub(crate) mod handles;
/// Making one, and the dress a fresh one wears.
pub(crate) mod model;
/// The retained draw record.
#[cfg(any(target_os = "macos", target_os = "windows"))]
pub(crate) mod native;
/// How a loupe comes out of its zoom area over its clip, and goes back in.
#[cfg(any(target_os = "macos", target_os = "windows", test))]
pub(crate) mod reveal;

#[cfg(test)]
mod tests;
