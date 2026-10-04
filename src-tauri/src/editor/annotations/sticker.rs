// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The sticker annotation: a picture laid over the source, turned, sized and
//! mirrored as a whole.
//!
//! Its picture is named by `asset` and drawn by the compositor out of the
//! sticker atlas; everything here is where the picture sits. [`super::shape`]
//! is the list of what a kind has to answer, and each of the sticker's
//! answers is one function in this module.

/// Draw-ready geometry: the compositor calls this directly, the macOS chrome
/// through `geometry.h`.
#[cfg(any(target_os = "macos", target_os = "windows", test))]
pub(crate) mod geometry;
/// What moving a sticker's grips does to it.
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

pub(crate) use model::StickerArt;
pub(crate) use play::StickerPlay;

#[cfg(test)]
mod tests;
