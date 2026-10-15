// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! RVM's matte decided. RVM is right about where the person is far more
//! often than about how see-through each pixel is, so every part of the
//! frame is decided as the person or the room, and only the line between
//! them is soft, drawn again at one steady softness:
//!
//! - Anything RVM gives half or more is the person, apart from specks of it
//!   floating in the room.
//! - Every other area is the room if it reaches the top or either side of
//!   the frame, or if RVM is sure of most of it and it is bigger than a
//!   pinhole. Anything else is a pocket inside the person, and the person.
//! - A bite out of the person's outline that RVM was unsure of, coloured
//!   like the person beside it and not like the room around it, is the
//!   person: where RVM lost a patch against something like a sofa.
//! - The outline is evened out, and the edge drawn along it fading over a
//!   steady width, handed over smoothly to the decided value away from it.
//!
//! The areas are decided on a half-size copy (`areas`) and the edge drawn at
//! full size near the outline alone (`edge`), about a third the work of
//! doing both on the whole frame at full size.

mod areas;
mod edge;
mod fields;

use areas::Areas;
use edge::Edge;

/// Sizes in pixels of a 1080-line matte, scaled to the matte's own.
const BAND: f32 = 8.0;
const EDGE: f32 = 8.0;
const OUTLINE: f32 = 5.0;
const SOFTEN: f32 = 2.0;
const FARTHEST: f32 = 24.0;
const NOTCH: f32 = 10.0;
const ROOM_SPREAD: f32 = 20.0;
const PERSON_SPREAD: f32 = 8.0;
/// The largest share of the frame a pinhole or a speck takes.
const TINY: f64 = 0.0005;
/// Alpha RVM is sure of, either way.
const CLEAR: f32 = 0.1;
const SOLID: f32 = 0.9;
const UNSURE: (f32, f32) = (0.03, 0.97);
/// How far a bite's colour must be from the room's, and how much nearer the
/// person's, to be taken as the person, in RGB from 0 to 1.
const UNLIKE_ROOM: f32 = 0.12;
const LIKE_PERSON: f32 = 0.5;

/// Buffers kept from frame to frame, so a frame allocates nothing.
pub(super) struct Cleanup {
  size: (usize, usize),
  areas: Areas,
  edge: Edge,
}

impl Cleanup {
  pub(super) fn new(size: (usize, usize)) -> Self {
    Self {
      size,
      areas: Areas::new(size),
      edge: Edge::new(size),
    }
  }

  /// `alpha`, RVM's matte of the 8-bit RGB frame `rgb`, decided in place.
  pub(super) fn apply(&mut self, alpha: &mut [u8], rgb: &[u8]) {
    self.areas.decide(alpha, rgb, self.size);
    self.edge.draw(&self.areas, alpha);
  }
}

#[cfg(test)]
mod tests;
