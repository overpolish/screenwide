// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The image annotation: a picture laid over the source, turned, sized and
//! mirrored as a whole.
//!
//! Its picture is named by `asset` and drawn by the compositor out of the
//! image atlas; everything here is where the picture sits. [`super::shape`]
//! is the list of what a kind has to answer, and each of the image's
//! answers is one function in this module.

/// Its own selection frame: the box along its turned sides, its grips and
/// their cursors, for both platforms' chrome.
#[cfg(any(target_os = "macos", target_os = "windows", test))]
pub(crate) mod frame;
/// Draw-ready geometry: the compositor calls this directly, the macOS chrome
/// through `geometry.h`.
#[cfg(any(target_os = "macos", target_os = "windows", test))]
pub(crate) mod geometry;
/// What moving an image's grips does to it.
#[cfg(any(target_os = "macos", target_os = "windows", test))]
pub(crate) mod gesture;
/// The grips the native chrome draws.
#[cfg(any(target_os = "macos", target_os = "windows"))]
pub(crate) mod handles;
/// Making one, and where it reaches.
pub(crate) mod model;
/// The retained draw record.
#[cfg(any(target_os = "macos", target_os = "windows"))]
pub(crate) mod native;
/// How an animated picture plays, and which frame a moment shows.
pub(crate) mod play;

pub(crate) use model::ImageArt;
pub(crate) use play::ImagePlay;

#[cfg(test)]
mod tests;
