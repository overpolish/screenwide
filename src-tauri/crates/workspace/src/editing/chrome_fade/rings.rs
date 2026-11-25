// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The gizmo's rings come and go by fading, and move from one layer to the
//! next by gliding there. They are one quad drawn through constants shared
//! by the whole chrome, so they cannot crossfade with themselves as the rest
//! of the chrome does: there is only ever one set of them.

use screenwide_osc::geometry::{Rect, Size};

use super::*;

#[derive(Default)]
pub(super) struct RingFade {
  /// The rings as last drawn, which keep being drawn as they fade out.
  last: Option<turn::Rings>,
  /// The rings as last presented, mid-glide included, which a glide to
  /// another layer's starts from.
  presented: Option<turn::Rings>,
  /// How much of the rings showed when they last came or went, whether they
  /// are coming or going, and since when.
  from: f32,
  shown: bool,
  since: Option<Instant>,
  /// The rings a glide set out from, and when.
  glide: Option<(turn::Rings, Instant)>,
}

impl RingFade {
  pub(super) fn is_settled(&self) -> bool {
    self.since.is_none() && self.glide.is_none()
  }

  /// How much of the rings shows at `now`.
  fn level(&self, now: Instant) -> f32 {
    let to = if self.shown { 1.0 } else { 0.0 };
    match self.since {
      Some(since) => self.from + (to - self.from) * blended(since, now),
      None => to,
    }
  }

  /// The rings to draw now for `rings`, the gizmo's as drawn; `changed`
  /// where the chrome has changed what it is drawn round, and `glides`
  /// where it may glide there, else the rings fade in afresh.
  pub(super) fn ease(
    &mut self,
    rings: Option<turn::Rings>,
    changed: bool,
    glides: bool,
    now: Instant,
  ) -> Option<turn::Rings> {
    if rings.is_some() != self.shown {
      self.from = self.level(now);
      self.shown = rings.is_some();
      self.since = Some(now);
    }
    if changed && rings.is_some() {
      if glides {
        self.glide = self.presented.map(|from| (from, now));
      } else if self.presented.is_some() {
        self.from = 0.0;
        self.since = Some(now);
      }
    }
    let level = self.level(now);
    if self.since.is_some_and(|since| progress(since, now) >= 1.0) {
      self.since = None;
    }
    let drawn = match rings {
      Some(rings) => {
        self.last = Some(rings);
        let glided = match self.glide {
          Some((from, since)) if eased(since, now) < 1.0 => glided(from, rings, eased(since, now)),
          _ => {
            self.glide = None;
            rings
          }
        };
        Some(glided)
      }
      None => {
        self.glide = None;
        self.last.filter(|_| level > 0.0)
      }
    };
    self.presented = rings.and(drawn);
    drawn.map(|mut rings| {
      // Row 7's `.w` is how much of the rings shows, which the crop's own
      // ease has already had its say in.
      rings.rows[7][3] *= level;
      rings
    })
  }
}

/// The rings `t` of the way from `from` to `to`: each ring's direction and
/// the middle they turn about, and the quad round them. Row 7, what is lit
/// and how much shows, is `to`'s.
fn glided(from: turn::Rings, to: turn::Rings, t: f32) -> turn::Rings {
  let mut rings = to;
  for (row, from) in rings.rows.iter_mut().zip(from.rows).take(7) {
    for (value, from) in row.iter_mut().zip(from) {
      *value = from + (*value - from) * t;
    }
  }
  for (value, from) in rings.quad.iter_mut().zip(from.quad) {
    *value = from + (*value - from) * t;
  }
  rings
}

/// Lays `rings` under `vertices`, a chrome `size` device pixels at `scale`.
/// The chrome is drawn in its vertices' order, so the rings go first: the
/// boxes and their grips are drawn over them, as the grips also win a press
/// where the two meet (`turn::grip_over_rings`).
pub(in crate::editing) fn add_rings(
  vertices: &mut Vec<Vertex>,
  constants: &mut RenderConstants,
  size: (u32, u32),
  scale: f64,
  rings: turn::Rings,
) {
  let scale = scale.max(0.1);
  let view = Size {
    width: f64::from(size.0) / scale,
    height: f64::from(size.1) / scale,
  };
  let [x, y, width, height] = rings.quad.map(|value| f64::from(value) / scale);
  constants.turn_rings = rings.rows;
  let mut quad = Vec::with_capacity(6);
  osc_gpu::add_quad(
    &mut quad,
    view,
    Rect::from_xywh(x, y, width, height),
    turn::RING_KIND,
  );
  vertices.splice(0..0, quad);
}
