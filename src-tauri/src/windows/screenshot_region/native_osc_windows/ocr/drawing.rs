// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;

/// The pill is centred on the selection and kept a margin inside the surface
/// (`+ocr.m:106-113`).
pub(crate) fn status_rect(label_width: f64, view: Size, region: Rect) -> Rect {
  let width =
    (label_width + STATUS_PADDING_X * 2.0).min((view.width - STATUS_MARGIN * 2.0).max(0.0));
  let top = (region.origin.y + region.size.height * 0.5 - STATUS_HEIGHT * 0.5).clamp(
    STATUS_MARGIN,
    (view.height - STATUS_HEIGHT - STATUS_MARGIN).max(STATUS_MARGIN),
  );
  let left = (region.origin.x + region.size.width * 0.5 - width * 0.5).clamp(
    STATUS_MARGIN,
    (view.width - width - STATUS_MARGIN).max(STATUS_MARGIN),
  );
  Rect::from_xywh(left, top, width, STATUS_HEIGHT)
}

/// A plate, its icon and - for text buttons - its label, laid out the way
/// `render_control` (`+ocr_toolbar.m:78-101`) did.
pub(super) fn add_control(
  out: &mut Vec<Vertex>,
  view: Size,
  rect: Rect,
  metrics: &ControlMetrics,
  icon: ControlIcon,
  label: Option<&super::super::text::TextTexture>,
  is_button: bool,
  scale: f64,
) {
  let rect = renderer::pixel_aligned_rect(rect, scale);
  renderer::add_plate(out, view, rect);
  let icon_left = if is_button {
    rect.origin.x + metrics.padding_x
  } else {
    rect.origin.x + (rect.size.width - metrics.icon_size) * 0.5
  };
  renderer::add_icon(
    out,
    view,
    icon as u8,
    (icon_left * scale).round() / scale,
    ((rect.origin.y + (rect.size.height - metrics.icon_size) * 0.5) * scale).round() / scale,
    (metrics.icon_size * scale).round() / scale,
  );
  if let Some(label) = label {
    renderer::add_label(
      out,
      view,
      renderer::pixel_aligned_rect(
        Rect::from_xywh(
          rect.origin.x + metrics.padding_x + metrics.icon_size + metrics.gap,
          rect.origin.y + (rect.size.height - label.size.height) * 0.5,
          label.size.width,
          label.size.height,
        ),
        scale,
      ),
    );
  }
}

pub(super) fn visual_fills(visual: ControlVisual) -> [[f32; 4]; 2] {
  [visual.fill, visual.foreground]
}

pub(super) fn push_segment(
  segments: &mut Vec<Segment>,
  out: &[Vertex],
  start: usize,
  action_fills: [[f32; 4]; 2],
  radius: f64,
  outline: [f32; 4],
  label: Option<ID3D11ShaderResourceView>,
) {
  if out.len() <= start {
    return;
  }
  segments.push(Segment {
    start: start as u32,
    count: (out.len() - start) as u32,
    action_fills,
    chrome: [radius as f32, MATERIAL_EMPHASIS, 0.0, 0.0],
    chrome_outline: outline,
    label,
    secondary: None,
  });
}
