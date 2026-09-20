// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later
use super::*;
use crate::editor::annotations::{arrow::new_arrow, timing::AnnotationTrack, AnnotationPoint};

#[test]
fn queued_seek_keeps_its_annotation_timing_snapshot() {
  let clip = RecordingAnnotationClip {
    annotation: new_arrow(
      "arrow".into(),
      AnnotationPoint { x: 0.0, y: 0.0 },
      AnnotationPoint { x: 100.0, y: 0.0 },
      None,
    ),
    track_id: AnnotationTrack::Primary,
    start_ms: 0,
    end_ms: 2000,
  };
  let clips = Arc::new(RwLock::new(vec![clip]));
  let (sender, receiver) = mpsc::channel();
  let decoder = NativeStillDecoder {
    annotation_clips: Arc::clone(&clips),
    sender,
    thread: None,
  };
  decoder.seek(1999, 1, true, vec![]).unwrap();
  clips.write().unwrap()[0].end_ms = 3000;
  decoder.seek(2999, 2, true, vec![]).unwrap();
  for (position, end) in [(1999, 2000), (2999, 3000)] {
    let DecoderCommand::Seek {
      position_ms,
      annotation_clips,
      ..
    } = receiver.recv().unwrap()
    else {
      panic!("expected seek")
    };
    assert_eq!(position_ms, position);
    assert_eq!(annotation_clips[0].end_ms, end);
    assert!(position_ms < annotation_clips[0].end_ms);
  }
}
