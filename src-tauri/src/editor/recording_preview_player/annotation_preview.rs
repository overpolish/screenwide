// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Time evaluation shared by playing and paused native compositions.

use super::*;
use crate::editor::annotations::timing::{
  revealed_annotations, AnnotationTrack, RecordingAnnotationClip,
};

/// How much source time one drawn frame covers, which is what the reveal's
/// motion blur is measured over. A paused composition passes zero: nothing
/// is moving, so nothing is blurred. `ranges` is the timeline the reveals are
/// timed on, and `pictures` the primary's and the camera's source sizes.
/// `held` hands the screen's redactions the fills read from their clips'
/// first frames; only the screen takes a redaction.
pub(super) fn apply_clips(
  composition: &mut PreviewCompositionSettings,
  clips: &[RecordingAnnotationClip],
  ranges: &[TimelineRange],
  source_ms: u64,
  frame_ms: f32,
  pictures: [(u32, u32); 2],
  held: Option<&HeldFillsHandle>,
) {
  let primary = &mut composition.recording_output.primary;
  primary.annotations = revealed_annotations(
    clips,
    AnnotationTrack::Primary,
    ranges,
    source_ms,
    frame_ms,
    pictures[0],
  );
  #[cfg(any(target_os = "macos", target_os = "windows"))]
  if let Some(held) = held {
    held.attach(
      &mut primary.annotations,
      clips,
      primary.capture_width_points,
      source_ms,
    );
  }
  #[cfg(not(any(target_os = "macos", target_os = "windows")))]
  let _ = held;
  composition.recording_output.camera.annotations = revealed_annotations(
    clips,
    AnnotationTrack::Camera,
    ranges,
    source_ms,
    frame_ms,
    pictures[1],
  );
}

/// The preview's held fills, where the platform decodes the frames they are
/// read from.
#[cfg(any(target_os = "macos", target_os = "windows"))]
pub(super) type HeldFillsHandle = Arc<super::held_fills::HeldFills>;
#[cfg(not(any(target_os = "macos", target_os = "windows")))]
pub(super) type HeldFillsHandle = ();

impl PlayerSources {
  /// Hands each pinned clip among `clips` its path, asking for any not yet
  /// worked out, nearest `position_ms` first. The screen's current canvas
  /// scale places the tips counters and text boxes are followed by, so a
  /// layout change attaches them again.
  pub(super) fn attach_pins(
    &self,
    clips: &mut [crate::editor::annotations::timing::RecordingAnnotationClip],
    position_ms: u64,
  ) {
    #[cfg(any(target_os = "macos", target_os = "windows"))]
    if let Some(pins) = &self.pins {
      pins.attach(clips, position_ms, self.screen_size_width());
    }
    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    let _ = (clips, position_ms);
  }

  /// How wide the screen is drawn on the canvas, in points of annotation
  /// size, or zero before the layout is known: what places the tips counters
  /// and text boxes are followed by.
  pub(super) fn screen_size_width(&self) -> f64 {
    self
      .composition_settings
      .as_ref()
      .and_then(|settings| settings.read().ok())
      .map_or(0.0, |settings| {
        settings.recording_output.primary.size_image_width()
      })
  }

  /// The primary's and the camera's source sizes, which a spotlight's glide
  /// is paced against; zero for a pane there is not.
  pub(super) fn annotation_pictures(&self) -> [(u32, u32); 2] {
    let pane = |index: usize| {
      self
        .playback_layout
        .panes
        .get(index)
        .map_or((0, 0), |pane| (pane.source_width, pane.source_height))
    };
    [pane(0), pane(1)]
  }

  /// Attaches every clip's pin again: after a path lands, or after a layout
  /// change that may have moved the tips counters and text boxes follow.
  pub(super) fn reattach_pins(&self, position_ms: u64) {
    if let Ok(mut clips) = self.annotation_clips.write() {
      self.attach_pins(&mut clips, position_ms);
    }
  }

  /// A paused macOS still resolves its annotations up front, because its worker
  /// composes from settings rather than from the frame it is about to
  /// present. The Windows path resolves them at present time instead.
  #[cfg(target_os = "macos")]
  pub(super) fn annotated_composition(
    &self,
    source_ms: u64,
    clips: &[RecordingAnnotationClip],
  ) -> Option<PreviewCompositionSettings> {
    let mut composition = self.composition_settings.as_ref()?.read().ok()?.clone();
    let ranges = self
      .animation_ranges
      .read()
      .unwrap_or_else(|poisoned| poisoned.into_inner());
    apply_clips(
      &mut composition,
      clips,
      &ranges,
      source_ms,
      0.0,
      self.annotation_pictures(),
      self.held_fills.as_ref(),
    );
    Some(composition)
  }
}

/// Annotations are authored in full-resolution source pixels; decoder proxies
/// have their own source grid while stroke width follows the output resolution.
pub(crate) fn remap_source(
  settings: &mut crate::screenshots::ScreenshotOutputSettings,
  source: (u32, u32),
  decoded: (u32, u32),
  output_width: u32,
) {
  use crate::editor::annotations::AnnotationPoint;
  let sx = f64::from(decoded.0) / f64::from(source.0.max(1));
  let sy = f64::from(decoded.1) / f64::from(source.1.max(1));
  let stroke = f64::from(settings.width) / f64::from(output_width.max(1));
  for annotation in &mut settings.annotations {
    annotation.shape = annotation.shape.mapped(|point| AnnotationPoint {
      x: point.x * sx,
      y: point.y * sy,
    });
    annotation.style.width *= stroke;
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::editor::annotations::{arrow::new_arrow, AnnotationPoint, AnnotationShape};
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
    let AnnotationShape::Arrow { start, end, .. } = output.annotations[0].shape else {
      unreachable!()
    };
    assert_eq!((start.x, start.y), (240.0, 135.0));
    assert_eq!((end.x, end.y), (720.0, 405.0));
    assert_eq!(output.annotations[0].style.width, width / 2.0);
  }
}
