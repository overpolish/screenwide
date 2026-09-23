// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! What a snapped sample draws: the axis guides a counter's disc landed on,
//! the equal gaps it lined up with, and the element an arrow's tip took
//! hold of.
//!
//! The twin of `annotation_add_snap_osc` in
//! `recording_preview_surface_macos+annotation_chrome.m`. Rust publishes both
//! normalised over the source image, exactly as it publishes the grips, so
//! everything here does is find that picture on screen and place them in it.

use super::*;

/// How far an equal-gap bar's end ticks reach either side of it, in points.
/// The twin of `kAnnotationGapTick`.
const GAP_TICK: f64 = 3.0;

/// The snap chrome in device pixels, ready for the selection overlay.
#[derive(Clone, Default)]
pub(crate) struct SnapChrome {
  /// The guide lines, in the same shape the layer engine's guides are drawn
  /// from: the x line, the y line, and whether each came from another
  /// annotation rather than the canvas.
  pub(crate) guides: Option<(Option<f32>, Option<f32>, bool, bool)>,
  /// Where the tip landed, which wears the grips' own disc.
  pub(crate) anchor: Option<[f32; 2]>,
  /// The element the anchor belongs to, outlined a device pixel wide.
  pub(crate) bounds: Option<[f32; 4]>,
  /// The equal-gap bars and their end ticks, already rectangles so the
  /// overlay only has to fill them. At most twelve: two axes, two gaps, a
  /// bar and two ticks each.
  pub(crate) gaps: Vec<[f32; 4]>,
}

/// Where the last sample's snap is drawn. Empty while the arrow chrome is not
/// on screen, or when that sample snapped to nothing.
pub(crate) fn snap_chrome(state: &SurfaceState, scale: f64) -> SnapChrome {
  let snap = state.annotation.snap;
  if snap.flags == 0 || !owns_chrome(state) {
    return SnapChrome::default();
  }
  let Some(image) = image_frame(state) else {
    return SnapChrome::default();
  };
  let guide_x = (snap.flags & SNAP_FLAG_GUIDE_X != 0)
    .then(|| window::pixel_center((image.x + snap.guide_x * image.width) * scale));
  let guide_y = (snap.flags & SNAP_FLAG_GUIDE_Y != 0)
    .then(|| window::pixel_center((image.y + snap.guide_y * image.height) * scale));
  let anchored = snap.flags & SNAP_FLAG_ANCHOR != 0;
  let mut gaps = Vec::new();
  if snap.flags & SNAP_FLAG_GAP_X != 0 {
    add_gap_bars(&mut gaps, &snap.gap_x, true, image, scale);
  }
  if snap.flags & SNAP_FLAG_GAP_Y != 0 {
    add_gap_bars(&mut gaps, &snap.gap_y, false, image, scale);
  }
  SnapChrome {
    guides: (guide_x.is_some() || guide_y.is_some()).then_some((
      guide_x,
      guide_y,
      snap.guide_x_object != 0,
      snap.guide_y_object != 0,
    )),
    anchor: anchored.then_some([
      ((image.x + snap.anchor_x * image.width) * scale) as f32,
      ((image.y + snap.anchor_y * image.height) * scale) as f32,
    ]),
    bounds: anchored.then_some([
      ((image.x + snap.box_x * image.width) * scale) as f32,
      ((image.y + snap.box_y * image.height) * scale) as f32,
      (snap.box_width * image.width * scale) as f32,
      (snap.box_height * image.height * scale) as f32,
    ]),
    gaps,
  }
}

/// One axis's two gaps as device-pixel rectangles: a hairline the length of
/// each gap, with a tick across each of its ends. `horizontal` is a gap
/// measured across the picture, whose bar runs left to right.
fn add_gap_bars(
  out: &mut Vec<[f32; 4]>,
  spans: &[NativeGapSpan; 2],
  horizontal: bool,
  image: PreviewSurfaceRect,
  scale: f64,
) {
  let tick = GAP_TICK * scale;
  for span in spans {
    let (origin, extent) = if horizontal {
      (image.x, image.width)
    } else {
      (image.y, image.height)
    };
    let (cross_origin, cross_extent) = if horizontal {
      (image.y, image.height)
    } else {
      (image.x, image.width)
    };
    let from = (origin + span.from * extent) * scale;
    let to = (origin + span.to * extent) * scale;
    if to < from {
      continue;
    }
    let cross = (cross_origin + span.cross * cross_extent) * scale;
    let bar =
      |x: f64, y: f64, width: f64, height: f64| [x as f32, y as f32, width as f32, height as f32];
    if to > from {
      out.push(if horizontal {
        bar(from, cross - 0.5, to - from, 1.0)
      } else {
        bar(cross - 0.5, from, 1.0, to - from)
      });
    }
    for end in [from, to] {
      out.push(if horizontal {
        bar(end - 0.5, cross - tick, 1.0, tick * 2.0)
      } else {
        bar(cross - tick, end - 0.5, tick * 2.0, 1.0)
      });
    }
  }
}
