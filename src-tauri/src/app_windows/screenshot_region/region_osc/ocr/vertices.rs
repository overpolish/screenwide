// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use std::time::Instant;

use screenwide_osc::geometry::{Rect, Size};

use super::super::renderer::{self, Vertex};
use super::{rect_kind, sweep, Chrome, PHASE_LOADING, PHASE_READY};

/// `OCR_SWEEP_KIND` in `osc.wgsl`.
const SWEEP_KIND: u32 = 69;
/// The area read is outlined as a QR code is (`ocr_box` kind 18).
const BORDER_KIND: u32 = 18;

impl Chrome {
  /// World-space OCR border, sheen and highlight quads.
  pub(crate) fn add_world_vertices(
    &self,
    out: &mut Vec<Vertex>,
    view: Size,
    region: Rect,
    scale: f64,
    now: Instant,
  ) {
    let live = self.phase == PHASE_LOADING || self.phase == PHASE_READY;
    if live && region.size.width > 0.0 && region.size.height > 0.0 {
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
        renderer::add_pixel_aligned_quad(out, view, edge, scale, BORDER_KIND);
      }
    }
    for highlight in &self.rects {
      renderer::add_pixel_aligned_quad(out, view, highlight.rect, scale, rect_kind(highlight.kind));
    }
    // Over the highlights, so the light passes over what was found.
    if let Some(sheen) = self.sheen.filter(|sheen| live && sheen.is_live(now)) {
      let elapsed = now.saturating_duration_since(sheen.since).as_secs_f64();
      if let Some((rect, uv)) = sweep::band(region, elapsed) {
        renderer::add_texture_quad(out, view, rect, uv, SWEEP_KIND);
      }
    }
  }
}
