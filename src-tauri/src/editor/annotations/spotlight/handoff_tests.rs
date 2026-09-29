// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::{glide_ms, spotlight_links};
use crate::editor::annotations::spotlight::model::new_spotlight;
use crate::editor::annotations::timing::{
  revealed_annotations, AnnotationTrack, RecordingAnnotationClip,
};
use crate::editor::annotations::{Annotation, AnnotationPoint, AnnotationShape};

fn spotlight(
  id: &str,
  [x0, y0, x1, y1]: [f64; 4],
  [start_ms, end_ms]: [u64; 2],
) -> RecordingAnnotationClip {
  let point = |x, y| AnnotationPoint { x, y };
  RecordingAnnotationClip {
    annotation: new_spotlight(id.to_owned(), [point(x0, y0), point(x1, y1)], None),
    track_id: AnnotationTrack::Primary,
    start_ms,
    end_ms,
    pin: None,
    path_ms: None,
  }
}

/// A picture whose diagonal is 2,000 pixels, so a glide of 400 - a fifth of
/// it - takes the second a path that long draws in over.
const PICTURE: (u32, u32) = (1_600, 1_200);

fn drawn(clips: &[RecordingAnnotationClip], at: u64) -> Vec<Annotation> {
  revealed_annotations(clips, AnnotationTrack::Primary, &[], at, 0.0, PICTURE)
}

fn left(annotation: &Annotation) -> f64 {
  match annotation.shape {
    AnnotationShape::Spotlight { start, .. } => start.x,
    _ => unreachable!(),
  }
}

const FIRST: [f64; 4] = [0.0, 0.0, 100.0, 100.0];
const SECOND: [f64; 4] = [400.0, 0.0, 500.0, 100.0];

#[test]
fn a_spotlight_butted_up_against_another_hands_its_light_on_without_fading() {
  let clips = [
    spotlight("a", FIRST, [0, 2_000]),
    spotlight("b", SECOND, [2_000, 6_000]),
  ];
  // Alone, the first would be all but gone by its last moment.
  let before = drawn(&clips, 1_999);
  assert_eq!(before.len(), 1);
  assert_eq!(before[0].reveal.opacity, 1.0);
  // The second is whole from its first moment, where the first box was,
  // and glides across to its own over the second its path takes.
  let at_join = drawn(&clips, 2_000);
  assert_eq!(at_join.len(), 1);
  assert_eq!(at_join[0].reveal.opacity, 1.0);
  assert_eq!(left(&at_join[0]), 0.0);
  // Already on its way a frame in, as a stroke sets off.
  assert!(left(&drawn(&clips, 2_016)[0]) > 0.0);
  let gliding = left(&drawn(&clips, 2_500)[0]);
  assert!(gliding > 0.0 && gliding < 400.0, "{gliding}");
  assert_eq!(left(&drawn(&clips, 3_000)[0]), 400.0);
  // Its own end fades as a lone spotlight's does.
  assert!(drawn(&clips, 5_999)[0].reveal.opacity < 0.01);
}

#[test]
fn a_longer_glide_takes_longer_as_a_longer_path_would_but_never_outstays_its_arrival() {
  let near = [
    spotlight("a", FIRST, [0, 2_000]),
    spotlight("b", SECOND, [2_000, 11_000]),
  ];
  let links = spotlight_links(&near);
  assert_eq!(glide_ms(&near, &links, 1, PICTURE, 9_000.0), Some(1_000.0));
  // Four times as far takes twice as long, which is as long as any path.
  let far = [
    spotlight("a", FIRST, [0, 2_000]),
    spotlight("b", [1_600.0, 0.0, 1_700.0, 100.0], [2_000, 11_000]),
  ];
  let far_links = spotlight_links(&far);
  assert_eq!(
    glide_ms(&far, &far_links, 1, PICTURE, 9_000.0),
    Some(2_000.0)
  );
  // A short clip glides over no more of itself than it would arrive over.
  assert_eq!(glide_ms(&near, &links, 1, PICTURE, 900.0), Some(300.0));
}

#[test]
fn a_frame_between_them_still_joins_and_the_first_holds_across_it() {
  let clips = [
    spotlight("a", FIRST, [0, 2_000]),
    spotlight("b", SECOND, [2_030, 4_000]),
  ];
  let between = drawn(&clips, 2_015);
  assert_eq!(between.len(), 1);
  assert_eq!(between[0].reveal.opacity, 1.0);
  // A longer gap is two lights, each fading on its own.
  let apart = [
    spotlight("a", FIRST, [0, 2_000]),
    spotlight("b", SECOND, [2_100, 4_000]),
  ];
  assert!(spotlight_links(&apart)
    .iter()
    .all(|link| link.from.is_none()));
  assert!(drawn(&apart, 2_050).is_empty());
}

#[test]
fn spotlights_side_by_side_or_either_side_of_the_camera_never_join() {
  let parallel = [
    spotlight("a", FIRST, [0, 2_000]),
    spotlight("b", SECOND, [500, 4_000]),
  ];
  assert!(spotlight_links(&parallel)
    .iter()
    .all(|link| link.onward.is_none()));
  let mut above = spotlight("b", SECOND, [2_000, 4_000]);
  above.annotation.above_camera = true;
  let split = [spotlight("a", FIRST, [0, 2_000]), above];
  assert!(spotlight_links(&split)
    .iter()
    .all(|link| link.onward.is_none()));
}

#[test]
fn where_two_end_as_one_starts_the_nearer_hands_on_and_the_other_fades() {
  let clips = [
    spotlight("far", FIRST, [0, 2_000]),
    spotlight("near", [300.0, 0.0, 400.0, 100.0], [0, 2_000]),
    spotlight("next", SECOND, [2_000, 4_000]),
  ];
  let links = spotlight_links(&clips);
  assert_eq!(links[0].onward, None);
  assert_eq!(links[1].onward, Some(2));
  assert_eq!(links[2].from, Some(1));
  // The far light fades out while the shade holds over the near one.
  let late = drawn(&clips, 1_999);
  assert!(late[0].reveal.opacity < 0.01);
  assert_eq!(late[1].reveal.opacity, 1.0);
}

#[test]
fn a_blur_only_one_side_has_fades_across_the_glide() {
  let mut first = spotlight("a", FIRST, [0, 2_000]);
  first.annotation.style.blur = true;
  let clips = [first, spotlight("b", SECOND, [2_000, 6_000])];
  let gliding = &drawn(&clips, 2_500)[0];
  assert!(gliding.style.blur);
  assert!(gliding.reveal.scale > 0.0 && gliding.reveal.scale < 1.0);
  let arrived = &drawn(&clips, 3_000)[0];
  assert!(!arrived.style.blur);
}
