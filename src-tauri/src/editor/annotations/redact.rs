// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The redaction: a box that hides what is under it.
//!
//! A redaction is not painted over the canvas the way the other kinds are. It
//! replaces the source pixels it covers before anything reads them, so the
//! crop, the magnifier and the export all see the covered pixels already
//! gone. Erase and colour read no covered pixel: erase takes its colour from
//! the ring just outside the box, colour takes the style's. Pixelate reads the
//! covered pixels only for the few colours they are made of; its blocks are
//! laid out by a seed rather than averaged from the picture, which leaves a
//! depixelation attack no shapes to recover. Blur is a Gaussian over the box's
//! own pixels, a look rather than a way to hide. The box is snapped outward
//! to whole source pixels, and a rounded corner is drawn wholly inside the
//! curve with its soft edge outside it, so no covered pixel survives
//! half-blended.

/// How large a classically pixelated box's cells are, and how wide a blurred
/// box's blur.
#[cfg(any(target_os = "macos", target_os = "windows", test))]
pub(crate) mod cells;
/// Draw-ready geometry and the distance that picks it, prepared for both
/// backends. The record never draws: its width stays zero.
#[cfg(any(target_os = "macos", target_os = "windows", test))]
pub(crate) mod geometry;
/// The grips the native chrome draws.
#[cfg(any(target_os = "macos", target_os = "windows"))]
pub(crate) mod handles;
/// What a recording's redaction reads once from its clip's first frame.
pub(crate) mod held;
/// Making one, and the dress a fresh one wears.
pub(crate) mod model;
/// The retained draw record: the box, its fill and its pixelation.
#[cfg(any(target_os = "macos", target_os = "windows", test))]
pub(crate) mod native;
/// The few colours a pixelated box draws its blocks in.
#[cfg(any(target_os = "macos", target_os = "windows", test))]
pub(crate) mod palette;
/// The records the Windows compositor's passes read.
#[cfg(any(target_os = "windows", test))]
pub(crate) mod records;
/// The ramp an animated redaction arrives through.
#[cfg(any(target_os = "macos", target_os = "windows", test))]
pub(crate) mod reveal;
/// Where a dragged side lands, and what a redaction offers the snap engine.
#[cfg(any(target_os = "macos", target_os = "windows", test))]
pub(crate) mod snap;
/// The surface an erased box on a recording follows over its clip.
#[cfg(any(target_os = "macos", target_os = "windows", test))]
pub(crate) mod surface_timeline;

#[cfg(test)]
mod tests;

#[cfg(any(target_os = "macos", target_os = "windows", test))]
pub(crate) use model::new_redact;
