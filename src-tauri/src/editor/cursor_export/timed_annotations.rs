// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later
use super::*;
use crate::editor::annotations::native::{NativeAnnotation, NativeAnnotationData};
use crate::editor::annotations::redact::native::{source_per_capture_point, RedactSource};
use crate::editor::annotations::timing::{AnnotationTrack, RecordingAnnotationClip};
use crate::editor::annotations::AnnotationKind;

#[repr(C)]
pub(super) struct NativeTimedAnnotation {
  annotation: NativeAnnotation,
  start_ms: u64,
  end_ms: u64,
}
const _: () = assert!(std::mem::size_of::<NativeTimedAnnotation>() == 144);

/// Every clip on the export's track as a native record, and the one set of
/// side buffers their `data_offset`s index. Each frame's scene is the clips
/// showing then, pointing into these same buffers, so the offsets hold. The
/// screen's redactions take the fills read from their clips' first frames.
pub(super) fn for_request(
  request: &CursorExportRequest<'_>,
) -> (Vec<NativeTimedAnnotation>, NativeAnnotationData) {
  let mut clips: Vec<_> = request
    .timeline
    .map_or(&[][..], |t| t.annotation_clips())
    .iter()
    .filter(|clip| clip.track_id == request.annotation_track)
    .cloned()
    .collect();
  if request.annotation_track == AnnotationTrack::Primary {
    crate::editor::recording_preview_player::held_surfaces::attach_for_export(
      request.screen,
      request.duration_ms,
      &mut clips,
      request.output.capture_width_points,
    );
  }
  // The stroke follows the output while the points stay in the source.
  let stroke_scale = f64::from(request.video.resolution_scale_percent)
    / f64::from(request.video.source_scale_percent.max(1));
  let source = RedactSource::Video {
    source_per_point: source_per_capture_point(request.width, request.output.capture_width_points),
  };
  pack_clips(clips.iter(), stroke_scale, source)
}

fn pack_clips<'a>(
  clips: impl Iterator<Item = &'a RecordingAnnotationClip>,
  stroke_scale: f64,
  source: RedactSource<'_>,
) -> (Vec<NativeTimedAnnotation>, NativeAnnotationData) {
  let mut data = NativeAnnotationData::default();
  let clips = clips
    .map(|clip| {
      let mut annotation = clip.annotation.clone();
      // A redaction's width is its block, which covers the source rather
      // than drawing on the output, so it keeps its size in source pixels.
      if annotation.shape.kind() != AnnotationKind::Redact {
        annotation.style.width *= stroke_scale;
      }
      NativeTimedAnnotation {
        annotation: data.pack(&annotation, source),
        start_ms: clip.start_ms,
        end_ms: clip.end_ms,
      }
    })
    .collect();
  (clips, data)
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::editor::annotations::{Annotation, AnnotationPoint, AnnotationShape, AnnotationStyle};

  fn counter_clip(value: u32) -> RecordingAnnotationClip {
    RecordingAnnotationClip {
      annotation: Annotation {
        above_camera: false,
        animated: true,
        id: value.to_string(),
        held: None,
        reveal: Default::default(),
        shape: AnnotationShape::Counter {
          center: AnnotationPoint { x: 10.0, y: 10.0 },
          value,
          angle: 0.0,
        },
        style: AnnotationStyle {
          align: Default::default(),
          color: "#ffcc00".to_owned(),
          head: Default::default(),
          radius: 0.0,
          redaction: Default::default(),
          strength: 0.0,
          width: 40.0,
        },
      },
      end_ms: 2_000,
      start_ms: 1_000,
      track_id: AnnotationTrack::Primary,
    }
  }

  /// A clip packed on its own would index a buffer that is thrown away, and
  /// the export would draw its counter with no number in it.
  #[test]
  fn every_clip_indexes_the_one_shared_text_buffer() {
    let clips = [counter_clip(1), counter_clip(12)];
    let (packed, data) = pack_clips(clips.iter(), 1.0, RedactSource::None);
    assert_eq!(data.text, b"112");
    let slots: Vec<_> = packed
      .iter()
      .map(|clip| (clip.annotation.data_offset, clip.annotation.data_count))
      .collect();
    assert_eq!(slots, [(0, 1), (1, 2)]);
  }
}
