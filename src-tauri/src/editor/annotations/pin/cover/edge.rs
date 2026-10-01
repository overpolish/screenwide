// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

/// How far an edge may wander, in tracking pixels, and still be the same
/// edge.
const STEADY: usize = 2;
/// How many frames an edge must be found on, and how far the content must
/// have moved meanwhile in tracking pixels, one way, before it is taken as a
/// cover's. A cover stays put while content moves under it; the edge of
/// something moving in front of a still background, a person before a wall,
/// moves with it, or only sways about.
const MIN_FRAMES: u32 = 3;
const MIN_TRAVEL: f32 = 12.0;

/// A cover's edge on one side of the content, as found frame after frame.
#[derive(Clone, Copy, Default)]
pub(super) struct Edge {
  /// Where it was first found, in lines.
  at: Option<usize>,
  frames: u32,
  /// How far the content has moved along the axis since, either way.
  travelled: f32,
}

impl Edge {
  /// Takes one voting frame's finding, `found`, on which the content moved
  /// `shift` along the axis. Not found, or found elsewhere, it starts over.
  pub(super) fn update(&mut self, found: Option<usize>, shift: f32) {
    match (self.at, found) {
      (Some(at), Some(found)) if at.abs_diff(found) <= STEADY => {
        self.frames += 1;
        self.travelled += shift;
      }
      _ => {
        *self = Self {
          at: found,
          frames: 1,
          travelled: shift,
        }
      }
    }
  }

  /// Where the edge is, once it has held still long enough.
  pub(super) fn proven(&self) -> Option<f32> {
    let proven = self.frames >= MIN_FRAMES && self.travelled.abs() >= MIN_TRAVEL;
    self.at.filter(|_| proven).map(|at| at as f32)
  }
}
