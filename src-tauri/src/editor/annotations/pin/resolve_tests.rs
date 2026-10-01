// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;
use crate::editor::annotations::pin::fold;
use crate::editor::annotations::pin::model::{AnnotationPin, PinKeyframe};
use crate::editor::annotations::timing::{placed_annotation, AnnotationTrack};
use crate::editor::annotations::AnnotationStyle;

fn point(x: f64, y: f64) -> AnnotationPoint {
  AnnotationPoint { x, y }
}

fn clip(shape: AnnotationShape, keyframes: &[(u64, f64, f64)]) -> RecordingAnnotationClip {
  RecordingAnnotationClip {
    path_ms: None,
    annotation: Annotation {
      above_camera: false,
      animated: false,
      id: "a".to_owned(),
      held: None,
      pen: false,
      reveal: Default::default(),
      shape,
      style: AnnotationStyle {
        align: Default::default(),
        blur: false,
        color: "#ff0000".to_owned(),
        head: Default::default(),
        hand_drawn: false,
        manual: false,
        tint: false,
        radius: 0.0,
        redaction: Default::default(),
        shadow: false,
        softness: 0.0,
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
        .map(|&(ms, dx, dy)| PinKeyframe {
          ms,
          dx,
          dy,
          edges: None,
          out_of_view: false,
        })
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
        dy: 0.0,
        edges: None,
        out_of_view: false,
      },
      PinKeyframe {
        ms: 2_000,
        dx: 30.0,
        dy: -10.0,
        edges: None,
        out_of_view: false,
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
      dy: 4.0,
      edges: None,
      out_of_view: false,
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

#[test]
fn show_here_puts_an_annotation_back_where_it_was_last_shown() {
  let sample = |ms, dy, visible| PinSample {
    ms,
    dx: 1.0,
    dy,
    scale: 1.0,
    confidence: if visible { 1.0 } else { 0.0 },
    visible,
  };
  // Shown while its content scrolls, then hidden under something while the
  // path bridged on wherever it liked, then shown again.
  let path = PinnedPath {
    samples: vec![
      sample(0, 0.0, true),
      sample(100, -20.0, true),
      sample(200, -90.0, false),
      sample(300, -140.0, false),
      sample(400, -30.0, true),
    ],
    covered: vec![[200, 400]],
    ..PinnedPath::default()
  };
  assert_eq!(path.last_seen(250), Some([1.0, -20.0]));
  assert_eq!(path.last_seen(399), Some([1.0, -20.0]));
  // Only where tracking hid it: shown, or past the stretch, there is nothing
  // to bring back.
  assert_eq!(path.last_seen(150), None);
  assert_eq!(path.last_seen(400), None);
}

/// A redaction pinned at 0 ms, followed 30 across and 40 up by one second,
/// where its content is doubtful and it is grown far more above than below.
fn grown_redaction() -> RecordingAnnotationClip {
  let mut pinned = clip(
    AnnotationShape::Redact {
      start: point(100.0, 100.0),
      end: point(200.0, 150.0),
      seed: 1,
    },
    &[(0, 0.0, 0.0)],
  );
  let sample = |ms, dx, dy| PinSample {
    ms,
    dx,
    dy,
    scale: 1.0,
    confidence: 1.0,
    visible: true,
  };
  pinned.pin.as_mut().expect("pinned").path = Some(std::sync::Arc::new(PinnedPath {
    samples: vec![sample(0, 0.0, 0.0), sample(1_000, 30.0, -40.0)],
    growth: vec![[0.0; 4], [5.0, 100.0, 5.0, 20.0]],
    shown: vec![[0, 5_000]],
    ..PinnedPath::default()
  }));
  pinned
}

/// `clip` shown at `ms` with its top edge dragged `by` pixels down.
fn top_dragged(clip: &RecordingAnnotationClip, ms: u64, by: f64) -> Annotation {
  let mut shown = shown_at(clip, ms);
  if let AnnotationShape::Redact { start, .. } = &mut shown.shape {
    start.y += by;
  }
  shown
}

fn redact(x0: f64, y0: f64, x1: f64, y1: f64) -> AnnotationShape {
  AnnotationShape::Redact {
    start: point(x0, y0),
    end: point(x1, y1),
    seed: 1,
  }
}

#[test]
fn resizing_a_redaction_away_from_its_pinned_frame_resizes_it_there_only() {
  let mut pinned = grown_redaction();
  // Dragged 110 down on the grown box: 10 down on the box itself.
  let shown = top_dragged(&pinned, 1_000, 110.0);
  fold(&mut pinned, &shown, 1_000);
  // The box as drawn stays; the resize is a keyframe where it was made.
  assert_eq!(pinned.annotation.shape, redact(100.0, 100.0, 200.0, 150.0));
  let keyframe = pinned
    .pin
    .as_ref()
    .expect("pinned")
    .keyframe_at(1_000)
    .copied();
  assert_eq!(
    keyframe.map(|keyframe| (keyframe.dx, keyframe.dy, keyframe.edges)),
    Some((30.0, -40.0, Some([0.0, -10.0, 0.0, 0.0])))
  );
  // Exactly that size there, ungrown, and as drawn on the pinned frame.
  assert_eq!(
    shown_at(&pinned, 1_000).shape,
    redact(130.0, 70.0, 230.0, 110.0)
  );
  assert_eq!(
    shown_at(&pinned, 0).shape,
    redact(100.0, 100.0, 200.0, 150.0)
  );
}

#[test]
fn a_redaction_s_size_eases_only_from_one_resize_to_the_next() {
  let mut pinned = clip(
    redact(100.0, 100.0, 200.0, 150.0),
    &[
      (0, 0.0, 0.0),
      (2_000, 0.0, 0.0),
      (3_000, 0.0, 0.0),
      (4_000, 0.0, 0.0),
    ],
  );
  let pin = pinned.pin.as_mut().expect("pinned");
  pin.keyframes[1].edges = Some([10.0, 0.0, 10.0, 0.0]);
  pin.keyframes[3].edges = Some([10.0, 0.0, 10.0, -20.0]);
  let pin = pinned.pin.as_ref().expect("pinned");
  // The size it was drawn at until the first resize, not creeping towards it.
  assert_eq!(pin.edges_at(1_000), [0.0; 4]);
  assert_eq!(pin.edges_at(2_000), [10.0, 0.0, 10.0, 0.0]);
  // A keyframe that only moved it sits on the way between two resizes.
  assert_eq!(pin.edges_at(3_000), [10.0, 0.0, 10.0, -10.0]);
  assert_eq!(pin.edges_at(4_500), [10.0, 0.0, 10.0, -20.0]);
}

#[test]
fn resizing_a_redaction_on_its_pinned_frame_resizes_the_box() {
  let mut pinned = grown_redaction();
  let shown = top_dragged(&pinned, 0, 10.0);
  fold(&mut pinned, &shown, 0);
  assert_eq!(pinned.annotation.shape, redact(100.0, 110.0, 200.0, 150.0));
  assert_eq!(pinned.pin.as_ref().expect("pinned").keyframes.len(), 1);
}

#[test]
fn an_annotation_said_to_be_out_of_view_is_hidden_until_the_next_keyframe() {
  let mut pinned = clip(
    counter(100.0, 100.0),
    &[(0, 0.0, 0.0), (1_000, 0.0, 0.0), (3_000, 5.0, 0.0)],
  );
  pinned.pin.as_mut().expect("pinned").keyframes[1].out_of_view = true;
  assert!(placed_annotation(&pinned, 500).is_some());
  assert!(placed_annotation(&pinned, 1_000).is_none());
  assert!(placed_annotation(&pinned, 2_990).is_none());
  assert!(placed_annotation(&pinned, 3_000).is_some());
}

#[test]
fn a_redaction_is_trimmed_clear_of_a_cover_and_a_resize_there_keeps_its_own_size() {
  let mut pinned = clip(redact(100.0, 100.0, 200.0, 150.0), &[(0, 0.0, 0.0)]);
  let sample = |ms, dy| PinSample {
    ms,
    dx: 0.0,
    dy,
    scale: 1.0,
    confidence: 1.0,
    visible: true,
  };
  // Scrolling up under a cover whose edge is at 80.
  let under = [f32::NEG_INFINITY, 80.0, f32::INFINITY, f32::INFINITY];
  pinned.pin.as_mut().expect("pinned").path = Some(std::sync::Arc::new(PinnedPath {
    samples: vec![sample(0, 0.0), sample(1_000, -40.0), sample(2_000, -80.0)],
    views: vec![under, under, under],
    shown: vec![[0, 5_000]],
    ..PinnedPath::default()
  }));
  assert_eq!(
    shown_at(&pinned, 1_000).shape,
    redact(100.0, 80.0, 200.0, 110.0)
  );
  // Wholly under, it closes up against the cover's edge.
  assert_eq!(
    shown_at(&pinned, 2_000).shape,
    redact(100.0, 80.0, 200.0, 80.0)
  );
  // Its bottom dragged 10 up where it is trimmed: the top it was trimmed to
  // is the cover's, not the hand's, so the resize is the bottom's alone.
  let mut shown = shown_at(&pinned, 1_000);
  if let AnnotationShape::Redact { end, .. } = &mut shown.shape {
    end.y -= 10.0;
  }
  fold(&mut pinned, &shown, 1_000);
  let pin = pinned.pin.as_ref().expect("pinned");
  assert_eq!(
    pin.keyframe_at(1_000).and_then(|keyframe| keyframe.edges),
    Some([0.0, 0.0, 0.0, -10.0])
  );
  assert_eq!(
    shown_at(&pinned, 1_000).shape,
    redact(100.0, 80.0, 200.0, 100.0)
  );
}
