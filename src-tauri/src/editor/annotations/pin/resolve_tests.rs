// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;
use crate::editor::annotations::pin::model::{AnnotationPin, PinKeyframe};
use crate::editor::annotations::timing::{placed_annotation, AnnotationTrack};
use crate::editor::annotations::AnnotationStyle;

fn point(x: f64, y: f64) -> AnnotationPoint {
  AnnotationPoint { x, y }
}

fn clip(shape: AnnotationShape, keyframes: &[(u64, f64, f64)]) -> RecordingAnnotationClip {
  RecordingAnnotationClip {
    annotation: Annotation {
      above_camera: false,
      animated: false,
      id: "a".to_owned(),
      held: None,
      reveal: Default::default(),
      shape,
      style: AnnotationStyle {
        align: Default::default(),
        color: "#ff0000".to_owned(),
        head: Default::default(),
        radius: 0.0,
        redaction: Default::default(),
        strength: 0.0,
        width: 8.0,
      },
    },
    track_id: AnnotationTrack::Primary,
    start_ms: 0,
    end_ms: 5_000,
    pin: Some(AnnotationPin {
      pinned_ms: keyframes[0].0,
      keyframes: keyframes
        .iter()
        .map(|&(ms, dx, dy)| PinKeyframe { ms, dx, dy })
        .collect(),
      path: None,
    }),
  }
}

fn counter(x: f64, y: f64) -> AnnotationShape {
  AnnotationShape::Counter {
    center: point(x, y),
    value: 1,
    angle: 0.0,
  }
}

fn shown_at(clip: &RecordingAnnotationClip, ms: u64) -> Annotation {
  placed_annotation(clip, ms).expect("shown").0
}

#[test]
fn dragging_a_pinned_annotation_between_keyframes_adds_one_where_it_was_left() {
  let mut pinned = clip(counter(100.0, 100.0), &[(1_000, 0.0, 0.0)]);
  let mut moved = shown_at(&pinned, 2_000);
  moved.shape = counter(130.0, 90.0);
  fold(&mut pinned, &moved, 2_000);
  // What was drawn stays; the keyframe says where it is at that moment.
  assert_eq!(pinned.annotation.shape, counter(100.0, 100.0));
  let pin = pinned.pin.as_ref().expect("pinned");
  assert_eq!(
    pin.keyframes,
    vec![
      PinKeyframe {
        ms: 1_000,
        dx: 0.0,
        dy: 0.0
      },
      PinKeyframe {
        ms: 2_000,
        dx: 30.0,
        dy: -10.0
      },
    ]
  );
  assert_eq!(shown_at(&pinned, 2_000).shape, counter(130.0, 90.0));
}

#[test]
fn dragging_on_a_keyframe_moves_that_keyframe() {
  let mut pinned = clip(
    counter(100.0, 100.0),
    &[(1_000, 0.0, 0.0), (3_000, 20.0, 0.0)],
  );
  let mut moved = shown_at(&pinned, 3_004);
  assert_eq!(moved.shape, counter(120.0, 100.0));
  moved.shape = counter(125.0, 104.0);
  fold(&mut pinned, &moved, 3_004);
  let keyframes = &pinned.pin.as_ref().expect("pinned").keyframes;
  assert_eq!(keyframes.len(), 2);
  assert_eq!(
    keyframes[1],
    PinKeyframe {
      ms: 3_000,
      dx: 25.0,
      dy: 4.0
    }
  );
}

#[test]
fn reshaping_a_pinned_annotation_changes_what_was_drawn_and_no_keyframe() {
  let arrow = |end: AnnotationPoint| AnnotationShape::Arrow {
    start: point(0.0, 0.0),
    control: point(50.0, 0.0),
    end,
  };
  let mut pinned = clip(arrow(point(100.0, 0.0)), &[(1_000, 10.0, 20.0)]);
  let mut reshaped = shown_at(&pinned, 1_000);
  // Only the tip's grip moved, to 140, 20 on the picture.
  reshaped.shape = AnnotationShape::Arrow {
    start: point(10.0, 20.0),
    control: point(60.0, 20.0),
    end: point(140.0, 20.0),
  };
  fold(&mut pinned, &reshaped, 1_000);
  assert_eq!(pinned.annotation.shape, arrow(point(130.0, 0.0)));
  assert_eq!(pinned.pin.as_ref().expect("pinned").keyframes.len(), 1);
}

#[test]
fn a_pinned_annotation_is_hidden_while_its_content_is_off_the_frame() {
  let mut pinned = clip(counter(100.0, 100.0), &[(0, 0.0, 0.0)]);
  let sample = |ms, visible| PinSample {
    ms,
    dx: 0.0,
    dy: 0.0,
    scale: 1.0,
    confidence: 1.0,
    visible,
  };
  pinned.pin.as_mut().expect("pinned").path = Some(std::sync::Arc::new(PinnedPath {
    samples: vec![sample(0, true), sample(1_000, false), sample(2_000, true)],
    shown: vec![[0, 1_000], [2_000, 5_000]],
    ..PinnedPath::default()
  }));
  assert!(placed_annotation(&pinned, 500).is_some());
  assert!(placed_annotation(&pinned, 1_500).is_none());
  // Back on the frame, it arrives again over the rest of its clip.
  assert_eq!(
    placed_annotation(&pinned, 2_500).map(|(_, span)| span),
    Some([2_000, 5_000])
  );
}

#[test]
fn content_back_too_late_for_an_arrival_and_leaving_stays_hidden() {
  let mut pinned = clip(counter(100.0, 100.0), &[(0, 0.0, 0.0)]);
  pinned.annotation.animated = true;
  let sample = |ms| PinSample {
    ms,
    dx: 0.0,
    dy: 0.0,
    scale: 1.0,
    confidence: 1.0,
    visible: true,
  };
  pinned.pin.as_mut().expect("pinned").path = Some(std::sync::Arc::new(PinnedPath {
    samples: vec![sample(0), sample(4_700)],
    shown: vec![[0, 1_000], [4_700, 5_000]],
    ..PinnedPath::default()
  }));
  // A counter needs 740 ms to arrive and leave; 300 are left.
  assert!(placed_annotation(&pinned, 4_800).is_none());
}
