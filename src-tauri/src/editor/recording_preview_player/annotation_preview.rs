// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Time evaluation shared by playing and paused native compositions.

use super::*;
use crate::editor::annotations::timing::{
  revealed_annotations, AnnotationTrack, RecordingAnnotationClip,
};

/// How much source time one drawn frame covers, which is what the reveal's
/// motion blur is measured over. A paused composition passes zero: nothing
/// is moving, so nothing is blurred.
///
pub(super) fn apply_clips(
  composition: &mut PreviewCompositionSettings,
  clips: &[RecordingAnnotationClip],
  source_ms: u64,
  frame_ms: f32,
) {
  composition.recording_output.primary.annotations =
    revealed_annotations(clips, AnnotationTrack::Primary, source_ms, frame_ms);
  composition.recording_output.camera.annotations =
    revealed_annotations(clips, AnnotationTrack::Camera, source_ms, frame_ms);
}

impl PlayerSources {
  pub(super) fn annotated_composition(
    &self,
    source_ms: u64,
    clips: &[RecordingAnnotationClip],
  ) -> Option<PreviewCompositionSettings> {
    let mut composition = self.composition_settings.as_ref()?.read().ok()?.clone();
    apply_clips(&mut composition, clips, source_ms, 0.0);
    Some(composition)
  }
}

/// Marks are authored in full-resolution source pixels; decoder proxies have
/// their own source grid while stroke width follows the output resolution.
pub(super) fn remap_source(
  settings: &mut crate::screenshots::ScreenshotOutputSettings,
  source: (u32, u32),
  decoded: (u32, u32),
  output_width: u32,
) {
  use crate::editor::annotations::AnnotationShape;
  let sx = f64::from(decoded.0) / f64::from(source.0.max(1));
  let sy = f64::from(decoded.1) / f64::from(source.1.max(1));
  let stroke = f64::from(settings.width) / f64::from(output_width.max(1));
  for annotation in &mut settings.annotations {
    let AnnotationShape::Arrow {
      start,
      control,
      end,
    } = &mut annotation.shape;
    for point in [start, control, end] {
      point.x *= sx;
      point.y *= sy;
    }
    annotation.style.width *= stroke;
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::editor::annotations::{model::new_arrow, AnnotationPoint, AnnotationShape};
  #[test]
  fn proxy_decode_preserves_normalized_arrow_position_and_stroke() {
    let mut output = crate::screenshots::test_output_settings(960, 540);
    output.annotations = vec![new_arrow(
      "a".into(),
      AnnotationPoint { x: 480.0, y: 270.0 },
      AnnotationPoint {
        x: 1440.0,
        y: 810.0,
      },
      None,
    )];
    let width = output.annotations[0].style.width;
    remap_source(&mut output, (1920, 1080), (960, 540), 1920);
    let AnnotationShape::Arrow { start, end, .. } = output.annotations[0].shape;
    assert_eq!((start.x, start.y), (240.0, 135.0));
    assert_eq!((end.x, end.y), (720.0, 405.0));
    assert_eq!(output.annotations[0].style.width, width / 2.0);
  }
}
