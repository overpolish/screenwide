// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::{NormalizedRect, FRAME_EDGE_BOTTOM, FRAME_EDGE_LEFT, FRAME_EDGE_RIGHT, FRAME_EDGE_TOP};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct InsetResize {
  pub rect: NormalizedRect,
  /// Ratio of the resized outer frame to its starting size. Horizontal and
  /// corner handles use width; vertical-only handles use height.
  pub scale: f64,
}

/// Resizes an outer frame by changing one uniform inset around fixed content.
/// `delta` is normalized to the canvas. The returned frame is clamped to the
/// canvas without ever moving or scaling `content`.
#[cfg(any(target_os = "windows", test))]
pub fn resize_uniform_inset(
  start: NormalizedRect,
  content: NormalizedRect,
  canvas: (f64, f64),
  edges: u32,
  delta: (f64, f64),
) -> InsetResize {
  let vertical_only = is_vertical_only(edges);
  let start_inset = inset_for_primary_axis(start, content, canvas, vertical_only);
  let requested_delta = if vertical_only {
    signed_outward_delta(delta.1 * canvas.1, edges, FRAME_EDGE_TOP, FRAME_EDGE_BOTTOM)
  } else {
    signed_outward_delta(delta.0 * canvas.0, edges, FRAME_EDGE_LEFT, FRAME_EDGE_RIGHT)
  };
  resized(
    content,
    start,
    canvas,
    vertical_only,
    start_inset + requested_delta,
  )
}

/// Reconstructs the same inset frame from the native gesture scalar emitted
/// to the preview model. This keeps React and the retained native pixels on
/// one gesture contract while the underlying image transform stays fixed.
pub fn resize_uniform_inset_from_scale(
  start: NormalizedRect,
  content: NormalizedRect,
  canvas: (f64, f64),
  edges: u32,
  scale: f64,
) -> InsetResize {
  let vertical_only = is_vertical_only(edges);
  let axis = if vertical_only { canvas.1 } else { canvas.0 };
  let start_size = if vertical_only {
    start.height
  } else {
    start.width
  };
  let content_size = if vertical_only {
    content.height
  } else {
    content.width
  };
  let inset = (start_size * axis * scale.max(0.0) - content_size * axis) / 2.0;
  resized(content, start, canvas, vertical_only, inset)
}

fn is_vertical_only(edges: u32) -> bool {
  edges & (FRAME_EDGE_TOP | FRAME_EDGE_BOTTOM) != 0
    && edges & (FRAME_EDGE_LEFT | FRAME_EDGE_RIGHT) == 0
}

#[cfg(any(target_os = "windows", test))]
fn inset_for_primary_axis(
  frame: NormalizedRect,
  content: NormalizedRect,
  canvas: (f64, f64),
  vertical_only: bool,
) -> f64 {
  if vertical_only {
    (frame.height - content.height) * canvas.1 / 2.0
  } else {
    (frame.width - content.width) * canvas.0 / 2.0
  }
}

#[cfg(any(target_os = "windows", test))]
fn signed_outward_delta(delta: f64, edges: u32, low: u32, high: u32) -> f64 {
  if edges & low != 0 && edges & high == 0 {
    -delta
  } else if edges & high != 0 && edges & low == 0 {
    delta
  } else {
    0.0
  }
}

