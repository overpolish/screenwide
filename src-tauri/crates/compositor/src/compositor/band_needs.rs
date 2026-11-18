// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! What a layer's settings and prepared annotations ask of the bands that
//! draw it (`bands.rs`), and of the window of a picture larger than the
//! device takes that the editor draws it from.

use super::bands::{meets, needed, LayerNeeds};
use super::scene_turn::scene_turn_rows;
use super::CanvasGeometry;
use crate::annotation_gpu::{PreparedArrows, FOCUS_WIDEST_SHARE};
use screenwide_annotations::redact::records::{REDACT_BLUR, REDACT_MOSAIC, REDACT_SPOTLIGHT};
use screenwide_annotations::snap::source_per_size;
use screenwide_annotations::{Annotation, AnnotationShape};
use screenwide_output::ScreenshotOutputSettings;

/// The farthest a redaction's blur reads past its pixel, as `redact_reach`
/// in `redact.wgsl` holds it.
const BLUR_REACH: f32 = 160.0;

/// How far a lens's glass bends what a pixel shows towards the lens's middle,
/// with its colours parted, as a share of the lens's shorter half side: the
/// bend `annotation_glass.wgsl` holds to 0.8 of a bezel 0.3 wide, and its
/// fringe of 0.08 of that.
const GLASS_BEND: f64 = 0.3 * 0.8 * 1.08;

/// How far a highlight's pixel reads the picture past its own column, as a
/// share of its line's height (`highlight_fill` in
/// `annotation_highlight.wgsl`), and past that in source pixels, which its
/// widest spread of taps reaches.
const HIGHLIGHT_SIDE: f64 = 0.1;
const HIGHLIGHT_TAPS: f64 = 10.0;

/// What an annotation reads of the picture for the part of it a band draws,
/// in source pixels.
#[derive(Clone, Copy, Debug)]
pub(super) enum Read {
  /// A lens's box, x0, y0, x1, y1: each of its pixels shows the picture
  /// `zoom` times larger about the box's middle, bent inwards through its
  /// glass by at most `bend`, read `pad` either side for a filtered read.
  Lens {
    bounds: [f64; 4],
    zoom: f64,
    bend: f64,
    pad: f64,
  },
  /// A highlight's line, x0, y0, x1, y1: each of its pixels reads the line's
  /// whole height, and `reach` either side of its column.
  Line { bounds: [f64; 4], reach: f64 },
}

impl Read {
  /// What this reads for the part `shown` of the picture, where it reads
  /// anything there.
  pub(super) fn of(&self, shown: [f64; 4]) -> Option<[f64; 4]> {
    match *self {
      Self::Lens {
        bounds,
        zoom,
        bend,
        pad,
      } => {
        let drawn = intersection(bounds, grown(shown, bend))?;
        let centre = [(bounds[0] + bounds[2]) / 2.0, (bounds[1] + bounds[3]) / 2.0];
        let zoom = zoom.max(1.0);
        let shows = |at: f64, axis: usize| centre[axis] + (at - centre[axis]) / zoom;
        Some(grown(
          [
            shows(drawn[0], 0),
            shows(drawn[1], 1),
            shows(drawn[2], 0),
            shows(drawn[3], 1),
          ],
          pad,
        ))
      }
      Self::Line { bounds, reach } => {
        let drawn = intersection(bounds, shown)?;
        Some([
          drawn[0] - reach,
          bounds[1] - reach,
          drawn[2] + reach,
          bounds[3] + reach,
        ])
      }
    }
  }
}

fn grown(rect: [f64; 4], by: f64) -> [f64; 4] {
  [rect[0] - by, rect[1] - by, rect[2] + by, rect[3] + by]
}

fn intersection(a: [f64; 4], b: [f64; 4]) -> Option<[f64; 4]> {
  meets(a, b).then(|| {
    [
      a[0].max(b[0]),
      a[1].max(b[1]),
      a[2].min(b[2]),
      a[3].min(b[3]),
    ]
  })
}

