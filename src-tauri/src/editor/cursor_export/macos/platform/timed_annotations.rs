// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later
use super::*;
use crate::editor::annotations::native::{NativeAnnotation, NativeAnnotationData};
use crate::editor::annotations::redact::native::{source_per_capture_point, RedactSource};
use crate::editor::annotations::spotlight::handoff::{
  drawn_until, glide_ms, joined, spotlight_links, SpotlightLinks,
};
use crate::editor::annotations::timing::{
  output_progress, placed_annotation, AnnotationTrack, RecordingAnnotationClip,
};
use crate::editor::annotations::{Annotation, AnnotationKind};
use crate::editor::timeline_edit::{output_at_us, rate_at, TimelineRange};

/// One record the Metal export draws while `start_ms <= t < end_ms` in source
/// time. Its arrival and leaving play from `reveal_start_ms` to
/// `reveal_end_ms` in output time, at its clip's pace `path_ms` - zero where
/// the clip names none - so a cut or a speed change moves neither out of
/// step with the frames that are kept. A redaction's surface timeline is read
/// from `clip_start_ms`, in source time. A clip that is not pinned is one
/// record over its own bounds, while a pinned clip is one record for each
/// place its pin puts it, each keeping its clip's reveal and surface. A
/// spotlight carries the ends it hands its light over at, as
/// `SpotlightJoins` bits, and how far its blur has arrived; one gliding in
/// from the spotlight before is one record for each step of its glide.
#[repr(C)]
pub(super) struct NativeTimedAnnotation {
  annotation: NativeAnnotation,
  start_ms: u64,
  end_ms: u64,
  reveal_start_ms: u64,
  reveal_end_ms: u64,
  clip_start_ms: u64,
  path_ms: f32,
  joins: u32,
  blur_share: f32,
  padding: u32,
}
const _: () = assert!(std::mem::size_of::<NativeTimedAnnotation>() == 184);

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
  pack_clips(
    &clips,
    ranges,
    stroke_scale,
    source,
    (request.width, request.height),
  )
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

/// How much output time each record of a glide covers: short enough that
/// the light moves a frame's worth at most between them at any frame rate.
const GLIDE_STEP_MS: f64 = 4.0;

/// One record's annotation, its source bounds, the stretch its reveal plays
/// over, and how far its blur has arrived.
type Timed = (Annotation, u64, u64, [u64; 2], f32);

/// Clip `index` of `clips` as timed records, on a picture `picture` source
/// pixels in size: a spotlight handing its light on is drawn until the next
/// one starts, and one taking it over glides in through a record for each
/// step, each drawn as the preview draws its middle.
fn timed_records(
  clips: &[RecordingAnnotationClip],
  links: &[SpotlightLinks],
  index: usize,
  ranges: &[TimelineRange],
  picture: (u32, u32),
) -> Vec<Timed> {
  let clip = &clips[index];
  let until = drawn_until(clips, links, index);
  let held = |ms: u64| if ms == clip.end_ms { until } else { ms };
  let mut records: Vec<Timed> = placed_records(clip)
    .into_iter()
    .map(|(annotation, start, end, [from, to])| {
      (annotation, start, held(end), [from, held(to)], 1.0)
    })
    .collect();
  if links[index].from.is_none() {
    return records;
  }
  let mut glide = Vec::new();
  let mut at = clip.start_ms;
  while at < until {
    let rate = rate_at(ranges, at as f64 * 1_000.0);
    let next = (at + (GLIDE_STEP_MS * rate).round().max(1.0) as u64).min(until);
    let middle = (at + next) / 2;
    let Some((annotation, [from, to])) = placed_annotation(clip, middle.min(clip.end_ms - 1))
    else {
      break;
    };
    let shown = [from, held(to)];
    let Some((elapsed_ms, duration_ms)) = output_progress(ranges, shown, middle) else {
      break;
    };
    if glide_ms(clips, links, index, picture, duration_ms).is_none_or(|ms| elapsed_ms >= ms) {
      break;
    }
    let (drawn, blur_share) = joined(
      clips,
      links,
      index,
      picture,
      annotation,
      elapsed_ms,
      duration_ms,
    );
    glide.push((drawn, at, next, shown, blur_share));
    at = next;
  }
  // What follows the glide keeps its own records from where the glide ends.
  records.retain_mut(|record| {
    record.1 = record.1.max(at);
    record.1 < record.2
  });
  glide.extend(records);
  glide
}

fn pack_clips(
  clips: &[RecordingAnnotationClip],
  ranges: &[TimelineRange],
  stroke_scale: f64,
  source: RedactSource<'_>,
  picture: (u32, u32),
) -> (Vec<NativeTimedAnnotation>, NativeAnnotationData) {
  let mut data = NativeAnnotationData::default();
  let output_ms = |ms: u64| (output_at_us(ranges, ms as f64 * 1_000.0) / 1_000.0).round() as u64;
  let links = spotlight_links(clips);
  let mut packed = Vec::new();
  for (index, clip) in clips.iter().enumerate() {
    let joins = links[index].joins().bits();
    for (mut annotation, start_ms, end_ms, [reveal_start_ms, reveal_end_ms], blur_share) in
      timed_records(clips, &links, index, ranges, picture)
    {
      // A redaction's width is its block, which covers the source rather
      // than drawing on the output, so it keeps its size in source pixels.
      if annotation.shape.kind() != AnnotationKind::Redact {
        annotation.style.width *= stroke_scale;
      }
      packed.push(NativeTimedAnnotation {
        annotation: data.pack(&annotation, source, 1.0),
        start_ms,
        end_ms,
        reveal_start_ms: output_ms(reveal_start_ms),
        reveal_end_ms: output_ms(reveal_end_ms),
        clip_start_ms: clip.start_ms,
        path_ms: clip.path_ms.unwrap_or(0.0),
        joins,
        blur_share,
        padding: 0,
      });
    }
  }
  (packed, data)
}

#[cfg(test)]
mod tests;
