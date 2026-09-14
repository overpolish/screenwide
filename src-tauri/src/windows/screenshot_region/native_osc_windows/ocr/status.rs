// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;

impl Chrome {
  #[allow(clippy::too_many_arguments)]
  pub(super) fn add_status(
    &mut self,
    device: &ID3D11Device,
    out: &mut Vec<Vertex>,
    segments: &mut Vec<Segment>,
    view: Size,
    region: Rect,
    scale: f64,
    light_mode: bool,
  ) {
    if !self.status_visible || self.message.is_empty() {
      return;
    }
    let Some(label) = self.text.label(
      device,
      &self.message,
      scale,
      light_mode,
      STATUS_FONT_SIZE,
      STATUS_LINE_HEIGHT,
    ) else {
      return;
    };
    let palette = ocr_palette(light_mode);
    // The plate keeps its material backing untinted and unbordered: both status
    // fills are transparent and the outline alpha of 0 skips the border in the
    // shader, matching the macOS pill.
    // No spinner yet: macOS spins a CAShapeLayer inside the pill's own layer,
    // and this compositor has no per-frame animated chrome primitive to hang
    // an equivalent arc on. Follow-up, tracked with the rest of the Windows
    // parity pass.
    let (fill, foreground) = if self.phase == PHASE_ERROR {
      (palette.status_error_fill, palette.status_error_foreground)
    } else {
      (palette.loading_fill, palette.loading_foreground)
    };
    let outline = [0.0; 4];
    let plate = status_rect(label.size.width, view, region);
    let start = out.len();
    renderer::add_plate(out, view, renderer::pixel_aligned_rect(plate, scale));
    renderer::add_label(
      out,
      view,
      renderer::pixel_aligned_rect(
        Rect::from_xywh(
          plate.origin.x + (plate.size.width - label.size.width) * 0.5,
          plate.origin.y + (plate.size.height - label.size.height) * 0.5,
          label.size.width,
          label.size.height,
        ),
        scale,
      ),
    );
    push_segment(
      segments,
      out,
      start,
      [fill, foreground],
      STATUS_RADIUS,
      outline,
      Some(label.view.clone()),
    );
  }

  #[allow(clippy::too_many_arguments)]
  pub(super) fn add_cancel(
    &mut self,
    device: &ID3D11Device,
    out: &mut Vec<Vertex>,
    segments: &mut Vec<Segment>,
    view: Size,
    scale: f64,
    light_mode: bool,
    appearance: Appearance,
  ) {
    if !self.cancel_visible {
      return;
    }
    let metrics = cancel_metrics();
    let Some(label) = self.text.label(
      device,
      "Cancel",
      scale,
      light_mode,
      metrics.font_size,
      metrics.line_height,
    ) else {
      return;
    };
    let width = metrics.padding_x * 2.0 + metrics.icon_size + metrics.gap + label.size.width;
    let left = ((view.width - width) * 0.5).floor();
    let plate = Rect::from_xywh(left, CANCEL_TOP, width, metrics.height);
    self.cancel.layout(&[ControlSpec {
      rect: plate,
      style: ControlStyle::button(ControlColor::Neutral, ControlSize::Regular),
      icon: ControlIcon::X,
    }]);
    let Some(visual) = self.cancel.visuals(appearance).first().copied() else {
      return;
    };
    let start = out.len();
    add_control(
      out,
      view,
      plate,
      ControlRender {
        metrics: &metrics,
        icon: ControlIcon::X,
        label: Some(&*label),
        is_button: true,
        scale,
      },
    );
    push_segment(
      segments,
      out,
      start,
      visual_fills(visual),
      metrics.radius,
      control_stroke(appearance),
      Some(label.view.clone()),
    );
  }
}
