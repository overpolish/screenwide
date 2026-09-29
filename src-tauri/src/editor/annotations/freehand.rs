// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The draw tool: a line drawn freehand.
//!
//! A stroke is the list of points the hand passed through, in source pixels,
//! thinned as it is drawn so a slow hand does not store every sample. It is
//! stored as drawn and fitted when it is drawn: [`path`] turns the points into
//! a chain of quadratic Béziers, once in Rust, and every renderer and the
//! chrome read that chain, so the picture, the halo and the pick all follow
//! the one line. A smoothed stroke keeps the same points and is fitted far
//! more loosely, so a wobbly curve comes out clean and can still be edited
//! like any other stroke.
//!
//! The stroke is held by its bounding box: the eight grips of a box scale its
//! points with it, and the body carries it. A fresh stroke held still at its
//! end is taken for the clean annotation it looks like, while the hand is
//! still down.

/// The prepared record, and the distance that picks a stroke.
#[cfg(any(target_os = "macos", target_os = "windows", test))]
pub(crate) mod geometry;
/// Moving a stroke, and drawing one out.
#[cfg(any(target_os = "macos", target_os = "windows", test))]
pub(crate) mod gesture;
/// The grips the native chrome draws, and the fitted line it picks by.
#[cfg(any(target_os = "macos", target_os = "windows"))]
pub(crate) mod handles;
/// Holding a fresh stroke still at its end.
#[cfg(any(target_os = "macos", target_os = "windows", test))]
pub(crate) mod hold;
/// Making one, and the dress a fresh one wears.
pub(crate) mod model;
/// The retained draw record, and the fitted line it carries.
#[cfg(any(target_os = "macos", target_os = "windows"))]
pub(crate) mod native;
/// Fitting a stroke's points to a smooth chain of curves.
#[cfg(any(target_os = "macos", target_os = "windows", test))]
pub(crate) mod path;
/// What a stroke held still at its end is taken for.
#[cfg(any(target_os = "macos", target_os = "windows", test))]
mod recognise;
/// How much of a stroke shows while it draws itself in and out.
#[cfg(any(target_os = "macos", target_os = "windows", test))]
pub(crate) mod reveal;

#[cfg(test)]
mod hold_tests;

#[cfg(test)]
mod tests;
