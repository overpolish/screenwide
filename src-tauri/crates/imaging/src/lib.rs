// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Captured pictures and what is read off their pixels: the still every
//! feature passes around, the colour round a region, and the edges and boxes
//! the ruler and annotation snapping find.

pub mod analysis;
pub mod surface_colour;

/// A captured still: straight (non-premultiplied) RGBA8, packed rows, top down.
/// That is what both the clipboard and the PNG encoder want.
#[derive(Clone)]
pub struct CapturedImage {
  pub rgba: Vec<u8>,
  pub width: u32,
  pub height: u32,
}
