// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The annotations an exported frame draws, resolved on the CPU for the
//! shared compositor exactly as the preview resolves them.

use super::CursorExportRequest;
use crate::editor::annotations::timing::{
  revealed_annotations, AnnotationTrack, RecordingAnnotationClip,
};
use crate::editor::annotations::{Annotation, AnnotationKind};
use crate::editor::recording_preview_player::{held_surfaces, pin_paths};
use crate::editor::timeline_edit::TimelineRange;

/// The export's annotation clips, ready to be revealed frame by frame, and
/// what their reveal is measured against.
pub(super) struct ExportAnnotations {
  clips: Vec<RecordingAnnotationClip>,
  track: AnnotationTrack,
  ranges: Vec<TimelineRange>,
  picture: (u32, u32),
}

impl ExportAnnotations {
  pub(super) fn for_request(request: &CursorExportRequest<'_>) -> Self {
    let mut clips = request
      .timeline
      .map_or(&[][..], |timeline| timeline.annotation_clips())
      .to_vec();
    // Pinned annotations take their paths, which the preview has normally
    // worked out already. The screen's redactions then take the fills read
    // from their clips' frames: a secure pixelation's zones from its first,
    // and the surface across it, both where the pin carries the box. Both
    // come before the stroke is scaled below: the preview placed counters'
    // and text boxes' tips at the edited scale, so the export asks for the
    // same paths.
    if request.annotation_track == AnnotationTrack::Primary {
      pin_paths::attach_for_export(
        request.screen,
        request.duration_ms,
        &mut clips,
        request.output.size_image_width(),
        request.cancelled,
      );
      held_surfaces::attach_for_export(
        request.screen,
        request.duration_ms,
        &mut clips,
        request.output.capture_width_points,
      );
    }
    // Annotations are authored against the source at its own scale while the
    // stroke follows the output. Scaled once rather than per frame. A
    // redaction's width is its block, which covers the source rather than
    // drawing on the output, so it keeps its size in source pixels.
    for clip in &mut clips {
      if clip.annotation.shape.kind() != AnnotationKind::Redact {
        clip.annotation.style.width *= f64::from(request.video.resolution_scale_percent)
          / f64::from(request.video.source_scale_percent.max(1));
      }
    }
    Self {
      clips,
      track: request.annotation_track,
      ranges: request
        .timeline
        .map_or(&[][..], |timeline| timeline.ranges())
        .to_vec(),
      picture: (request.width, request.height),
    }
  }

  /// What the frame `position_ms` into the source draws, revealed as far as
  /// it has arrived on the edited timeline. `window_ms` is how much source
  /// time the frame covers, which a moving annotation smears over.
  pub(super) fn at(&self, position_ms: u64, window_ms: f32) -> Vec<Annotation> {
    let mut annotations = revealed_annotations(
      &self.clips,
      self.track,
      &self.ranges,
      position_ms,
      window_ms,
      self.picture,
    );
    held_surfaces::resolve_surfaces(&mut annotations, &self.clips, position_ms);
    annotations
  }
}
