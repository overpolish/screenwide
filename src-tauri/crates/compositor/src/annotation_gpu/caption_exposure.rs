// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! A caption piece over the frame's exposure: drawn at its size and how
//! solid it is now, and, where it moved, grew or faded since the exposure
//! began, at every sample along the way, which the shader averages, as it
//! does a moving annotation's.

use super::arrows::{PreviewArrow, PreviewGeometry, PreviewSample};
use super::prepare::exposure_sample_count;
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

/// `arrow`, prepared at its piece's full size and solid, drawn as `piece`
/// is now, with an exposure sample for each step of the way it came since
/// the exposure began, added to `samples`. A backdrop is sized by its
/// block, so it moves but never grows.
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
  arrow.geometry = grown(full, centre, scale[0], [0.0; 2]);
  // As far as its farthest corner travelled: across and down, and out from
  // the centre as it grew.
  let reach = (full.bx - full.ax).hypot(full.by - full.ay) * 0.5;
  let travel = (now[0] - opened[0]).hypot(now[1] - opened[1]) + reach * (scale[0] - scale[1]).abs();
  let count = exposure_sample_count(travel, opacity - opened_opacity);
  if count == 0 {
    // A still piece carries how solid it is in its colour, as a still
    // annotation does.
    arrow.color[3] *= opacity;
    return;
  }
  arrow.sample_first = samples.len() as u32;
  arrow.sample_count = count;
  for tap in 0..count {
    let t = (tap as f32 + 0.5) / count as f32;
    let lerp = |from: f32, to: f32| from + (to - from) * t;
    let by = [0, 1].map(|axis| lerp(opened[axis], now[axis]) - now[axis]);
    samples.push(PreviewSample {
      geometry: grown(full, centre, lerp(scale[1], scale[0]), by),
      opacity: lerp(opened_opacity, opacity),
    });
  }
}
