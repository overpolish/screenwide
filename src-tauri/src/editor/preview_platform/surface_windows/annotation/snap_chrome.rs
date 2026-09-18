// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! What a snapped sample draws: the axis guides a counter's centre landed on,
//! and the element an arrow's tip took hold of.
//!
//! The twin of `annotation_add_snap_osc` in
//! `recording_preview_surface_macos+annotation_chrome.m`. Rust publishes both
//! normalised over the source image, exactly as it publishes the grips, so
//! everything here does is find that picture on screen and place them in it.

use super::*;

/// The snap chrome in device pixels, ready for the selection overlay.
#[derive(Clone, Copy, Default)]
pub(crate) struct SnapChrome {
  /// The guide lines, in the same shape the layer engine's guides are drawn
  /// from: the x line, the y line, and whether each came from another
  /// annotation rather than the canvas.
  pub(crate) guides: Option<(Option<f32>, Option<f32>, bool, bool)>,
  /// Where the tip landed, which wears the grips' own disc.
  pub(crate) anchor: Option<[f32; 2]>,
  /// The element the anchor belongs to, outlined a device pixel wide.
  pub(crate) bounds: Option<[f32; 4]>,
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
  SnapChrome {
    guides: (guide_x.is_some() || guide_y.is_some()).then_some((
      guide_x,
      guide_y,
      snap.guide_x_object != 0,
      snap.guide_y_object != 0,
    )),
    anchor: anchored.then(|| {
      [
        ((image.x + snap.anchor_x * image.width) * scale) as f32,
        ((image.y + snap.anchor_y * image.height) * scale) as f32,
      ]
    }),
    bounds: anchored.then(|| {
      [
        ((image.x + snap.box_x * image.width) * scale) as f32,
        ((image.y + snap.box_y * image.height) * scale) as f32,
        (snap.box_width * image.width * scale) as f32,
        (snap.box_height * image.height * scale) as f32,
      ]
    }),
  }
}
