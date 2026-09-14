// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;

impl Ruler {
  /// The cursor readout (`render`, `+ruler.m:654-828`): a rounded plate, the
  /// picked-pixel swatch, its `#RRGGBB` hex, an optional `W × H px` row from
  /// the two live probes, the animated checkmark and the tolerance notice.
  #[allow(clippy::too_many_arguments)]
  pub(super) fn add_loupe(
    &mut self,
    device: &ID3D11Device,
    out: &mut Vec<Vertex>,
    segments: &mut Vec<Segment>,
    view: Size,
    display_id: u32,
    scale: f64,
    light_mode: bool,
    now: Instant,
    atlas: &super::super::text::TextTexture,
    cells: &AtlasMetrics,
    fills: [[f32; 4]; 2],
    value: ControlMetrics,
    control: f64,
    inset: f64,
  ) {
    if !self.transient_chrome
      || self.interaction_active
      || self.hovered_artifact_key != 0
      || !point_in_surface(self.point, view)
    {
      return;
    }
    let tolerance = self.tolerance_amount(now);
    let tolerance_label = (self.tolerance_visible || tolerance > 0.001)
      .then(|| {
        self.text.ink_label(
          device,
          tolerance_text(self.tolerance_mode),
          scale,
          light_mode,
          value.font_size,
          value.line_height,
        )
      })
      .flatten();

    let colour = hex_text(self.color);
    let dimensions = probe_dimensions_text(&self.data.probes, display_id);
    // A ToggleMenuButton at its capture size: the swatch in the leading glyph
    // slot, then the dimensions over the hex on the readout tier. The glyph
    // slot carries the control's side inset outside the swatch and half the
    // icon-to-label gap inside it; the text takes the other half and the side
    // inset closes the right edge. The hex is seven cells of one pitch, so
    // only the dimensions change the width, and the callout follows the
    // pointer anyway.
    // Proportional, as the dimensions line and the macOS readout are: a
    // fixed pitch made the hex read as tabular digits in a monospaced face.
    let colour_width = cells.text_width(&colour);
    let dimensions_width = dimensions
      .as_ref()
      .map_or(0.0, |text| cells.text_width(text));
    let text_offset = inset + value.icon_size + control + control;
    let width = text_offset + colour_width.max(dimensions_width) + inset;
    let height = value.callout_height;
    let origin = loupe_origin(self.point, width, height, view, inset);
    let plate = Rect::from_xywh(origin.x, origin.y, width, height);

    let start = out.len();
    renderer::add_plate(out, view, renderer::pixel_aligned_rect(plate, scale));
    let icon_top = (height - value.icon_size) * 0.5;
    let swatch = Rect::from_xywh(
      origin.x + inset,
      origin.y + icon_top,
      value.icon_size,
      value.icon_size,
    );
    renderer::add_pixel_aligned_quad(out, view, swatch, scale, 29);

    // The lines stack with no gap between them, and the block they make is
    // centred in the taller control.
    let text_left = origin.x + text_offset;
    let line_height = atlas.size.height;
    let lines = if dimensions.is_some() { 2.0 } else { 1.0 };
    let mut text_top = origin.y + (height - line_height * lines) * 0.5;
    if let Some(dimensions) = dimensions.as_ref() {
      add_atlas_text(
        out,
        view,
        cells,
        dimensions,
        text_left,
        text_top,
        line_height,
        0.0,
        scale,
        48,
      );
      text_top += line_height;
    }
    add_atlas_text(
      out,
      view,
      cells,
      &colour,
      text_left,
      text_top,
      line_height,
      0.0,
      scale,
      48,
    );

    // Matches CheckOnClick: scale and fade in, then only fade on expiry.
    let copied = self.copied.amount(now);
    let check_scale = if self.copied.target { copied } else { 1.0 };
    let center_x = swatch.origin.x + swatch.size.width * 0.5;
    let center_y = swatch.origin.y + swatch.size.height * 0.5;
    let a = Point {
      x: center_x - 4.0 * check_scale,
      y: center_y - 0.5 * check_scale,
    };
    let b = Point {
      x: center_x - 1.0 * check_scale,
      y: center_y + 2.5 * check_scale,
    };
    let c = Point {
      x: center_x + 5.0 * check_scale,
      y: center_y - 3.5 * check_scale,
    };
    renderer::add_line(out, view, a, b, 2.0 * check_scale.max(0.001), 30);
    renderer::add_line(out, view, b, c, 2.0 * check_scale.max(0.001), 30);

    let secondary = tolerance_label.as_ref().and_then(|label| {
      (tolerance > 0.001).then(|| {
        let label_scale = if self.tolerance.target {
          tolerance
        } else {
          1.0
        };
        let size = Size {
          width: label.size.width * label_scale,
          height: label.size.height * label_scale,
        };
        renderer::add_pixel_aligned_texture_quad(
          out,
          view,
          Rect::from_xywh(
            origin.x + (width - size.width) * 0.5,
            origin.y + (height - size.height) * 0.5,
            size.width,
            size.height,
          ),
          Rect::from_xywh(0.0, 0.0, 1.0, 1.0),
          scale,
          37,
        );
        label.view.clone()
      })
    });
    push_segment(
      segments,
      out,
      start,
      fills,
      value.callout_radius,
      control_stroke(if light_mode {
        Appearance::Light
      } else {
        Appearance::Dark
      }),
      Some(atlas.view.clone()),
      secondary,
    );
  }
}
