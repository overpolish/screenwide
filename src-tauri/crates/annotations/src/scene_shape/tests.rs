// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;
use crate::timing::{active_annotations, AnnotationTrack};
use crate::{AnnotationPoint, AnnotationStyle};

fn point(x: f64, y: f64) -> AnnotationPoint {
  AnnotationPoint { x, y }
}

fn arrow(x: f64) -> AnnotationShape {
  AnnotationShape::Arrow {
    start: point(x, 0.0),
    control: point(x + 50.0, 0.0),
    end: point(x + 100.0, 0.0),
  }
}

fn clip(shape: AnnotationShape) -> RecordingAnnotationClip {
  RecordingAnnotationClip {
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
    end_ms: 20_000,
    pin: None,
    path_ms: None,
    recording_span: None,
    scene_shapes: Vec::new(),
  }
}

fn scene(id: &str, start_ms: u64, end_ms: u64) -> RecordingSceneClip {
  serde_json::from_value(serde_json::json!({
    "id": id, "startMs": start_ms, "endMs": end_ms, "preset": "screen-only"
  }))
  .unwrap()
}

/// Where the arrow starts across, drawn `at` ms into an uncut recording on
/// a picture of no size, where every glide takes the scene's own time.
fn start_x(clip: &RecordingAnnotationClip, scenes: &[RecordingSceneClip], at: u64) -> f64 {
  start_x_on(clip, scenes, at, (0, 0))
}

/// [`start_x`] on a picture `picture` source pixels in size.
fn start_x_on(
  clip: &RecordingAnnotationClip,
  scenes: &[RecordingSceneClip],
  at: u64,
  picture: (u32, u32),
) -> f64 {
  let drawn = active_annotations(
    std::slice::from_ref(clip),
    scenes,
    AnnotationTrack::Primary,
    &[],
    at,
    picture,
  );
  match &drawn[0].shape {
    AnnotationShape::Arrow { start, .. } => start.x,
    _ => unreachable!(),
  }
}

#[test]
fn an_annotation_glides_to_its_place_in_a_scene_and_back() {
  let scenes = [scene("s", 4_000, 10_000)];
  let mut placed = clip(arrow(0.0));
  placed.scene_shapes.push(SceneShape {
    scene_id: "s".to_owned(),
    shape: arrow(600.0),
  });
  assert_eq!(start_x(&placed, &scenes, 2_000), 0.0);
  // Arriving over the scene's first 600 ms, eased, as the picture does.
  let arriving = start_x(&placed, &scenes, 4_300);
  assert!(arriving > 0.0 && arriving < 600.0, "{arriving}");
  assert_eq!(start_x(&placed, &scenes, 7_000), 600.0);
  let leaving = start_x(&placed, &scenes, 9_700);
  assert!(leaving > 0.0 && leaving < 600.0, "{leaving}");
  assert_eq!(start_x(&placed, &scenes, 12_000), 0.0);
}

#[test]
fn butting_scenes_hand_an_annotation_straight_from_one_place_to_the_next() {
  let scenes = [scene("one", 2_000, 6_000), scene("two", 6_000, 12_000)];
  let mut placed = clip(arrow(0.0));
  for (scene_id, x) in [("one", 300.0), ("two", 900.0)] {
    placed.scene_shapes.push(SceneShape {
      scene_id: scene_id.to_owned(),
      shape: arrow(x),
    });
  }
  // The first scene does not leave for the composition before handing on.
  assert_eq!(start_x(&placed, &scenes, 5_900), 300.0);
  let handing = start_x(&placed, &scenes, 6_300);
  assert!(handing > 300.0 && handing < 900.0, "{handing}");
  // A scene where it has no place of its own takes it back to its own.
  let scenes = [scene("one", 2_000, 6_000), scene("three", 6_000, 12_000)];
  let back = start_x(&placed, &scenes, 6_300);
  assert!(back > 0.0 && back < 300.0, "{back}");
}

#[test]
fn an_annotation_that_appears_inside_its_scene_appears_in_its_place_there() {
  let scenes = [scene("s", 4_000, 10_000)];
  let mut placed = clip(arrow(0.0));
  (placed.start_ms, placed.end_ms) = (4_200, 9_900);
  placed.scene_shapes.push(SceneShape {
    scene_id: "s".to_owned(),
    shape: arrow(600.0),
  });
  // Within what would be the scene's arrival and leaving, it stays put.
  assert_eq!(start_x(&placed, &scenes, 4_300), 600.0);
  assert_eq!(start_x(&placed, &scenes, 9_800), 600.0);
}

