// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;

impl Ruler {
  /// The floating chrome: the four pooled label sets and then the loupe, which
  /// macOS kept above them in the view order.
  #[allow(clippy::too_many_arguments)]
  pub(crate) fn add_chrome_vertices(
    &mut self,
    device: &ID3D11Device,
    out: &mut Vec<Vertex>,
    segments: &mut Vec<Segment>,
    view: Size,
    display_id: u32,
    offset: Point,
    scale: f64,
    light_mode: bool,
    now: Instant,
  ) {
    self.label_rects.clear();
    if !self.visible {
      return;
    }
    let value = metrics();
    // Every readout assembled from this atlas - the callout's two lines and
    // the measurement, probe and radius labels - sits one tier below a
    // control label, so the atlas is rasterized at the readout role.
    let Some(atlas) = self.text.hex_atlas(
      device,
      scale,
      light_mode,
      value.readout_font_size,
      value.readout_line_height,
    ) else {
      return;
    };
    let Some(cells) = atlas.atlas else {
      return;
    };
    let appearance = if light_mode {
      Appearance::Light
    } else {
      Appearance::Dark
    };
    // macOS read these from a one-control `ControlGroup` that nothing ever
    // hovered, so the resolved visual is the normal one.
    let visual = control_visual(
      ControlStyle::button(ControlColor::Neutral, ControlSize::Regular),
      Interaction::Normal,
      appearance,
    );
    let fills = [visual.fill, visual.foreground];
    let (control, inset) = spacing();

    for item in self.labels.clone() {
      let (id, kind, text, frame) = match item {
        LabelItem::Measurement(measurement) => {
          let text = measurement_text(Rect::from_xywh(
            measurement.x,
            measurement.y,
            measurement.width,
            measurement.height,
          ));
          let plate = measurement_label_rect(
            self,
            measurement,
            cells.text_width(&text),
            value,
            control,
            inset,
            view,
            offset,
          );
          (measurement.id, 1_u8, text, plate)
        }
        LabelItem::Probe(probe) | LabelItem::GuideGap(probe) => {
          let text = stamped_probe_text(probe);
          let plate = probe_label_rect(
            self,
            probe,
            None,
            cells.text_width(&text),
            value,
            control,
            inset,
            view,
            offset,
          );
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
            value,
            control,
            inset,
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
      let start = out.len();
      renderer::add_plate(out, view, renderer::pixel_aligned_rect(frame, scale));
      let text_top = frame.origin.y + (frame.size.height - atlas.size.height) * 0.5;
      add_atlas_text(
        out,
        view,
        &cells,
        &text,
        frame.origin.x + value.padding_x,
        text_top,
        atlas.size.height,
        0.0,
        scale,
        11,
      );
      push_segment(
        segments,
        out,
        start,
        fills,
        value.radius,
        control_stroke(appearance),
        Some(atlas.view.clone()),
        None,
      );
    }

    self.add_loupe(
      device, out, segments, view, display_id, scale, light_mode, now, &atlas, &cells, fills,
      value, control, inset,
    );
  }
}
