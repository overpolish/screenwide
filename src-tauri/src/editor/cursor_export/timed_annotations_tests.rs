// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;
use crate::editor::annotations::{AnnotationPoint, AnnotationShape, AnnotationStyle};

fn counter_clip(value: u32) -> RecordingAnnotationClip {
  RecordingAnnotationClip {
    path_ms: None,
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
        blur: false,
        color: "#ffcc00".to_owned(),
        head: Default::default(),
        hand_drawn: false,
        manual: false,
        radius: 0.0,
        redaction: Default::default(),
        softness: 0.0,
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
  let (packed, data) = pack_clips(&clips, &[], 1.0, RedactSource::None, (0, 0));
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
      edges: None,
      out_of_view: false,
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
  let (packed, _) = pack_clips(
    std::slice::from_ref(&clip),
    &[],
    1.0,
    RedactSource::None,
    (0, 0),
  );
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

/// A clip whose start was trimmed away arrives from the first kept frame,
/// and one played at twice the speed arrives over the same output time.
#[test]
fn reveal_bounds_are_where_the_timeline_keeps_the_clip() {
  let range = |source_start_us, source_end_us, output_start_us, playback_rate| TimelineRange {
    output_start_us,
    source_end_us,
    source_start_us,
    playback_rate,
  };
  let ranges = [
    range(0, 500_000, 0, 1.0),
    range(1_500_000, 3_500_000, 500_000, 2.0),
  ];
  let (packed, _) = pack_clips(&[counter_clip(1)], &ranges, 1.0, RedactSource::None, (0, 0));
  assert_eq!(
    (
      packed[0].start_ms,
      packed[0].reveal_start_ms,
      packed[0].reveal_end_ms
    ),
    (1_000, 500, 750)
  );
}
