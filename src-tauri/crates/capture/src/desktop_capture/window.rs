// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! A window recorded from its displays rather than on its own: each frame
//! crops the window out of whichever displays it stands on, where it stands
//! at that moment, so it is followed as it moves. The output keeps the size
//! the window had when the recording started; a window resized since is
//! scaled to fit it, centred, the rest black.

use super::plan::{constrained_scale, intersections, piece, pixel_edge};
use super::{CapturePiece, DesktopDisplay, DesktopRect, OutputLimits};

/// The output a window recording keeps for its whole length: the window's
/// size at `scale`, the device pixels of the display it starts on, held to
/// the video limits.
pub fn window_output(window: DesktopRect, scale: f64) -> Option<(u32, u32)> {
  if !window.valid() || !scale.is_finite() || scale <= 0.0 {
    return None;
  }
  let limits = OutputLimits::VIDEO;
  let scale = constrained_scale(window, scale, limits);
  let edge = |points: f64, maximum: u32| {
    let pixels = pixel_edge(points, scale).min(maximum);
    pixels - pixels % limits.alignment
  };
  let width = edge(window.width, limits.max_width);
  let height = edge(window.height, limits.max_height);
  (width > 0 && height > 0).then_some((width, height))
}

/// Where the window, standing at `window` in global points, comes from on
/// each display and goes in a `width` by `height` output.
pub fn window_pieces(
  displays: &[DesktopDisplay],
  window: DesktopRect,
  width: u32,
  height: u32,
) -> Vec<CapturePiece> {
  if !window.valid() || width == 0 || height == 0 {
    return Vec::new();
  }
  let scale = (f64::from(width) / window.width).min(f64::from(height) / window.height);
  // The stretch of the desktop the whole output shows, so the window lands
  // centred in it.
  let shown = DesktopRect {
    x: window.x - (f64::from(width) / scale - window.width) / 2.0,
    y: window.y - (f64::from(height) / scale - window.height) / 2.0,
    width: f64::from(width) / scale,
    height: f64::from(height) / scale,
  };
  intersections(displays, window)
    .into_iter()
    .filter_map(|(display, intersection)| piece(display, intersection, shown, scale, width, height))
    .collect()
}

/// The radius, in pixels, of the smallest round corner that leaves out every
/// clear pixel in a window's top start corner, `clear(x, y)` telling them
/// apart, looked at up to `span` pixels in. A window's corner is a little
/// fuller than a circle, so the circle that clears all of it cuts a hair of
/// the window at its middle rather than leaving a sliver of black at its
/// ends. None where nothing is clear, a square corner, or where no corner
/// within `span` clears it all.
pub fn corner_radius(clear: impl Fn(u32, u32) -> bool, span: u32) -> Option<f64> {
  let clear_pixels = (0..span)
    .flat_map(|y| (0..span).map(move |x| (x, y)))
    .filter(|&(x, y)| clear(x, y))
    .collect::<Vec<_>>();
  if clear_pixels.is_empty() {
    return None;
  }
  // A pixel is outside a corner of radius `r` when its middle lies beyond
  // the arc about the corner's centre, `(r, r)`.
  let left_out = |radius: f64| {
    clear_pixels.iter().all(|&(x, y)| {
      let (dx, dy) = (radius - (f64::from(x) + 0.5), radius - (f64::from(y) + 0.5));
      dx > 0.0 && dy > 0.0 && dx.hypot(dy) >= radius
    })
  };
  (1..=span).map(f64::from).find(|&radius| left_out(radius))
}