/// What `annotation`, on a picture drawn `scale` canvas pixels to its own,
/// reads of the picture: a lens, or each line of a highlight.
fn reads(annotation: &Annotation, scale: f64, per_size: f64) -> Vec<Read> {
  match &annotation.shape {
    AnnotationShape::Magnify { start, end, zoom } => {
      let bounds = [
        start.x.min(end.x),
        start.y.min(end.y),
        start.x.max(end.x),
        start.y.max(end.y),
      ];
      let shorter = (bounds[2] - bounds[0]).min(bounds[3] - bounds[1]) / 2.0;
      vec![Read::Lens {
        bounds,
        zoom: *zoom,
        bend: shorter * GLASS_BEND,
        // A texel either side of the lens's ground, which lies a canvas
        // pixel apart at most, and the lens's feathered edge.
        pad: 2.0 + 2.0 / scale.max(1e-6),
      }]
    }
    AnnotationShape::Highlight { bands, .. } if !bands.is_empty() => bands
      .iter()
      .map(|band| Read::Line {
        bounds: [band.left, band.top, band.right, band.bottom],
        reach: (band.bottom - band.top) * HIGHLIGHT_SIDE + HIGHLIGHT_TAPS,
      })
      .collect(),
    // A highlight with no lines found is read across its box.
    AnnotationShape::Highlight { .. } => {
      screenwide_annotations::group::group_bounds(std::iter::once(annotation), per_size)
        .map(|bounds| {
          vec![Read::Line {
            bounds: bounds.edges(),
            reach: HIGHLIGHT_TAPS,
          }]
        })
        .unwrap_or_default()
    }
    _ => Vec::new(),
  }
}

impl LayerNeeds {
  pub(super) fn of(
    source: (u32, u32),
    settings: &ScreenshotOutputSettings,
    geometry: &CanvasGeometry,
    prepared: &PreparedArrows,
  ) -> Self {
    let placement = &geometry.placement;
    let upright = settings
      .rotation
      .is_none_or(|rotation| rotation.is_upright());
    let inverse_turn = (!upright).then(|| {
      let rows = scene_turn_rows(settings, placement, geometry.size);
      [0, 1, 2].map(|row| [rows[row][0], rows[row][1], rows[row][2]])
    });
    let per_size = source_per_size(source, settings.capture_width_points);
    let scale = placement.image_width / f64::from(source.0.max(1));
    let reads = settings
      .annotations
      .iter()
      .flat_map(|annotation| reads(annotation, scale, per_size))
      .collect();
    let margin = prepared
      .redactions
      .records
      .iter()
      .map(|record| match record.mode {
        REDACT_BLUR | REDACT_SPOTLIGHT => (record.size.max(0.0) * 3.0).ceil().min(BLUR_REACH),
        // A cell is averaged whole, so the window holds one either side.
        REDACT_MOSAIC => record.size.max(1.0).ceil() + 2.0,
        _ => 0.0,
      })
      .fold(0.0_f32, f32::max) as u32
      + 2;
    // The focus gather reads up to a few of its widest blurs round a pixel,
    // from levels whose texels are as wide as the coarsest.
    let overlap = if prepared.focus.is_empty() {
      0
    } else {
      let shorter = geometry.size.0.min(geometry.size.1) as f32;
      2 * screenwide_gpu::mips::COARSEST_TEXEL + (4.0 * FOCUS_WIDEST_SHARE * shorter).ceil() as u32
    };
    let [deviation, arrived] = prepared.spotlight_blur;
    let reads_past = if deviation.is_finite() && deviation >= 0.5 && arrived > 0.0 {
      super::mark_blur::reach(deviation).ceil() as u32
    } else {
      0
    };
    Self {
      source,
      image_rect: [
        placement.image_x,
        placement.image_y,
        placement.image_width,
        placement.image_height,
      ],
      inverse_turn,
      reads,
      margin,
      overlap,
      reads_past,
    }
  }
}

/// The part of a `source`-sized picture, x0, y0, x1, y1 in its pixels, that
/// drawing `part` of its canvas reads: what `part` shows of it, the whole of
/// every annotation there that reads the picture, and as far as a redaction
/// reads past a pixel it paints. `part` is x0, y0, x1, y1 in canvas pixels
/// and may reach past the canvas, where a placed canvas draws the picture
/// faded.
pub fn picture_needed(
  source: (u32, u32),
  settings: &ScreenshotOutputSettings,
  geometry: &CanvasGeometry,
  prepared: &PreparedArrows,
  part: [f64; 4],
) -> [u32; 4] {
  needed(&LayerNeeds::of(source, settings, geometry, prepared), part)
}
