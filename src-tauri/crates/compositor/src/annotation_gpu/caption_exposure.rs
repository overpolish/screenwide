// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! A caption piece over the frame's exposure: drawn at its size, how solid
//! and how far turned it is now, and, where it moved, grew, faded or turned
//! since the exposure began, at every sample along the way, which the
//! shader averages, as it does a moving annotation's.

use super::arrows::{PreviewArrow, PreviewGeometry, PreviewSample};
use super::caption_turn::{hold_bounds, turn_geometry, Turn};
use super::prepare::exposure_sample_count;
use screenwide_annotations::flags::{CASTS, TURNED};
use screenwide_captions::CaptionPiece;

/// `geometry`, prepared at its full size about `centre`, drawn `scale`
/// times as large about that centre and moved `by`.
fn grown(geometry: PreviewGeometry, centre: [f32; 2], scale: f32, by: [f32; 2]) -> PreviewGeometry {
  let at = |value: f32, axis: usize| centre[axis] + (value - centre[axis]) * scale + by[axis];
  let mut grown = geometry;
  grown.ax = at(geometry.ax, 0);
  grown.bx = at(geometry.bx, 0);
  grown.ay = at(geometry.ay, 1);
  grown.by = at(geometry.by, 1);
  // The type's centre, in the box's start head.
  grown.start_head[4] = at(geometry.start_head[4], 0);
  grown.start_head[5] = at(geometry.start_head[5], 1);
  grown.width = geometry.width * scale;
  grown.rounding = geometry.rounding * scale;
  grown
}

fn lerp<const N: usize>(from: [f64; N], to: [f64; N], t: f64) -> [f64; N] {
  std::array::from_fn(|axis| from[axis] + (to[axis] - from[axis]) * t)
}

/// How `piece` is turned `t` of the way from as the exposure began to now.
fn turn_at(piece: &CaptionPiece, t: f64) -> Turn {
  Turn {
    own: lerp(piece.turn[1], piece.turn[0], t),
    line: piece.line_turn.as_ref().map(|line| {
      (
        lerp(line.turn[1], line.turn[0], t),
        lerp(line.pivot[1], line.pivot[0], t),
        line.reach,
      )
    }),
  }
}

/// How far, in canvas pixels, a point `reach` from `piece`'s centre moves
/// as it turns over the exposure: along the arc of each turn, the line's
/// swinging it about a pivot as far again as the line reaches.
fn turn_travel(piece: &CaptionPiece, reach: f32) -> f32 {
  let swept = |[now, before]: [[f64; 3]; 2]| -> f64 {
    (0..3)
      .map(|axis| (now[axis] - before[axis]).to_radians().abs())
      .sum()
  };
  let own = swept(piece.turn) * f64::from(reach);
  let line = piece.line_turn.as_ref().map_or(0.0, |line| {
    swept(line.turn) * (line.reach + f64::from(reach))
  });
  (own + line) as f32
}

/// `arrow`, prepared at its piece's full size and solid, drawn as `piece`
/// is now, with an exposure sample for each step of the way it came since
/// the exposure began, added to `samples`. A backdrop is sized by its
/// block, so it moves but never grows. A turning piece carries its turn in
/// each geometry, and its arrow the bounds every sample reaches.
pub(super) fn expose(
  arrow: &mut PreviewArrow,
  piece: &CaptionPiece,
  grows: bool,
  samples: &mut Vec<PreviewSample>,
) {
  let full = arrow.geometry;
  let centre = [(full.ax + full.bx) * 0.5, (full.ay + full.by) * 0.5];
  let as_f32 = |pair: [f64; 2]| pair.map(|value| value as f32);
  let scale = if grows {
    as_f32(piece.scale).map(|scale| scale.max(0.0))
  } else {
    [1.0; 2]
  };
  let [opacity, opened_opacity] = as_f32(piece.opacity).map(|opacity| opacity.clamp(0.0, 1.0));
  let now = as_f32(piece.centre);
  let opened = as_f32(piece.previous_centre);
  let turns = piece.turn != [[0.0; 3]; 2] || piece.line_turn.is_some();
  // A turned word's shadow turns with it, so its bounds hold how far the
  // shadow falls and spreads, grown as the word is.
  let shadow = if arrow.flags & CASTS != 0 {
    full.end_head[0].hypot(full.end_head[1]) + full.end_head[2]
  } else {
    0.0
  };
  let margin = |geometry: &PreviewGeometry| {
    f64::from(
      shadow
        * if full.cx > 0.0 {
          geometry.width / full.cx
        } else {
          1.0
        },
    )
  };
  arrow.geometry = grown(full, centre, scale[0], [0.0; 2]);
  let mut seen = true;
  if turns {
    arrow.flags |= TURNED;
    let geometry = arrow.geometry;
    seen = turn_geometry(&mut arrow.geometry, &turn_at(piece, 1.0), margin(&geometry));
  }
  // As far as its farthest corner travelled: across and down, out from the
  // centre as it grew, and round as it turned.
  let reach = (full.bx - full.ax).hypot(full.by - full.ay) * 0.5;
  let travel = (now[0] - opened[0]).hypot(now[1] - opened[1])
    + reach * (scale[0] - scale[1]).abs()
    + turn_travel(piece, reach * scale[0].max(scale[1]));
  let count = exposure_sample_count(travel, opacity - opened_opacity);
  if count == 0 {
    // A still piece carries how solid it is in its colour, as a still
    // annotation does; edge on, it shows nothing.
    arrow.color[3] *= if seen { opacity } else { 0.0 };
    return;
  }
  arrow.sample_first = samples.len() as u32;
  arrow.sample_count = count;
  for tap in 0..count {
    let t = (tap as f32 + 0.5) / count as f32;
    let lerp = |from: f32, to: f32| from + (to - from) * t;
    let by = [0, 1].map(|axis| lerp(opened[axis], now[axis]) - now[axis]);
    let mut sample = PreviewSample {
      geometry: grown(full, centre, lerp(scale[1], scale[0]), by),
      opacity: lerp(opened_opacity, opacity),
    };
    if turns {
      let room = margin(&sample.geometry);
      if !turn_geometry(&mut sample.geometry, &turn_at(piece, f64::from(t)), room) {
        sample.opacity = 0.0;
      }
      hold_bounds(&mut arrow.geometry, &sample.geometry);
    }
    samples.push(sample);
  }
}
