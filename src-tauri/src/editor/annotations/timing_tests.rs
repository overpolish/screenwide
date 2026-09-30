// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;
use crate::editor::annotations::{AnnotationShape, AnnotationStyle};

fn clip(
  id: &str,
  track_id: AnnotationTrack,
  start_ms: u64,
  end_ms: u64,
) -> RecordingAnnotationClip {
  RecordingAnnotationClip {
    path_ms: None,
    annotation: Annotation {
      above_camera: false,
      animated: true,
      id: id.to_owned(),
      held: None,
      pen: false,
      reveal: Default::default(),
      shape: AnnotationShape::Arrow {
        start: Default::default(),
        control: Default::default(),
        end: Default::default(),
      },
      style: AnnotationStyle {
        align: Default::default(),
        blur: false,
        color: "#ff0000".to_owned(),
        head: Default::default(),
        hand_drawn: false,
        manual: false,
        radius: 0.0,
        redaction: Default::default(),
        shadow: false,
        softness: 0.0,
        strength: 0.0,
        width: 8.0,
      },
    },
    track_id,
    start_ms,
    end_ms,
    pin: None,
  }
}

#[test]
fn active_annotations_respects_intervals_and_tracks() {
  let clips = vec![
    clip("a", AnnotationTrack::Primary, 1_000, 2_000),
    clip("b", AnnotationTrack::Camera, 1_000, 2_000),
  ];
  assert_eq!(
    active_annotations(&clips, AnnotationTrack::Primary, 999).len(),
    0
  );
  assert_eq!(
    active_annotations(&clips, AnnotationTrack::Primary, 1_000)[0].id,
    "a"
  );
  assert_eq!(
    active_annotations(&clips, AnnotationTrack::Primary, 2_000).len(),
    0
  );
  assert_eq!(
    active_annotations(&clips, AnnotationTrack::Camera, 1_500)[0].id,
    "b"
  );
}

#[test]
fn active_annotations_no_longer_truncates_at_thirty_two() {
  let clips: Vec<_> = (0..40)
    .map(|index| clip(&index.to_string(), AnnotationTrack::Primary, 0, 1_000))
    .collect();
  assert_eq!(
    active_annotations(&clips, AnnotationTrack::Primary, 500).len(),
    40
  );
}

#[test]
fn invalid_clip_is_rejected() {
  assert!(validate_clips(&[clip("bad", AnnotationTrack::Primary, 2_000, 2_000)]).is_err());
}

/// A clip draws in over its own pace, and one the editor has not paced
/// over its kind's second.
#[test]
fn a_clip_draws_in_at_its_own_pace() {
  let drawn = |path_ms: Option<f32>, at: u64| {
    let mut paced = clip("a", AnnotationTrack::Primary, 0, 10_000);
    paced.path_ms = path_ms;
    revealed_annotations(&[paced], AnnotationTrack::Primary, &[], at, 0.0, (0, 0))[0]
      .reveal
      .high
  };
  assert_eq!(drawn(None, 1_000), 1.0);
  assert!(drawn(Some(2_000.0), 1_000) < 1.0);
  assert_eq!(drawn(Some(2_000.0), 2_000), 1.0);
  assert_eq!(drawn(Some(600.0), 600), 1.0);
}

fn kept(source_start_ms: u64, source_end_ms: u64, playback_rate: f64) -> TimelineRange {
  TimelineRange {
    output_start_us: 0,
    source_end_us: source_end_ms * 1_000,
    source_start_us: source_start_ms * 1_000,
    playback_rate,
  }
}

fn reveal_at(
  ranges: &[TimelineRange],
  at: u64,
) -> crate::editor::annotations::reveal::AnnotationReveal {
  let clip = clip("a", AnnotationTrack::Primary, 1_000, 10_000);
  revealed_annotations(&[clip], AnnotationTrack::Primary, ranges, at, 0.0, (0, 0))[0].reveal
}

/// A clip whose start the timeline trimmed away arrives from the first
/// frame it is seen, rather than already part way in.
#[test]
fn a_trimmed_start_arrives_where_the_clip_is_first_seen() {
  let ranges = [kept(3_000, 20_000, 1.0)];
  assert_eq!(reveal_at(&ranges, 3_000).high, 0.0);
  assert!(reveal_at(&ranges, 3_500).high < 1.0);
  assert_eq!(reveal_at(&ranges, 4_000).high, 1.0);
}

/// A clip whose end the timeline trimmed away has left by the last frame
/// that is kept, rather than being cut off whole.
#[test]
fn a_trimmed_end_leaves_before_the_cut() {
  let ranges = [kept(0, 6_000, 1.0)];
  assert!(reveal_at(&ranges, 5_999).low > 0.99);
  assert!(reveal_at(&[], 5_999).is_whole());
}

/// Arriving takes the same output time at any speed.
#[test]
fn a_faster_timeline_does_not_hurry_the_arrival() {
  let ranges = [kept(0, 20_000, 2.0)];
  assert!(reveal_at(&ranges, 2_000).high < 1.0);
  assert_eq!(reveal_at(&ranges, 3_000).high, 1.0);
}
