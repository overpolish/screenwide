// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The text annotation: a solid box around lines of type, with an optional
//! pointer.
//!
//! The box is never sized by hand. Its size follows from the text and the
//! type size alone, so the only things a hand moves are the box and the tip
//! of its pointer. Everything that is a text box's own lives here, set by
//! `screenwide_text`'s engine; [`super::shape`] is the list of what a kind has
//! to answer, and every answer of the text box's is one function in this
//! module.

/// One box being typed into, from the press that opens it to the commit.
#[cfg(any(target_os = "macos", target_os = "windows", test))]
pub mod edit;
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
/// What a text's look makes of it: face, size, shown text and tail.
pub mod look;
/// Making one, the dress a fresh one wears, and its control points.
pub mod model;
/// The retained draw record, and the look drawn beside it.
#[cfg(any(target_os = "macos", target_os = "windows"))]
pub mod native;
/// How a text box arrives and leaves: the box first, then its pointer.
#[cfg(any(target_os = "macos", target_os = "windows", test))]
pub(crate) mod reveal;
/// Where a fresh text box lands, and what one offers the snap engine.
#[cfg(any(target_os = "macos", target_os = "windows", test))]
pub(crate) mod snap;
/// A box whose look sets chosen words apart, as the typing places the caret
/// and reads presses by.
#[cfg(any(target_os = "macos", target_os = "windows", test))]
pub mod typed;

#[cfg(test)]
mod tests;

#[cfg(any(target_os = "macos", target_os = "windows", test))]
pub use model::new_text;