fn resized(
  content: NormalizedRect,
  start: NormalizedRect,
  canvas: (f64, f64),
  vertical_only: bool,
  requested_inset: f64,
) -> InsetResize {
  let width = canvas.0.max(1.0);
  let height = canvas.1.max(1.0);
  let maximum = (content.x * width)
    .min(content.y * height)
    .min((1.0 - content.x - content.width) * width)
    .min((1.0 - content.y - content.height) * height)
    .max(0.0);
  let inset = requested_inset.clamp(0.0, maximum);
  let rect = NormalizedRect {
    x: content.x - inset / width,
    y: content.y - inset / height,
    width: content.width + inset * 2.0 / width,
    height: content.height + inset * 2.0 / height,
  };
  let (next_size, start_size) = if vertical_only {
    (rect.height, start.height)
  } else {
    (rect.width, start.width)
  };
  InsetResize {
    rect,
    scale: if start_size.abs() > f64::EPSILON {
      next_size / start_size
    } else {
      1.0
    },
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  fn rect(x: f64, y: f64, width: f64, height: f64) -> NormalizedRect {
    NormalizedRect {
      x,
      y,
      width,
      height,
    }
  }

  fn assert_rect_close(actual: NormalizedRect, expected: NormalizedRect) {
    assert!((actual.x - expected.x).abs() < 1e-9);
    assert!((actual.y - expected.y).abs() < 1e-9);
    assert!((actual.width - expected.width).abs() < 1e-9);
    assert!((actual.height - expected.height).abs() < 1e-9);
  }

  #[test]
  fn horizontal_drag_grows_equal_output_pixel_inset() {
    let content = rect(0.3, 0.2, 0.4, 0.6);
    let result = resize_uniform_inset(
      content,
      content,
      (1_000.0, 500.0),
      FRAME_EDGE_RIGHT,
      (0.04, 0.0),
    );
    assert_rect_close(result.rect, rect(0.26, 0.12, 0.48, 0.76));
    assert!((result.scale - 1.2).abs() < 1e-9);
  }

  #[test]
  fn vertical_drag_uses_height_for_the_emitted_scale() {
    let content = rect(0.3, 0.2, 0.4, 0.6);
    let result = resize_uniform_inset(
      content,
      content,
      (1_000.0, 500.0),
      FRAME_EDGE_TOP,
      (0.0, -0.04),
    );
    assert_rect_close(result.rect, rect(0.28, 0.16, 0.44, 0.68));
    assert!((result.scale - (0.68 / 0.6)).abs() < 1e-9);
  }

  #[test]
  fn corner_drag_uses_horizontal_travel_and_width_scale() {
    let content = rect(0.3, 0.2, 0.4, 0.6);
    let result = resize_uniform_inset(
      content,
      content,
      (1_000.0, 500.0),
      FRAME_EDGE_RIGHT | FRAME_EDGE_BOTTOM,
      (0.04, 0.5),
    );
    assert_rect_close(result.rect, rect(0.26, 0.12, 0.48, 0.76));
    assert!((result.scale - 1.2).abs() < 1e-9);
  }

  #[test]
  fn contraction_stops_at_the_fixed_content() {
    let content = rect(0.3, 0.2, 0.4, 0.6);
    let start = rect(0.26, 0.12, 0.48, 0.76);
    let result = resize_uniform_inset(
      start,
      content,
      (1_000.0, 500.0),
      FRAME_EDGE_LEFT,
      (1.0, 0.0),
    );
    assert_rect_close(result.rect, content);
  }

  #[test]
  fn expansion_stops_at_the_nearest_canvas_edge() {
    let content = rect(0.1, 0.2, 0.7, 0.5);
    let result = resize_uniform_inset(
      content,
      content,
      (1_000.0, 500.0),
      FRAME_EDGE_RIGHT,
      (2.0, 0.0),
    );
    assert_rect_close(result.rect, rect(0.0, 0.0, 0.9, 0.9));
  }

  #[test]
  fn emitted_scale_reconstructs_the_same_frame() {
    let content = rect(0.3, 0.2, 0.4, 0.6);
    let start = rect(0.28, 0.16, 0.44, 0.68);
    let direct = resize_uniform_inset(
      start,
      content,
      (1_000.0, 500.0),
      FRAME_EDGE_BOTTOM,
      (0.0, 0.08),
    );
    let reconstructed = resize_uniform_inset_from_scale(
      start,
      content,
      (1_000.0, 500.0),
      FRAME_EDGE_BOTTOM,
      direct.scale,
    );
    assert_rect_close(direct.rect, reconstructed.rect);
  }
}
