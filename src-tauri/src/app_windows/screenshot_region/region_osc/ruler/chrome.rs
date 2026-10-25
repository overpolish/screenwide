// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::tokens::{CAPTION_LINE, CAPTION_SIZE, INSET};
use super::*;
use screenwide_osc::style::RULER_LABEL_FILLS;

/// Room round a label for its shadow, in points: `--island-shadow`'s wide
/// layer, 10 down with a 28 blur, fades out inside it (`chrome_shadow` in
/// `osc.wgsl`, which reads the points back from this room).
const SHADOW_ROOM: f64 = 52.0;
/// A radius past half of any label's height, which makes its plate a pill.
pub(super) const PILL: f64 = 1.0e4;
/// How large a label starts as it comes in; it grows to full size over the
/// transition as it fades in, from its own centre.
const INTRO_SCALE: f64 = 0.9;

impl Ruler {
  /// The floating chrome: the four pooled label sets and then the loupe, which
  /// macOS kept above them in the view order. Every label and the loupe are
  /// drawn in the island's black whatever the appearance, so they carry the
  /// dark appearance's ink.
  #[allow(clippy::too_many_arguments)]
  pub(crate) fn add_chrome_vertices(
    &mut self,
    device: &Gpu,
    out: &mut Vec<Vertex>,
    segments: &mut Vec<Segment>,
    view: Size,
    display_id: u32,
    offset: Point,
    scale: f64,
    now: Instant,
  ) {
    self.label_rects.clear();
    if !self.visible {
      return;
    }
    // Every figure the Ruler shows - the callout's two lines and the
    // measurement, probe and radius labels - is a readout, so the atlas is
    // set in the caption role.
    let Some(atlas) = self
      .text
      .hex_atlas(device, scale, false, CAPTION_SIZE, CAPTION_LINE)
    else {
      return;
    };
    let Some(cells) = atlas.atlas else {
      return;
    };

    for item in self.labels.clone() {
      let (id, kind, text, frame) = match item {
        LabelItem::Measurement(measurement) => {
          let text = measurement_text(Rect::from_xywh(
            measurement.x,
            measurement.y,
            measurement.width,
            measurement.height,
          ));
          let plate =
            measurement_label_rect(self, measurement, cells.text_width(&text), view, offset);
          (measurement.id, 1_u8, text, plate)
        }
        LabelItem::Probe(probe) | LabelItem::GuideGap(probe) => {
          let text = stamped_probe_text(probe);
          let plate = probe_label_rect(self, probe, None, cells.text_width(&text), view, offset);
          (
            probe.id,
            if matches!(item, LabelItem::Probe(_)) {
              2
            } else {
              3
            },
            text,
            plate,
          )
        }
        LabelItem::Radius(radius) => {
          let text = radius_text(radius);
          let probe = radius_label_probe(radius);
          let plate = probe_label_rect(
            self,
            probe,
            Some(radius),
            cells.text_width(&text),
            view,
            offset,
          );
          (radius.id, 4_u8, text, plate)
        }
      };
      self.label_rects.push(LabelRect {
        id,
        kind,
        rect: frame,
      });
      let intro = self.label_intro(kind, id, now);
      let start = out.len();
      add_shadowed_plate(out, view, frame, scale);
      let text_top = frame.origin.y + (frame.size.height - atlas.size.height) * 0.5;
      add_atlas_text(
        out,
        view,
        &cells,
        &text,
        frame.origin.x + INSET,
        text_top,
        atlas.size.height,
        0.0,
        scale,
        11,
      );
      grow_from_centre(&mut out[start..], view, frame, intro);
      push_segment(
        segments,
        out,
        start,
        RULER_LABEL_FILLS,
        label_chrome(PILL, scale, intro),
        Some(atlas.view.clone()),
        None,
      );
    }

    self.add_loupe(
      device, out, segments, view, display_id, scale, now, &atlas, &cells,
    );
  }
}

/// A label's plate and, under it, the room its shadow is drawn in.
pub(super) fn add_shadowed_plate(out: &mut Vec<Vertex>, view: Size, frame: Rect, scale: f64) {
  renderer::add_quad(
    out,
    view,
    Rect::from_xywh(
      frame.origin.x - SHADOW_ROOM,
      frame.origin.y - SHADOW_ROOM,
      frame.size.width + SHADOW_ROOM * 2.0,
      frame.size.height + SHADOW_ROOM * 2.0,
    ),
    65,
  );
  renderer::add_plate(out, view, renderer::pixel_aligned_rect(frame, scale));
}

/// The segment's `chrome` row for a label: its radius and shadow room in
/// pixels, both as large as the label currently is, and how far it has
/// still to fade in.
pub(super) fn label_chrome(radius: f64, scale: f64, intro: f64) -> [f32; 4] {
  let size = intro_scale(intro);
  [
    (radius * scale * size) as f32,
    0.0,
    (SHADOW_ROOM * scale * size) as f32,
    (1.0 - intro) as f32,
  ]
}

fn intro_scale(intro: f64) -> f64 {
  INTRO_SCALE + (1.0 - INTRO_SCALE) * intro
}

/// Scales a label's vertices about its centre while it comes in. Vertices are
/// already in clip space, which maps each axis linearly from the view, so
/// scaling there about the centre's clip position is scaling the label.
pub(super) fn grow_from_centre(vertices: &mut [Vertex], view: Size, frame: Rect, intro: f64) {
  if intro >= 1.0 {
    return;
  }
  let size = intro_scale(intro) as f32;
  let centre = [
    (2.0 * (frame.origin.x + frame.size.width * 0.5) / view.width.max(1.0) - 1.0) as f32,
    (1.0 - 2.0 * (frame.origin.y + frame.size.height * 0.5) / view.height.max(1.0)) as f32,
  ];
  for vertex in vertices {
    for axis in 0..2 {
      vertex.position[axis] = centre[axis] + (vertex.position[axis] - centre[axis]) * size;
    }
  }
}
