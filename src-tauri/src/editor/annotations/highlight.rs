// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The highlight: a marker drawn over lines of text.
//!
//! A highlight is a selection. Pressed at one point of the picture and let go
//! at another, it covers the lines of text between the two the way selecting
//! text does: the first line from the press to its end, every line between in
//! full, and the last line from its start to where the hand let go. The lines
//! are found in the picture's own pixels when the selection is made and kept in
//! the document as bands, so a preview, an export and a document opened later
//! all draw the same highlight without reading the picture again.
//!
//! It is not painted over the picture with opacity, which vanishes on a dark
//! page. Each covered pixel is read and recoloured: the page's surface becomes
//! the highlight's colour, and whatever stands out from the surface - the text -
//! becomes ink that reads on it. The surface and ink brightness the recolouring
//! measures against are read with the lines and kept beside them. Recolouring
//! guesses which pixels are page and which are ink, which mixed content can
//! defeat, so a highlight can tint instead: each pixel is laid under the
//! colour the way a felt marker's ink is, whatever it is.
//!
//! Where the picture cannot be read as text, a highlight can be laid by hand
//! instead: the drag spans a box, and the marker goes over it in strokes of
//! its own width, the way a formula or a picture on a page is marked.

/// Finding the lines under a selection, and the page they are printed on.
#[cfg(any(target_os = "macos", target_os = "windows", test))]
pub(crate) mod detect;
/// The prepared record both compositors draw from, and the distance that
/// picks a highlight from its grips' record.
#[cfg(any(target_os = "macos", target_os = "windows", test))]
pub(crate) mod geometry;
/// What moving a highlight's grips does to it.
#[cfg(any(target_os = "macos", target_os = "windows", test))]
pub(crate) mod gesture;
/// The grips the native chrome draws.
#[cfg(any(target_os = "macos", target_os = "windows"))]
pub(crate) mod handles;
/// Laying one by hand over a box.
#[cfg(any(target_os = "macos", target_os = "windows", test))]
pub(crate) mod manual;
/// Making one, and what it is made of.
pub(crate) mod model;
/// The retained draw record: its bands, its seed and its dress.
#[cfg(any(target_os = "macos", target_os = "windows"))]
pub(crate) mod native;
/// The pixels a selection reads, held for the length of a gesture.
#[cfg(any(target_os = "macos", target_os = "windows", test))]
pub(crate) mod picture;

#[cfg(test)]
mod tests;
