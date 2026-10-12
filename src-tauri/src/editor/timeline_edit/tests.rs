// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;
use screenwide_timeline_edit::RecordingTimelineSegment;

fn edit(artifact_id: u64, split: f64) -> RecordingTimelineEdit {
  RecordingTimelineEdit {
    annotation_clips: Vec::new(),
    artifact_id,
    keyboard_deletions: Box::default(),
    silence_cuts: Vec::new(),
    transcript_cuts: Vec::new(),
    transcript_corrections: Vec::new(),
    transcript_allowed_words: Vec::new(),
    scene_clips: Vec::new(),
    next_segment_id: 2,
    segments: vec![
      RecordingTimelineSegment {
        id: 0,
        source_end: split,
        source_start: 0.0,
        playback_rate: 1.0,
      },
      RecordingTimelineSegment {
        id: 1,
        source_end: 1.0,
        source_start: split,
        playback_rate: 1.0,
      },
    ],
  }
}

#[test]
fn keeps_the_newest_save_and_rebinds_the_artifact() {
  let project = screenwide_project::scratch::scratch_project("timeline-newest");

  persist(&project, 7, 1, edit(7, 0.25)).unwrap();
  persist(&project, 7, 2, edit(7, 0.75)).unwrap();
  // A save that arrives late must not undo the one after it.
  persist(&project, 7, 1, edit(7, 0.5)).unwrap();
  let (revision, restored) = for_project(&project, 99, 8_000).unwrap();

  assert_eq!(revision, 2);
  assert_eq!(restored.artifact_id, 99);
  assert_eq!(restored.segments[0].source_end, 0.75);
  // Saving the edit leaves what was recorded alone.
  assert_eq!(
    screenwide_project::read(&project)
      .unwrap()
      .recorded()
      .unwrap()
      .media
      .primary,
    "media/recording.mov"
  );
}

#[test]
fn live_annotation_clips_become_the_recordings_first_edit() {
  use screenwide_annotations::arrow::new_arrow;
  use screenwide_annotations::timing::{AnnotationTrack, RecordingAnnotationClip};
  use screenwide_annotations::AnnotationPoint;

  let project = screenwide_project::scratch::scratch_project("timeline-initial");
  let clip = RecordingAnnotationClip {
    path_ms: None,
    pin: None,
    recording_span: None,
    annotation: new_arrow(
      "annotation".to_owned(),
      AnnotationPoint { x: 10.0, y: 20.0 },
      AnnotationPoint { x: 30.0, y: 40.0 },
      None,
    ),
    track_id: AnnotationTrack::Primary,
    start_ms: 0,
    end_ms: 8_000,
  };

  persist_initial_edit(&project, 7, vec![clip.clone()], Vec::new()).unwrap();
  let (revision, restored) = for_project(&project, 7, 8_000).unwrap();

  assert_eq!(revision, 0);
  assert_eq!(restored.annotation_clips, vec![clip]);
  // The editor's own first save lands on top instead of being refused.
  persist(&project, 7, 1, edit(7, 0.5)).unwrap();
  assert_eq!(for_project(&project, 7, 8_000).unwrap().0, 1);
  // Reopening the project does not put the first edit back over it.
  persist_initial_edit(&project, 8, Vec::new(), Vec::new()).unwrap();
  assert_eq!(for_project(&project, 8, 8_000).unwrap().1.segments.len(), 2);
}
