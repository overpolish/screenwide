// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The magnifier: one lens of clear glass that enlarges what lies directly
//! under it, about its own middle.
//!
//! The lens shows the finished picture under it - the layers as they are
//! composed, cropped, inset, rounded and turned, and the annotations beneath
//! it in their stacking order - so nothing cropped away or hidden shows
//! larger. Enlarged pixels stay pixels: each texel is drawn as a crisp square
//! with only its edges smoothed, since a smoothly scaled capture reads as a
//! blurred one.
//!
//! The lens is drawn out, held, moved and resized as a shape's box is,
//! through [`super::box_gesture`]; its zoom is one of a few looks, so making
//! it bigger shows more at the same zoom. What lies under it is edited
//! through it: [`lens`] reads a press inside it back to what it shows there.

/// The C entry point the macOS chrome prepares a lens through.
#[cfg(target_os = "macos")]
mod ffi;
/// Draw-ready geometry and the distance that picks it, for both backends.
#[cfg(any(target_os = "macos", target_os = "windows", test))]
pub mod geometry;
/// The grips the native chrome draws.
#[cfg(any(target_os = "macos", target_os = "windows"))]
pub(crate) mod handles;
/// Seeing and pressing through a lens.
pub mod lens;
/// Making one, the dress a fresh one wears, and reading an old one.
pub mod model;
/// The retained draw record.
#[cfg(any(target_os = "macos", target_os = "windows"))]
pub(crate) mod native;
/// How a lens settles onto its clip, and leaves it again.
#[cfg(any(target_os = "macos", target_os = "windows", test))]
pub(crate) mod reveal;

#[cfg(test)]
mod tests;
