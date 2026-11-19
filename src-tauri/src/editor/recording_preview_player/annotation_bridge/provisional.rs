// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The clips a gesture or a typing session draws through until it commits.

use super::gesture::track;
use super::*;

/// The clips a gesture or a typing session draws through: the clips it began
/// on, with every annotation it holds written into its own clip. One with no
/// clip yet gets a provisional, unpaced one reaching back a draw-in, so it is
/// finished drawing at the playhead, under the hand, until the editor paces
/// it. `position` is the playhead in the recording and `timeline_ms` where it
/// plays on the timeline.
pub(super) fn provisional_clips(
  before: &[RecordingAnnotationClip],
  working: &[Annotation],
  pane: u32,
  (position, timeline_ms): (u64, u64),
  scenes: &screenwide_annotations::scene_shape::SceneEdit,
) -> Vec<RecordingAnnotationClip> {
  let reach_back = screenwide_annotations::reveal::REVEAL_DRAW_IN_MS as u64;
  let mut next = before.to_vec();
  for annotation in working {
    if let Some(clip) = next.iter_mut().find(|c| c.annotation.id == annotation.id) {
      // A pinned annotation was shown where its pin put it at `position`, and
      // one in a scene where the scene puts it.
      scenes.fold(clip, annotation, position);
    } else {
      next.push(RecordingAnnotationClip {
        path_ms: None,
        pin: None,
        annotation: annotation.clone(),
        track_id: track(pane),
        start_ms: timeline_ms.saturating_sub(reach_back),
        end_ms: timeline_ms.saturating_add(3000),
        recording_span: Some([
          position.saturating_sub(reach_back),
          position.saturating_add(3000),
        ]),
        scene_shapes: Vec::new(),
      });
    }
  }
  next
}
