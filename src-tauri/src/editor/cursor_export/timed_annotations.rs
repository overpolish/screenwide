// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later
use super::*;
use crate::editor::annotations::native::{NativeAnnotation, NativeAnnotationData};
use crate::editor::annotations::redact::native::{source_per_capture_point, RedactSource};
use crate::editor::annotations::timing::{
  placed_annotation, AnnotationTrack, RecordingAnnotationClip,
};
use crate::editor::annotations::{Annotation, AnnotationKind};

/// One record the Metal export draws while `start_ms <= t < end_ms`. Its
/// arrival and leaving play over `reveal_start_ms..reveal_end_ms`, and a
/// redaction's surface timeline is read from `clip_start_ms`: for a clip that
/// is not pinned all three are its own bounds, while a pinned clip is one
/// record for each place its pin puts it, each keeping its clip's reveal and
/// surface.
#[repr(C)]
pub(super) struct NativeTimedAnnotation {
  annotation: NativeAnnotation,
  start_ms: u64,
  end_ms: u64,
  reveal_start_ms: u64,
  reveal_end_ms: u64,
  clip_start_ms: u64,
}
const _: () = assert!(std::mem::size_of::<NativeTimedAnnotation>() == 168);

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
      request.cancelled,
    );
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
  stroke_scale: f64,
  source: RedactSource<'_>,
) -> (Vec<NativeTimedAnnotation>, NativeAnnotationData) {
  let mut data = NativeAnnotationData::default();
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
              annotation: data.pack(&annotation, source),
              start_ms,
              end_ms,
              reveal_start_ms,
              reveal_end_ms,
              clip_start_ms: clip.start_ms,
            }
          },
        )
        .collect::<Vec<_>>()
    })
    .collect();
  (clips, data)
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::editor::annotations::{AnnotationPoint, AnnotationShape, AnnotationStyle};

  fn counter_clip(value: u32) -> RecordingAnnotationClip {
    RecordingAnnotationClip {
      pin: None,
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

  /// A pinned counter is drawn where its path puts it frame by frame, not
  /// drawn while its content is off the frame, and shown again over the
  /// stretch it comes back for.
  #[test]
  fn a_pinned_clip_is_drawn_where_its_path_puts_it_and_not_while_hidden() {
    use crate::editor::annotations::pin::model::PinKeyframe;
    use crate::editor::annotations::pin::resolve::{PinSample, PinnedPath};
    use crate::editor::annotations::pin::AnnotationPin;
    let sample = |ms, dy: f32, visible| PinSample {
      ms,
      dx: 0.0,
      dy,
      scale: 1.0,
      confidence: 1.0,
      visible,
    };
    let mut clip = counter_clip(1);
    // Standing whole, so coming back 300 ms before its end still shows it.
    clip.annotation.animated = false;
    clip.pin = Some(AnnotationPin {
      pinned_ms: 1_000,
      keyframes: vec![PinKeyframe {
        ms: 1_000,
        dx: 0.0,
        dy: 0.0,
      }],
      path: Some(std::sync::Arc::new(PinnedPath {
        samples: vec![
          sample(1_000, 0.0, true),
          sample(1_200, -5.0, true),
          sample(1_400, -50.0, false),
          sample(1_700, 0.0, true),
        ],
        shown: vec![[1_000, 1_400], [1_700, 2_000]],
        ..PinnedPath::default()
      })),
    });
    let (packed, _) = pack_clips(std::iter::once(&clip), 1.0, RedactSource::None);
    let spans: Vec<_> = packed
      .iter()
      .map(|record| {
        (
          record.start_ms,
          record.end_ms,
          record.reveal_start_ms,
          record.reveal_end_ms,
          record.annotation.p0[1],
        )
      })
      .collect();
    assert_eq!(
      spans,
      [
        (1_000, 1_198, 1_000, 1_400, 10.0),
        (1_198, 1_398, 1_000, 1_400, 5.0),
        (1_698, 2_000, 1_698, 2_000, 10.0),
      ]
    );
    assert!(packed.iter().all(|record| record.clip_start_ms == 1_000));
  }
}
