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
/// The slight turn and drift a recording can give it, worked out per frame.
#[cfg(any(target_os = "macos", target_os = "windows", test))]
pub(crate) mod sway;

pub(crate) use model::ImageArt;
pub(crate) use play::ImagePlay;

/// Sets how far into its clip `annotation` is drawn, where it is an image
/// whose picture moves or that sways; anything else is left as it is.
pub(crate) fn clocked(annotation: &mut super::Annotation, clock_ms: f64) {
  if let super::AnnotationShape::Image {
    play,
    sway,
    clock_ms: clock,
    ..
  } = &mut annotation.shape
  {
    if play.is_some() || sway.is_some() {
      *clock = Some(clock_ms);
    }
  }
}

#[cfg(test)]
mod tests;
