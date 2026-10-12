// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;
use crate::{AnnotationShape, AnnotationStyle};

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
        tint: false,
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
    recording_span: None,
  }
}

#[test]
fn active_annotations_respects_intervals_and_tracks() {
  let clips = vec![
    clip("a", AnnotationTrack::Primary, 1_000, 2_000),
    clip("b", AnnotationTrack::Camera, 1_000, 2_000),
  ];
  assert_eq!(
    active_annotations(&clips, AnnotationTrack::Primary, &[], 999).len(),
    0
  );
  assert_eq!(
    active_annotations(&clips, AnnotationTrack::Primary, &[], 1_000)[0].id,
    "a"
  );
  assert_eq!(
    active_annotations(&clips, AnnotationTrack::Primary, &[], 2_000).len(),
    0
  );
  assert_eq!(
    active_annotations(&clips, AnnotationTrack::Camera, &[], 1_500)[0].id,
    "b"
  );
}

#[test]
fn active_annotations_no_longer_truncates_at_thirty_two() {
  let clips: Vec<_> = (0..40)
    .map(|index| clip(&index.to_string(), AnnotationTrack::Primary, 0, 1_000))
    .collect();
  assert_eq!(
    active_annotations(&clips, AnnotationTrack::Primary, &[], 500).len(),
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

/// 5-10 s of the recording plays first, then 0-5 s.
fn reordered() -> [TimelineRange; 2] {
  [
    kept(5_000, 10_000, 1.0),
    TimelineRange {
      output_start_us: 5_000_000,
      ..kept(0, 5_000, 1.0)
    },
  ]
}

/// The reveal of a clip `[start_ms, end_ms)` on the timeline, drawn on the
/// frame `source_ms` into the recording.
fn reveal(
  ranges: &[TimelineRange],
  [start_ms, end_ms]: [u64; 2],
  source_ms: u64,
) -> Option<crate::reveal::AnnotationReveal> {
  let clip = clip("a", AnnotationTrack::Primary, start_ms, end_ms);
  revealed_annotations(
    &[clip],
    AnnotationTrack::Primary,
    ranges,
    source_ms,
    0.0,
    (0, 0),
  )
  .first()
  .map(|annotation| annotation.reveal)
}

/// A frame is timed where it plays: 3 s into the recording is half a second
/// into a timeline that keeps 0-0.5 s and 3-20 s.
#[test]
fn a_frame_is_timed_where_it_plays_on_the_timeline() {
  let ranges = [
    kept(0, 500, 1.0),
    TimelineRange {
      output_start_us: 500_000,
      ..kept(3_000, 20_000, 1.0)
    },
  ];
  assert_eq!(reveal(&ranges, [500, 9_500], 3_000).unwrap().high, 0.0);
  assert!(reveal(&ranges, [500, 9_500], 3_500).unwrap().high < 1.0);
  assert_eq!(reveal(&ranges, [500, 9_500], 4_000).unwrap().high, 1.0);
  assert!(reveal(&ranges, [500, 9_500], 400).is_none());
}

/// A clip at either end of the video is whole there, with nothing before
/// it to arrive from or after it to leave for, and still animates at its
/// other end; one a millisecond short of an end still reaches it.
#[test]
fn a_clip_at_an_end_of_the_video_holds_there() {
  let ranges = [kept(0, 10_000, 1.0)];
  assert!(reveal(&ranges, [0, 5_000], 0).unwrap().is_whole());
  assert!(reveal(&ranges, [0, 5_000], 4_999).unwrap().low > 0.99);
  assert!(reveal(&ranges, [5_000, 10_000], 9_999).unwrap().is_whole());
  assert_eq!(reveal(&ranges, [5_000, 10_000], 5_000).unwrap().high, 0.0);
  assert_eq!(reaches_video_ends(Some(10_000), [1, 9_999]), (true, true));
  assert_eq!(reaches_video_ends(Some(10_000), [2, 9_998]), (false, false));
  assert_eq!(reaches_video_ends(None, [0, 10_000]), (false, false));
}

/// Arriving takes the same time on the timeline at any speed: on a timeline
/// playing twice as fast, it takes twice as much of the recording.
#[test]
fn a_faster_timeline_does_not_hurry_the_arrival() {
  let ranges = [kept(0, 20_000, 2.0)];
  assert!(reveal(&ranges, [1_000, 9_000], 3_000).unwrap().high < 1.0);
  assert_eq!(reveal(&ranges, [1_000, 9_000], 4_000).unwrap().high, 1.0);
}

/// A clip placed across a jump plays straight on across it: it holds both
/// sides, and its timing counts on from where it started.
#[test]
fn a_clip_plays_on_across_a_jump_in_the_recording() {
  let ranges = reordered();
  // 4-6 s on the timeline: 9-10 s of the recording, then 0-1 s.
  assert!(reveal(&ranges, [4_000, 6_000], 9_500).is_some());
  assert!(reveal(&ranges, [4_000, 6_000], 500).is_some());
  assert!(reveal(&ranges, [4_000, 6_000], 7_000).is_none());
  assert!(reveal(&ranges, [4_000, 6_000], 2_000).is_none());
  assert_eq!(progress([4_000, 6_000], 5_500), Some((1_500.0, 2_000.0)));
}

/// What follows the picture is read across the stretch of the recording a
/// clip plays, wherever the timeline has put it.
#[test]
fn a_clip_reads_the_recording_its_timeline_stretch_plays() {
  let mut clips = [
    clip("a", AnnotationTrack::Primary, 0, 1_000),
    clip("b", AnnotationTrack::Primary, 5_000, 7_000),
  ];
  lay_on_recording(&mut clips, &reordered());
  assert_eq!(clips[0].recording_span(), [5_000, 6_000]);
  assert_eq!(clips[1].recording_span(), [0, 2_000]);
  lay_on_recording(&mut clips, &[]);
  assert_eq!(clips[1].recording_span(), [5_000, 7_000]);
}

/// A moving or swaying image is timed from the start of its clip on the
/// timeline, and an image that neither plays nor sways carries no clock.
#[test]
fn a_moving_or_swaying_image_is_timed_from_the_start_of_its_clip() {
  use crate::image::{model::new_image, ImageArt, ImagePlay};
  let image = |play: Option<ImagePlay>, seed: Option<u32>| {
    let art = ImageArt {
      asset: "image:0123456789abcdef0123456789abcdef".to_owned(),
      aspect: 1.0,
      pixels: None,
      play,
    };
    let mut moving = clip("a", AnnotationTrack::Primary, 1_000, 10_000);
    moving.annotation = new_image("a".to_owned(), Default::default(), &art, None, 1.0);
    if let AnnotationShape::Image { sway, .. } = &mut moving.annotation.shape {
      *sway = seed;
    }
    let drawn = revealed_annotations(&[moving], AnnotationTrack::Primary, &[], 1_750, 0.0, (0, 0));
    match &drawn[0].shape {
      AnnotationShape::Image { clock_ms, .. } => *clock_ms,
      _ => unreachable!(),
    }
  };
  let play = ImagePlay {
    cycle_ms: 400.0,
    frames: 4,
    frame: 0,
    once: false,
  };
  assert_eq!(image(Some(play), None), Some(750.0));
  assert_eq!(image(None, Some(3)), Some(750.0));
  assert_eq!(image(None, None), None);
}
