// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The text annotation: a solid box around lines of type, with an optional
//! pointer.
//!
//! The box is never sized by hand. Its size follows from the text and the
//! type size alone, so the only things a hand moves are the box and the tip
//! of its pointer. Everything that is a text box's own lives here;
//! [`super::shape`] is the list of what a kind has to answer, and every answer
//! of the text box's is one function in this module.

/// The families the caption templates bring, read by the text engines.
pub mod app_fonts;
/// One box being typed into, from the press that opens it to the commit.
#[cfg(any(target_os = "macos", target_os = "windows", test))]
pub mod edit;
/// Which face type is set in: Inter for annotations, any family for
/// captions.
pub mod face;
/// Draw-ready geometry and the distance that picks it: the compositor calls
/// this directly, the macOS chrome through `geometry.h`.
#[cfg(any(target_os = "macos", target_os = "windows", test))]
pub mod geometry;
/// What moving a text box's grips does to it.
#[cfg(any(target_os = "macos", target_os = "windows", test))]
pub(crate) mod gesture;
/// The grips the native chrome draws.
#[cfg(any(target_os = "macos", target_os = "windows"))]
pub(crate) mod handles;
/// The font families installed on the system.
pub mod installed;
/// How much room a block of text takes, asked of the platform's text engine.
#[cfg(any(target_os = "macos", target_os = "windows", test))]
pub mod metrics;
/// Making one, and the dress a fresh one wears.
pub mod model;
/// The retained draw record.
#[cfg(any(target_os = "macos", target_os = "windows"))]
pub(crate) mod native;
/// How a text box arrives and leaves: the box first, then its pointer.
#[cfg(any(target_os = "macos", target_os = "windows", test))]
pub(crate) mod reveal;
/// Where a fresh text box lands, and what one offers the snap engine.
#[cfg(any(target_os = "macos", target_os = "windows", test))]
pub(crate) mod snap;
/// Inter set by DirectWrite on Windows: the face annotation type is measured
/// and rasterised with there. macOS sets it with Core Text, in `macos/`.
#[cfg(target_os = "windows")]
pub mod type_device;
/// Typing where the platform has no text view to lay over the box: the
/// text, the selection and every edit a key makes. macOS types into a native
/// text view and reads only the marks the shader draws.
#[cfg(any(target_os = "macos", target_os = "windows", test))]
#[cfg_attr(target_os = "macos", allow(dead_code, unused_imports))]
pub mod typing;

#[cfg(test)]
mod tests;

#[cfg(any(target_os = "macos", target_os = "windows", test))]
pub use model::new_text;
