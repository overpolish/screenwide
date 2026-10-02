// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::super::PreparedArrows;
use super::placed_native;
use crate::editor::annotations::native::{native_annotations, NativeAnnotations};
use crate::editor::annotations::redact::native::{
  source_per_capture_point, RedactPicture, RedactSource,
};
use crate::editor::annotations::redact::records::{redact_records, RedactRecords};
use crate::editor::annotations::spotlight::native::{blur_deviation, blur_strength};
use crate::editor::annotations::text::typing::TypingMarks;
use crate::editor::annotations::Annotation;
use crate::screenshots::{CapturedImage, OutputPlacement, ScreenshotOutputSettings};

/// A layer's annotations resolved against its source once: what each draws,
/// the redactions the pre-pass applies in the source's own pixels, and the
/// spotlights' blur. Placing them is all a redraw at another geometry adds.
#[derive(Clone, Default)]
pub(crate) struct SourceAnnotations {
  native: NativeAnnotations,
  redactions: RedactRecords,
  blur_strength: f32,
  source: (u32, u32),
}

impl SourceAnnotations {
  /// `picture` is a screenshot's own pixels, which its redactions read their
  /// fills from; a video frame has none, and its redactions take the fills
  /// held from their clips' frames.
  pub(crate) fn new(
    annotations: &[Annotation],
    source: (u32, u32),
    settings: &ScreenshotOutputSettings,
    picture: Option<&CapturedImage>,
  ) -> Self {
    if annotations.is_empty() || source.0 == 0 || source.1 == 0 {
      return Self::default();
    }
    let picture = picture.map(|picture| {
      RedactPicture::new(
        &picture.rgba,
        picture.width,
        picture.height,
        settings.capture_width_points,
      )
    });
    let native = native_annotations(
      annotations,
      picture.as_ref().map_or_else(
        || RedactSource::Video {
          source_per_point: source_per_capture_point(source.0, settings.capture_width_points),
        },
        RedactSource::Picture,
      ),
      settings.size_scale(),
    );
    Self {
      redactions: redact_records(&native.items, &native.data.points, source.0, source.1),
      blur_strength: blur_strength(&native.items),
      native,
      source,
    }
  }

  /// The annotations placed with the picture `placement` puts on the canvas.
  /// `halo` is the hovered annotation's place in the list and the halo's
  /// width in canvas pixels; `typing` the box being typed into.
  pub(crate) fn placed(
    &self,
    placement: OutputPlacement,
    halo: Option<(usize, f32)>,
    typing: Option<(usize, TypingMarks)>,
  ) -> PreparedArrows {
    if self.native.items.is_empty() {
      return PreparedArrows::default();
    }
    let source = self.source;
    let mut prepared = placed_native(
      self.native.clone(),
      (placement.image_x, placement.image_y),
      (
        f64::from(placement.image_width) / f64::from(source.0),
        f64::from(placement.image_height) / f64::from(source.1),
      ),
      halo,
      typing,
    );
    prepared.redactions = self.redactions.clone();
    // The cursor and the marks are drawn on the canvas, so the source's blur
    // is carried there.
    prepared.spotlight_blur = [
      blur_deviation(self.blur_strength, source.0, source.1) * placement.image_width as f32
        / source.0 as f32,
      self.blur_strength,
    ];
    prepared
  }
}
