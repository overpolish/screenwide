// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The arrow annotation: a quadratic Bézier with a head on one or both ends.
//!
//! Everything that is an arrow's own - how one is made, what its grips do,
//! how it is packed for the native side, how it is drawn and picked - lives
//! here. [`super::shape`] is the list of what a kind has to answer, and every
//! answer of the arrow's is one function in this module.

/// The curve itself, and the one number the bend is held as.
pub(crate) mod bend;
/// How far a point falls from a prepared arrow, which is how a press picks
/// one.
#[cfg(any(target_os = "windows", test))]
pub(crate) mod distance;
/// Draw-ready geometry for the D3D11 backend; the Metal one prepares the same
/// numbers through `geometry.h`.
#[cfg(any(target_os = "windows", test))]
pub(crate) mod geometry;
/// What moving one of an arrow's grips does to its shape.
pub(crate) mod gesture;
/// The grips the native chrome draws.
#[cfg(any(target_os = "macos", target_os = "windows"))]
pub(crate) mod handles;
/// Making one, and the dress a fresh one wears.
pub(crate) mod model;
/// The retained draw record.
#[cfg(any(target_os = "macos", target_os = "windows"))]
pub(crate) mod native;
/// What a fresh arrow does while it is being drawn out, and what it offers
/// the snap engine.
pub(crate) mod snap;

pub(crate) use model::new_arrow;
