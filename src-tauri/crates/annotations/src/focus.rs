// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The focus: a box left sharp while everything around it softens.
//!
//! A focus is not a mark on the picture but a hole in a blur over it. The
//! blur is applied to the source the way a spotlight's is, as one extra pass
//! over the whole picture, so the crop and the magnifier see it too; what
//! lies outside the box softens by its distance from the box's edge and, on
//! a screen turned in depth, by how far past the box the screen sits. The
//! cursor, the camera and every mark stay sharp: they are drawn over the
//! softened source.
//!
//! The box is held and edited exactly as a redaction's or a spotlight's is,
//! through [`super::box_gesture`].

/// Draw-ready geometry and the distance that picks a box, prepared for both
/// backends.
#[cfg(any(target_os = "macos", target_os = "windows", test))]
pub mod geometry;
/// The grips the native chrome draws.
#[cfg(any(target_os = "macos", target_os = "windows"))]
pub(crate) mod handles;
/// Making one, and the dress a fresh one wears.
pub mod model;
/// The retained draw record, and the strength the blur carries.
#[cfg(any(target_os = "macos", target_os = "windows", test))]
pub mod native;

#[cfg(test)]
mod tests;
