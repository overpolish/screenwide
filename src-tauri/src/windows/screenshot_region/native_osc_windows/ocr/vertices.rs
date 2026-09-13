// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;

impl Chrome {
  /// World-space OCR border and highlight quads.
  pub(crate) fn add_world_vertices(
    &self,
    out: &mut Vec<Vertex>,
    view: Size,
    region: Rect,
    scale: f64,
  ) {
    if (self.phase == PHASE_LOADING || self.phase == PHASE_READY)
      && region.size.width > 0.0
      && region.size.height > 0.0
    {
      let half = 1.0 / scale.max(1.0);
      let (left, top) = (region.origin.x, region.origin.y);
      let (width, height) = (region.size.width, region.size.height);
      for edge in [
        Rect::from_xywh(left - half, top - half, width + half * 2.0, half * 2.0),
        Rect::from_xywh(
          left - half,
          region.bottom() - half,
          width + half * 2.0,
          half * 2.0,
        ),
        Rect::from_xywh(left - half, top - half, half * 2.0, height + half * 2.0),
        Rect::from_xywh(
          region.right() - half,
          top - half,
          half * 2.0,
          height + half * 2.0,
        ),
      ] {
        renderer::add_pixel_aligned_quad(out, view, edge, scale, 18);
      }
    }
    for highlight in &self.rects {
      renderer::add_pixel_aligned_quad(out, view, highlight.rect, scale, rect_kind(highlight.kind));
    }
  }

  /// Lays the chrome out and appends it as one draw call per control. Layout
  /// happens here rather than in `set_ocr` so a resized or re-scaled surface
  /// can never hit-test against a stale rectangle.
  #[allow(clippy::too_many_arguments)]
  pub(crate) fn add_chrome_vertices(
    &mut self,
    device: &ID3D11Device,
    out: &mut Vec<Vertex>,
    segments: &mut Vec<Segment>,
    view: Size,
    region: Rect,
    scale: f64,
    light_mode: bool,
  ) {
    let appearance = if light_mode {
      Appearance::Light
    } else {
      Appearance::Dark
    };
    self.add_status(device, out, segments, view, region, scale, light_mode);
    self.add_cancel(device, out, segments, view, scale, light_mode, appearance);
    self.add_toolbar(
      device, out, segments, view, region, scale, light_mode, appearance,
    );
  }
}
