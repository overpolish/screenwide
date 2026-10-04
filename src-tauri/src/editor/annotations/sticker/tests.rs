// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::geometry::{prepare_sticker, sticker_distance};
use super::gesture::resized;
use super::model::{bounds, sticker_turn, MIN_STICKER_SIZE};
use crate::editor::annotations::box_gesture::{EDGE_BOTTOM, EDGE_LEFT, EDGE_RIGHT, EDGE_TOP};
use crate::editor::annotations::reveal::AnnotationReveal;
use crate::editor::annotations::{AnnotationPoint, AnnotationShape};

fn point(x: f64, y: f64) -> AnnotationPoint {
  AnnotationPoint { x, y }
}

#[test]
fn a_press_drops_the_chosen_picture_and_the_drag_carries_it() {
  use crate::editor::annotations::edit::{AnnotationEdit, FreshAnnotation};
  use crate::editor::annotations::gesture::AnnotationGestureTarget;
  use crate::editor::annotations::snap::SnapModifiers;
  use crate::editor::annotations::AnnotationKind;
  let fresh = FreshAnnotation {
    angle: Some(1.0),
    sticker: Some(super::StickerArt {
      asset: "emoji:🔥".to_owned(),
      aspect: 0.5,
      pixels: None,
      play: None,
    }),
  };
  let mut annotations = Vec::new();
  let mut edit = AnnotationEdit::begin(
    &mut annotations,
    AnnotationGestureTarget::New,
    point(100.0, 100.0),
    None,
    Some(AnnotationKind::Sticker),
    &fresh,
    2.0,
  )
  .expect("a fresh sticker");
  edit.update(
    &mut annotations,
    point(140.0, 120.0),
    SnapModifiers::default(),
    None,
  );
  // Twice the points of a capture at 2x, upright whatever a counter was
  // last turned to, showing the picture the editor chose, where the drag
  // carried it.
  assert_eq!(
    annotations[0].shape,
    AnnotationShape::Sticker {
      center: point(140.0, 120.0),
      size: 192.0,
      angle: 0.0,
      aspect: 0.5,
      flip: false,
      asset: "emoji:🔥".to_owned(),
      play: None,
    }
  );
  // Placed, it is let go, so the next sticker's picture can be chosen.
  assert_eq!(edit.chosen_id(), None);
}

#[test]
fn a_picture_of_its_own_lands_at_its_own_size_or_the_largest_that_fits() {
  use crate::editor::annotations::edit::FreshAnnotation;
  let art = |pixels: f64, aspect: f64| FreshAnnotation {
    angle: None,
    sticker: Some(super::StickerArt {
      asset: "image:0123456789abcdef0123456789abcdef".to_owned(),
      aspect,
      pixels: Some(pixels),
      play: None,
    }),
  };
  let size = |fresh: FreshAnnotation| {
    let fresh = fresh.within((1920, 1080));
    let sticker = super::model::new_sticker(
      "s".to_owned(),
      point(0.0, 0.0),
      fresh.sticker.as_ref(),
      None,
      2.0,
    );
    let AnnotationShape::Sticker { size, .. } = sticker.shape else {
      unreachable!()
    };
    size
  };
  // Small enough: its own pixels, not the emoji's 96 points.
  assert_eq!(size(art(300.0, 1.5)), 300.0);
  // A wide picture is held by the width, a tall one by the height.
  assert_eq!(size(art(4000.0, 2.0)), 1920.0);
  assert_eq!(size(art(4000.0, 0.5)), 1080.0);
  // A square one by the shorter side of the picture.
  assert_eq!(size(art(4000.0, 1.0)), 1080.0);
}

#[test]
fn a_document_sticker_reads_back_unmirrored_and_still_when_it_does_not_say() {
  let shape: AnnotationShape = serde_json::from_str(
    r#"{"kind":"sticker","center":{"x":10,"y":20},"size":64,"angle":0.5,"aspect":2,"asset":"emoji:👍"}"#,
  )
  .expect("a stored sticker");
  assert_eq!(
    shape,
    AnnotationShape::Sticker {
      center: point(10.0, 20.0),
      size: 64.0,
      angle: 0.5,
      aspect: 2.0,
      flip: false,
      asset: "emoji:👍".to_owned(),
      play: None,
    }
  );
  assert!(shape.placed());
}

