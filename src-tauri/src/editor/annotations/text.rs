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

/// One box being typed into, from the press that opens it to the commit.
#[cfg(any(target_os = "macos", target_os = "windows", test))]
pub(crate) mod edit;
/// Draw-ready geometry and the distance that picks it, prepared for both
/// backends: the D3D11 one calls this directly, the Metal one through
/// `geometry.h`.
#[cfg(any(target_os = "macos", target_os = "windows", test))]
pub(crate) mod geometry;
/// What moving a text box's grips does to it.
#[cfg(any(target_os = "macos", target_os = "windows", test))]
pub(crate) mod gesture;
/// The grips the native chrome draws.
#[cfg(any(target_os = "macos", target_os = "windows"))]
pub(crate) mod handles;
/// How much room a block of text takes, asked of the platform's text engine.
#[cfg(any(target_os = "macos", target_os = "windows", test))]
pub(crate) mod metrics;
/// Making one, and the dress a fresh one wears.
pub(crate) mod model;
/// The retained draw record.
#[cfg(any(target_os = "macos", target_os = "windows"))]
pub(crate) mod native;
/// How a text box arrives and leaves: the box first, then its pointer.
#[cfg(any(target_os = "macos", target_os = "windows", test))]
pub(crate) mod reveal;
/// Where a fresh text box lands, and what one offers the snap engine.
#[cfg(any(target_os = "macos", target_os = "windows", test))]
pub(crate) mod snap;
/// Typing where the platform has no text view to lay over the box: the
/// text, the selection and every edit a key makes.
#[cfg(any(target_os = "windows", test))]
pub(crate) mod typing;

#[cfg(test)]
mod tests;

#[cfg(any(target_os = "macos", target_os = "windows", test))]
pub(crate) use model::new_text;
