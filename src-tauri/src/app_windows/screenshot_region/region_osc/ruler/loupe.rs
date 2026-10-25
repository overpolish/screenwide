// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::chrome::{add_shadowed_plate, label_chrome, PILL};
use super::tokens::{CALLOUT_HEIGHT, CAPTION_LINE, CAPTION_SIZE, ICON, INLINE, INSET, SEPARATE};
use super::*;
use screenwide_osc::style::RULER_LABEL_FILLS;

/// The hex digits and letters, whose widest advance sets the hex line's
/// width, so the callout keeps one width as the colour under it changes.
const HEX_DIGITS: &str = "0123456789ABCDEF";

impl Ruler {
  /// The cursor readout, on the labels' black: the picked colour's swatch,
  /// and beside it the `W × H px` of the two live probes over the colour's
  /// `#RRGGBB` in the secondary ink, or the hex alone with no probes. The
  /// swatch turns to a check when the colour is copied, and a tolerance
  /// change shows its name over the whole callout for a moment.
  #[allow(clippy::too_many_arguments)]
  pub(super) fn add_loupe(
    &mut self,
    device: &Gpu,
    out: &mut Vec<Vertex>,
    segments: &mut Vec<Segment>,
    view: Size,
    display_id: u32,
    scale: f64,
    now: Instant,
    atlas: &super::super::text::TextTexture,
    cells: &AtlasMetrics,
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
          &tolerance_text(self.tolerance_mode),
          scale,
          false,
          CAPTION_SIZE,
          CAPTION_LINE,
        )
      })
      .flatten();

    let colour = hex_text(self.color);
    let dimensions = probe_dimensions_text(&self.data.probes, display_id);
    let hex_width = steady_hex_width(cells);
    let dimensions_width = dimensions
      .as_ref()
      .map_or(0.0, |text| cells.text_width(text));
    let height = CALLOUT_HEIGHT;
    // A pill with the swatch at its start, the size and circle of the
    // annotate island's colour, as far in from the edge as from the top.
    let edge = (height - ICON) * 0.5;
    let text_offset = edge + ICON + INLINE;
    let width = text_offset + hex_width.max(dimensions_width) + INSET;
    let origin = loupe_origin(self.point, width, height, view, SEPARATE);
    let plate = Rect::from_xywh(origin.x, origin.y, width, height);

    let start = out.len();
    add_shadowed_plate(out, view, plate, scale);
    let swatch = Rect::from_xywh(origin.x + edge, origin.y + edge, ICON, ICON);
    renderer::add_pixel_aligned_quad(out, view, swatch, scale, 29);

    // The lines stack with no gap between them, and the block they make is
    // centred in the taller callout.
    let text_left = origin.x + text_offset;
    let line_height = atlas.size.height;
    let lines = if dimensions.is_some() { 2.0 } else { 1.0 };
    let mut text_top = origin.y + (height - line_height * lines) * 0.5;
    let colour_kind = if let Some(dimensions) = dimensions.as_ref() {
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
      67
    } else {
      48
    };
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
      colour_kind,
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
      RULER_LABEL_FILLS,
      label_chrome(PILL, scale, 1.0),
      Some(atlas.view.clone()),
      secondary,
    );
  }
}

/// `#` and six of the widest hex glyph: the hex line's width for any colour.
fn steady_hex_width(cells: &AtlasMetrics) -> f64 {
  let widest = HEX_DIGITS
    .chars()
    .filter_map(super::super::text::glyph_index)
    .map(|index| cells.advance(index))
    .fold(0.0_f64, f64::max);
  cells.text_width("#") + widest * 6.0
}
