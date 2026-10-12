// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Edits saved before version 4 kept annotation and scene clips in recording
//! time. They are carried onto the timeline once, as the edit is read: each
//! clip becomes the stretch, or stretches, of the timeline its footage plays
//! on. Footage the timeline cut away is left behind, and footage a reordered
//! timeline plays in separate places makes one clip per place.

use super::RecordingTimelineEdit;

/// A segment's place on the timeline, in milliseconds: the recording
/// `[source_start, source_end)` it plays, from `timeline_start` on, at
/// `rate`.
struct Placed {
  rate: f64,
  source_end: f64,
  source_start: f64,
  timeline_start: f64,
}

fn placed(edit: &RecordingTimelineEdit, duration_ms: u64) -> Vec<Placed> {
  let duration = duration_ms as f64;
  let mut timeline = 0.0;
  edit
    .segments
    .iter()
    .map(|segment| {
      let piece = Placed {
        rate: segment.playback_rate,
        source_end: segment.source_end * duration,
        source_start: segment.source_start * duration,
        timeline_start: timeline,
      };
      timeline += (piece.source_end - piece.source_start) / piece.rate;
      piece
    })
    .collect()
}

/// Where the recording `[start_ms, end_ms)` plays on the timeline, in whole
/// milliseconds: one stretch for each place it plays, in timeline order,
/// with stretches that meet joined.
fn on_timeline(segments: &[Placed], [start_ms, end_ms]: [u64; 2]) -> Vec<[u64; 2]> {
  let (start, end) = (start_ms as f64, end_ms as f64);
  let mut stretches: Vec<[f64; 2]> = segments
    .iter()
    .filter_map(|segment| {
      let from = start.max(segment.source_start);
      let to = end.min(segment.source_end);
      (to > from).then(|| {
        [
          segment.timeline_start + (from - segment.source_start) / segment.rate,
          segment.timeline_start + (to - segment.source_start) / segment.rate,
        ]
      })
    })
    .collect();
  stretches.sort_by(|a, b| a[0].total_cmp(&b[0]));
  let mut joined: Vec<[f64; 2]> = Vec::new();
  for stretch in stretches {
    match joined.last_mut() {
      Some(last) if stretch[0] - last[1] < 0.5 => last[1] = last[1].max(stretch[1]),
      _ => joined.push(stretch),
    }
  }
  joined
    .into_iter()
    .map(|[from, to]| [from.round() as u64, to.round() as u64])
    .filter(|[from, to]| to > from)
    .collect()
}

/// `edit`'s annotation and scene clips, saved in recording time, carried
/// onto the timeline its segments lay out for a recording `duration_ms`
/// long. A clip that plays in more than one place keeps its identity in the
/// first and takes a numbered one in each after it.
pub(crate) fn carry_clips_onto_timeline(edit: &mut RecordingTimelineEdit, duration_ms: u64) {
  let segments = placed(edit, duration_ms);
  edit.annotation_clips = std::mem::take(&mut edit.annotation_clips)
    .into_iter()
    .flat_map(|clip| {
      on_timeline(&segments, [clip.start_ms, clip.end_ms])
        .into_iter()
        .enumerate()
        .map(move |(index, [start_ms, end_ms])| {
          let mut part = clip.clone();
          part.start_ms = start_ms;
          part.end_ms = end_ms;
          if index > 0 {
            part.annotation.id = format!("{}-{}", clip.annotation.id, index + 1);
          }
          part
        })
        .collect::<Vec<_>>()
    })
    .collect();
  let mut scenes: Vec<_> = std::mem::take(&mut edit.scene_clips)
    .into_iter()
    .flat_map(|clip| {
      on_timeline(&segments, [clip.start_ms, clip.end_ms])
        .into_iter()
        .enumerate()
        .map(move |(index, [start_ms, end_ms])| {
          let mut part = clip.clone();
          part.start_ms = start_ms;
          part.end_ms = end_ms;
          if index > 0 {
            part.id = format!("{}-{}", clip.id, index + 1);
          }
          part
        })
        .collect::<Vec<_>>()
    })
    .collect();
  scenes.sort_by_key(|clip| clip.start_ms);
  edit.scene_clips = scenes;
}

#[cfg(test)]
mod tests;