#[test]
fn a_pinned_annotation_follows_its_content_not_the_scenes() {
  let mut pinned = clip(arrow(0.0));
  pinned.scene_shapes.push(SceneShape {
    scene_id: "s".to_owned(),
    shape: arrow(600.0),
  });
  pinned.pin = Some(
    serde_json::from_value(serde_json::json!({
      "pinnedMs": 0, "keyframes": [{ "ms": 0, "dx": 0.0, "dy": 0.0 }]
    }))
    .unwrap(),
  );
  let scenes = [scene("s", 0, 10_000)];
  let moment = SceneMoment {
    timeline_ms: 5_000,
    video_ms: None,
    picture: (1_000, 1_000),
  };
  assert_eq!(pinned.shape_in_scenes(&scenes, moment), None);
}

#[test]
fn a_longer_move_takes_longer_up_to_a_limit() {
  let scenes = [scene("s", 4_000, 10_000)];
  let placed = |x: f64| {
    let mut placed = clip(arrow(0.0));
    placed.scene_shapes.push(SceneShape {
      scene_id: "s".to_owned(),
      shape: arrow(x),
    });
    placed
  };
  let picture = (1_000, 1_000);
  // A short move arrives in the scene's own 600 ms.
  assert_eq!(start_x_on(&placed(100.0), &scenes, 4_700, picture), 100.0);
  // Most of the picture's diagonal takes longer: still on its way.
  let far = start_x_on(&placed(1_300.0), &scenes, 4_700, picture);
  assert!(far > 0.0 && far < 1_300.0, "{far}");
  assert!(start_x_on(&placed(1_300.0), &scenes, 9_200, picture) < 1_300.0);
  assert_eq!(start_x_on(&placed(100.0), &scenes, 9_200, picture), 100.0);
  // However far, it takes no longer than a second and a half.
  let farthest = placed(10_000.0);
  assert!(start_x_on(&farthest, &scenes, 5_400, picture) < 10_000.0);
  assert_eq!(start_x_on(&farthest, &scenes, 5_600, picture), 10_000.0);
}

#[test]
fn a_move_in_a_scene_is_that_scenes_and_its_words_stay_the_annotations() {
  let scenes = vec![scene("s", 4_000, 10_000)];
  let edit = SceneEdit::at(scenes.clone(), &[], 7_000, [(0, 0); 2]);
  let mut counter = clip(AnnotationShape::Counter {
    center: point(10.0, 10.0),
    value: 1,
    angle: 0.0,
  });
  let mut moved = counter.annotation.clone();
  moved.shape = AnnotationShape::Counter {
    center: point(400.0, 300.0),
    value: 1,
    angle: 1.0,
  };
  edit.fold(&mut counter, &moved, 7_000);
  // Outside the scene it stays where it was drawn.
  assert_eq!(
    counter.annotation.shape,
    AnnotationShape::Counter {
      center: point(10.0, 10.0),
      value: 1,
      angle: 0.0,
    }
  );
  assert_eq!(counter.shape_in_scene("s"), Some(moved.shape.clone()));
  // Renumbered anywhere, it is renumbered in the scene too.
  counter.annotation.shape = AnnotationShape::Counter {
    center: point(10.0, 10.0),
    value: 4,
    angle: 0.0,
  };
  assert!(matches!(
    counter.shape_in_scene("s"),
    Some(AnnotationShape::Counter { value: 4, center, .. }) if center == point(400.0, 300.0)
  ));
}

#[test]
fn an_edit_in_a_scene_that_leaves_it_where_it_is_gives_the_scene_nothing() {
  let edit = SceneEdit::at(vec![scene("s", 0, 10_000)], &[], 5_000, [(0, 0); 2]);
  let mut arrow_clip = clip(arrow(0.0));
  let mut recoloured = arrow_clip.annotation.clone();
  recoloured.style.color = "#00ff00".to_owned();
  edit.fold(&mut arrow_clip, &recoloured, 5_000);
  assert!(arrow_clip.scene_shapes.is_empty());
  assert_eq!(arrow_clip.annotation.style.color, "#00ff00");
}

#[test]
fn a_counters_tail_turns_the_short_way_round() {
  let at = |angle: f64| AnnotationShape::Counter {
    center: point(0.0, 0.0),
    value: 1,
    angle,
  };
  let halfway = at(0.1).toward(&at(std::f64::consts::TAU - 0.1), 0.5);
  match halfway {
    AnnotationShape::Counter { angle, .. } => assert!(angle.abs() < 1e-9, "{angle}"),
    _ => unreachable!(),
  }
}

#[test]
fn a_glide_takes_as_long_as_its_twin_on_the_strip_says() {
  // The same case as `recording-annotation-glides.test.ts`.
  let ms = glide::glide_ms(&arrow(0.0), &arrow(1_300.0), (1_000, 1_000));
  assert!((ms - 1_150.5).abs() < 0.5, "{ms}");
}
