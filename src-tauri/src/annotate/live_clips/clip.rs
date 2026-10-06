// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! One live annotation's visible span as an editable clip, shared by a
//! recording and the replay buffer.

use super::super::geometry::source_annotation;
use crate::editor::annotations::pace::path_ms;
use crate::editor::annotations::reveal::clip_ms_for_visible;
use crate::editor::annotations::timing::{AnnotationTrack, RecordingAnnotationClip};
use crate::editor::annotations::Annotation;
use crate::recording::cursor::CursorSource;

/// One annotation's clip for a recording that showed it from `start_ms` until
/// it went at `gone_ms`, paced by its path on the recording's own frame, as
/// the editor paces the clips it writes.
///
/// An animated annotation's clip runs past that moment by its own closing
/// phase, so it is still whole when it goes and starts leaving afterwards
/// rather than before. Callers trim whatever the recording had no room for.
pub(super) fn annotation_clip(
  annotation: &Annotation,
  source: &CursorSource,
  start_ms: u64,
  gone_ms: u64,
) -> Option<RecordingAnnotationClip> {
  let annotation = source_annotation(annotation, source)?;
  let visible_ms = gone_ms.saturating_sub(start_ms);
  let frame = (source.video_width, source.video_height);
  let path_ms = path_ms(&annotation, frame);
  let length_ms = if annotation.animated {
    clip_ms_for_visible(visible_ms as f32, path_ms).round() as u64
  } else {
    visible_ms
  };
  Some(RecordingAnnotationClip {
    path_ms,
    pin: None,
    annotation,
    track_id: AnnotationTrack::Primary,
    start_ms,
    // An annotation drawn and cleared inside one millisecond is still a clip
    // the editor has to be able to see and grab.
    end_ms: (start_ms + length_ms).max(start_ms + 1),
  })
}
