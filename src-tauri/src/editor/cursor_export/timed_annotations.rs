// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later
use super::*;
use crate::editor::annotations::native::{NativeAnnotation, NativeAnnotationData};
use crate::editor::annotations::redact::native::{source_per_capture_point, RedactSource};
use crate::editor::annotations::timing::{
  placed_annotation, AnnotationTrack, RecordingAnnotationClip,
};
use crate::editor::annotations::{Annotation, AnnotationKind};
use crate::editor::timeline_edit::{output_at_us, TimelineRange};

/// One record the Metal export draws while `start_ms <= t < end_ms` in source
/// time. Its arrival and leaving play from `reveal_start_ms` to
/// `reveal_end_ms` in output time, at its clip's pace `path_ms` - zero where
/// the clip names none - so a cut or a speed change moves neither out of
/// step with the frames that are kept. A redaction's surface timeline is read
/// from `clip_start_ms`, in source time. A clip that is not pinned is one
/// record over its own bounds, while a pinned clip is one record for each
/// place its pin puts it, each keeping its clip's reveal and surface.
#[repr(C)]
pub(super) struct NativeTimedAnnotation {
  annotation: NativeAnnotation,
  start_ms: u64,
  end_ms: u64,
  reveal_start_ms: u64,
  reveal_end_ms: u64,
  clip_start_ms: u64,
  path_ms: f32,
  padding: u32,
}
const _: () = assert!(std::mem::size_of::<NativeTimedAnnotation>() == 176);

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
    crate::editor::recording_preview_player::pin_paths::attach_for_export(
      request.screen,
      request.duration_ms,
      &mut clips,
      request.output.size_image_width(),
      request.cancelled,
    );
    crate::editor::recording_preview_player::held_surfaces::attach_for_export(
      request.screen,
      request.duration_ms,
      &mut clips,
      request.output.capture_width_points,
    );
  }
  // Sizes are in points, drawn at the capture's scale, and the stroke then
  // follows the output while the points stay in the source.
  let stroke_scale = request.output.size_scale()
    * f64::from(request.video.resolution_scale_percent)
    / f64::from(request.video.source_scale_percent.max(1));
  let source = RedactSource::Video {
    source_per_point: source_per_capture_point(request.width, request.output.capture_width_points),
  };
  let ranges = request
    .timeline
    .map_or(&[][..], |timeline| timeline.ranges());
  pack_clips(clips.iter(), ranges, stroke_scale, source)
}

/// How far either side of a sample a frame's time may be rounded.
const FRAME_SLACK_MS: u64 = 2;

/// The moments a pinned clip may change where it is drawn: each tracked
/// frame, or, before its path is worked out, halfway between keyframes.
fn pinned_changes(clip: &RecordingAnnotationClip) -> Vec<u64> {
  let Some(pin) = clip.pin.as_ref() else {
    return vec![clip.start_ms];
  };
  let mut times: Vec<u64> = match pin.path.as_deref() {
    // A frame's own time may be rounded a millisecond or two either side of
    // its sample's, so each place starts that much early.
    Some(path) => path
      .samples
      .iter()
      .map(|sample| sample.ms.saturating_sub(FRAME_SLACK_MS))
      .collect(),
    None => {
      let keyframes = pin.sorted();
      keyframes
        .windows(2)
        .map(|pair| pair[0].ms + (pair[1].ms - pair[0].ms) / 2)
        .collect()
    }
  };
  times.retain(|ms| *ms > clip.start_ms && *ms < clip.end_ms);
  times.insert(0, clip.start_ms);
  times
}

/// `clip` as records: one for a clip that is not pinned, and one for each
/// stretch a pinned clip is drawn in one place, with the stretches it is off
/// the frame left out.
fn placed_records(clip: &RecordingAnnotationClip) -> Vec<(Annotation, u64, u64, [u64; 2])> {
  if clip.pin.is_none() {
    return vec![(
      clip.annotation.clone(),
      clip.start_ms,
      clip.end_ms,
      [clip.start_ms, clip.end_ms],
    )];
  }
  let changes = pinned_changes(clip);
  let mut records: Vec<(Annotation, u64, u64, [u64; 2])> = Vec::new();
  for (index, &from) in changes.iter().enumerate() {
    let to = changes.get(index + 1).copied().unwrap_or(clip.end_ms);
    let at = if from == clip.start_ms {
      from
    } else {
      from + FRAME_SLACK_MS
    };
    let Some((annotation, shown)) = placed_annotation(clip, at) else {
      continue;
    };
    match records.last_mut() {
      Some((last, _, end, last_shown))
        if *end == from && *last == annotation && last_shown[1] == shown[1] =>
      {
        *end = to;
      }
      // The stretch arrives from where its first record starts, which may be
      // the slack before its first sample.
      _ => records.push((annotation, from, to, [shown[0].min(from), shown[1]])),
    }
  }
  records
}

fn pack_clips<'a>(
  clips: impl Iterator<Item = &'a RecordingAnnotationClip>,
  ranges: &[TimelineRange],
  stroke_scale: f64,
  source: RedactSource<'_>,
) -> (Vec<NativeTimedAnnotation>, NativeAnnotationData) {
  let mut data = NativeAnnotationData::default();
  let output_ms = |ms: u64| (output_at_us(ranges, ms as f64 * 1_000.0) / 1_000.0).round() as u64;
  let clips = clips
    .flat_map(|clip| {
      placed_records(clip)
        .into_iter()
        .map(
          |(mut annotation, start_ms, end_ms, [reveal_start_ms, reveal_end_ms])| {
            // A redaction's width is its block, which covers the source rather
            // than drawing on the output, so it keeps its size in source pixels.
            if annotation.shape.kind() != AnnotationKind::Redact {
              annotation.style.width *= stroke_scale;
            }
            NativeTimedAnnotation {
              annotation: data.pack(&annotation, source, 1.0),
              start_ms,
              end_ms,
              reveal_start_ms: output_ms(reveal_start_ms),
              reveal_end_ms: output_ms(reveal_end_ms),
              clip_start_ms: clip.start_ms,
              path_ms: clip.path_ms.unwrap_or(0.0),
              padding: 0,
            }
          },
        )
        .collect::<Vec<_>>()
    })
    .collect();
  (clips, data)
}

#[cfg(test)]
#[path = "timed_annotations_tests.rs"]
mod tests;