#[test]
fn an_animated_sticker_keeps_how_it_plays_but_never_its_clock() {
  let mut shape: AnnotationShape = serde_json::from_str(
    r#"{"kind":"sticker","center":{"x":0,"y":0},"size":64,"angle":0,"aspect":1,"asset":"image:x","play":{"cycleMs":400,"frames":3,"frame":1,"once":true}}"#,
  )
  .expect("a stored sticker");
  let AnnotationShape::Sticker {
    play: Some(play), ..
  } = &mut shape
  else {
    panic!("no play");
  };
  assert_eq!(
    (play.cycle_ms, play.frames, play.frame, play.once),
    (400.0, 3, 1, true)
  );
  play.clock_ms = Some(120.0);
  let written = serde_json::to_string(&shape).expect("a written sticker");
  assert!(
    written.contains(r#""play":{"cycleMs":400.0,"frames":3,"frame":1,"once":true}"#),
    "{written}"
  );
}

#[test]
fn a_turned_sticker_fits_the_box_its_grips_stand_on() {
  // A 2:1 picture 100 long, a quarter turn round: 50 wide and 100 tall.
  let (low, high) = bounds(point(0.0, 0.0), 100.0, std::f64::consts::FRAC_PI_2, 2.0);
  assert!((high.x - low.x - 50.0).abs() < 1e-9, "{low:?} {high:?}");
  assert!((high.y - low.y - 100.0).abs() < 1e-9, "{low:?} {high:?}");
}

#[test]
fn a_corner_sizes_the_picture_from_the_opposite_corner() {
  let (low, high) = (point(0.0, 0.0), point(100.0, 50.0));
  // The bottom-right corner dragged out to twice the width, less far down:
  // the larger ask wins, so the box reaches the hand.
  let (center, size) = resized(
    low,
    high,
    100.0,
    EDGE_RIGHT | EDGE_BOTTOM,
    point(200.0, 60.0),
  );
  assert_eq!(size, 200.0);
  assert_eq!(center, point(100.0, 50.0));
  // The top-left one holds the bottom-right corner.
  let (center, size) = resized(low, high, 100.0, EDGE_LEFT | EDGE_TOP, point(50.0, 25.0));
  assert_eq!(size, 50.0);
  assert_eq!(center, point(75.0, 37.5));
}

#[test]
fn a_side_dragged_past_its_opposite_stops_at_the_smallest_sticker() {
  let (low, high) = (point(0.0, 0.0), point(100.0, 100.0));
  let (center, size) = resized(low, high, 100.0, EDGE_LEFT, point(150.0, 50.0));
  assert_eq!(size, MIN_STICKER_SIZE);
  // An edge leaves the other axis centred where it was.
  assert_eq!(center, point(100.0 - MIN_STICKER_SIZE / 2.0, 50.0));
}

#[test]
fn the_turning_grip_stands_the_top_towards_the_hand() {
  let center = point(0.0, 0.0);
  assert_eq!(sticker_turn(center, point(0.0, -10.0), 1.0, false), 0.0);
  let right = sticker_turn(center, point(10.0, 0.0), 0.0, false);
  assert!((right - std::f64::consts::FRAC_PI_2).abs() < 1e-9);
  // Shift holds it to the eighth turns, a little off a diagonal included.
  let snapped = sticker_turn(center, point(10.0, -9.0), 0.0, true);
  assert!((snapped - std::f64::consts::FRAC_PI_4).abs() < 1e-9);
  // A press on the middle keeps the turn it had.
  assert_eq!(sticker_turn(center, center, 0.7, false), 0.7);
}

#[test]
fn a_turned_sticker_is_picked_inside_its_turned_box_only() {
  // A picture 40 wide and 20 tall, turned a quarter round: 20 wide and 40
  // tall on the canvas.
  let geometry = prepare_sticker(
    [100.0, 100.0],
    [100.0, 120.0],
    [90.0, 100.0],
    0.0,
    false,
    AnnotationReveal::WHOLE,
  );
  assert!(sticker_distance([100.0, 115.0], &geometry) <= 0.0);
  assert!(sticker_distance([115.0, 100.0], &geometry) > 0.0);
}

#[test]
fn an_arriving_sticker_grows_out_of_its_middle() {
  let reveal = AnnotationReveal {
    scale: 0.5,
    ..AnnotationReveal::WHOLE
  };
  let geometry = prepare_sticker([0.0, 0.0], [40.0, 0.0], [0.0, 20.0], 0.0, false, reveal);
  assert_eq!(geometry.b, [20.0, 0.0]);
  assert_eq!(geometry.c, [0.0, 10.0]);
  assert_eq!(geometry.width, 40.0);
}
