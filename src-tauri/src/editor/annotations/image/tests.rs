// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::geometry::{image_distance, prepare_image};
use super::gesture::resized;
use super::model::{bounds, image_turn, MIN_IMAGE_SIZE};
use crate::editor::annotations::box_gesture::{EDGE_BOTTOM, EDGE_LEFT, EDGE_RIGHT, EDGE_TOP};
use crate::editor::annotations::reveal::AnnotationReveal;
use crate::editor::annotations::{AnnotationPoint, AnnotationShape};

const PICTURE: &str = "image:0123456789abcdef0123456789abcdef";

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
    image: Some(super::ImageArt {
      asset: PICTURE.to_owned(),
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
    Some(AnnotationKind::Image),
    &fresh,
    2.0,
  )
  .expect("a fresh image");
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
    AnnotationShape::Image {
      center: point(140.0, 120.0),
      size: 192.0,
      angle: 0.0,
      aspect: 0.5,
      flip: false,
      asset: PICTURE.to_owned(),
      play: None,
      sway: None,
      clock_ms: None,
    }
  );
  // Placed, it is let go, so the next image's picture can be chosen.
  assert_eq!(edit.chosen_id(), None);
}

#[test]
fn a_press_before_any_picture_is_chosen_places_nothing() {
  use crate::editor::annotations::edit::{AnnotationEdit, FreshAnnotation};
  use crate::editor::annotations::gesture::AnnotationGestureTarget;
  use crate::editor::annotations::AnnotationKind;
  let fresh = FreshAnnotation {
    angle: None,
    image: None,
  };
  let mut annotations = Vec::new();
  let edit = AnnotationEdit::begin(
    &mut annotations,
    AnnotationGestureTarget::New,
    point(100.0, 100.0),
    None,
    Some(AnnotationKind::Image),
    &fresh,
    2.0,
  );
  assert!(edit.is_none());
  assert!(annotations.is_empty());
}

#[test]
fn a_picture_of_its_own_lands_at_its_own_size_or_the_largest_that_fits() {
  use crate::editor::annotations::edit::FreshAnnotation;
  let art = |pixels: f64, aspect: f64| FreshAnnotation {
    angle: None,
    image: Some(super::ImageArt {
      asset: PICTURE.to_owned(),
      aspect,
      pixels: Some(pixels),
      play: None,
    }),
  };
  let size = |fresh: FreshAnnotation| {
    let fresh = fresh.within((1920, 1080));
    let image = super::model::new_image(
      "s".to_owned(),
      point(0.0, 0.0),
      fresh.image.as_ref().expect("a chosen picture"),
      None,
      2.0,
    );
    let AnnotationShape::Image { size, .. } = image.shape else {
      unreachable!()
    };
    size
  };
  // Small enough: its own pixels, not the 96 points of a picture of unknown
  // size.
  assert_eq!(size(art(300.0, 1.5)), 300.0);
  // A wide picture is held by the width, a tall one by the height.
  assert_eq!(size(art(4000.0, 2.0)), 1920.0);
  assert_eq!(size(art(4000.0, 0.5)), 1080.0);
  // A square one by the shorter side of the picture.
  assert_eq!(size(art(4000.0, 1.0)), 1080.0);
}

#[test]
fn a_document_image_reads_back_unmirrored_and_still_when_it_does_not_say() {
  let shape: AnnotationShape = serde_json::from_str(
    r#"{"kind":"image","center":{"x":10,"y":20},"size":64,"angle":0.5,"aspect":2,"asset":"image:0123456789abcdef0123456789abcdef"}"#,
  )
  .expect("a stored image");
  assert_eq!(
    shape,
    AnnotationShape::Image {
      center: point(10.0, 20.0),
      size: 64.0,
      angle: 0.5,
      aspect: 2.0,
      flip: false,
      asset: PICTURE.to_owned(),
      play: None,
      sway: None,
      clock_ms: None,
    }
  );
  assert!(shape.placed());
}

#[test]
fn an_image_keeps_how_it_plays_and_sways_but_never_its_clock() {
  let mut shape: AnnotationShape = serde_json::from_str(
    r#"{"kind":"image","center":{"x":0,"y":0},"size":64,"angle":0,"aspect":1,"asset":"image:x","play":{"cycleMs":400,"frames":3,"frame":1,"once":true},"sway":7}"#,
  )
  .expect("a stored image");
  let AnnotationShape::Image {
    play: Some(play),
    sway,
    clock_ms,
    ..
  } = &mut shape
  else {
    panic!("no play");
  };
  assert_eq!(
    (play.cycle_ms, play.frames, play.frame, play.once),
    (400.0, 3, 1, true)
  );
  assert_eq!(*sway, Some(7));
  *clock_ms = Some(120.0);
  let written = serde_json::to_string(&shape).expect("a written image");
  assert!(
    written.contains(r#""play":{"cycleMs":400.0,"frames":3,"frame":1,"once":true},"sway":7}"#),
    "{written}"
  );
}

#[test]
fn a_turned_image_fits_the_box_its_grips_stand_on() {
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
fn a_side_dragged_past_its_opposite_stops_at_the_smallest_image() {
  let (low, high) = (point(0.0, 0.0), point(100.0, 100.0));
  let (center, size) = resized(low, high, 100.0, EDGE_LEFT, point(150.0, 50.0));
  assert_eq!(size, MIN_IMAGE_SIZE);
  // An edge leaves the other axis centred where it was.
  assert_eq!(center, point(100.0 - MIN_IMAGE_SIZE / 2.0, 50.0));
}

/// A square picture 100 across in the middle of nowhere, turned `angle`,
/// and the press that begins a drag on it at `at`.
fn turned(
  angle: f64,
  at: AnnotationPoint,
) -> (
  crate::editor::annotations::Annotation,
  crate::editor::annotations::gesture::AnnotationDragOrigin,
) {
  let art = super::ImageArt {
    asset: PICTURE.to_owned(),
    aspect: 1.0,
    pixels: Some(100.0),
    play: None,
  };
  let mut image = super::model::new_image("i".to_owned(), point(0.0, 0.0), &art, None, 1.0);
  if let AnnotationShape::Image { angle: turn, .. } = &mut image.shape {
    *turn = angle;
  }
  let origin = crate::editor::annotations::gesture::AnnotationDragOrigin::new(at, &image.shape);
  (image, origin)
}

#[test]
fn a_turned_image_s_corner_sizes_it_from_its_own_opposite_corner() {
  use crate::editor::annotations::gesture::AnnotationHandle;
  // A quarter turn: the picture's own bottom-right corner sits at the
  // bottom left on screen, and its own top-left corner at the top right.
  let angle = std::f64::consts::FRAC_PI_2;
  let (mut image, origin) = turned(angle, point(-50.0, 50.0));
  super::gesture::drag(
    &mut image,
    AnnotationHandle::Edges(EDGE_RIGHT | EDGE_BOTTOM),
    point(-150.0, 150.0),
    &origin,
    false,
    None,
  );
  let AnnotationShape::Image { center, size, .. } = image.shape else {
    unreachable!()
  };
  assert!((size - 200.0).abs() < 1e-9, "{size}");
  // The held corner stays at the top right.
  assert!(
    (center.x - -50.0).abs() < 1e-9 && (center.y - 50.0).abs() < 1e-9,
    "{center:?}"
  );
}

#[test]
fn a_turned_image_s_radius_dot_reads_along_its_own_diagonal() {
  use crate::editor::annotations::gesture::AnnotationHandle;
  // A quarter turn: the picture's own top-left corner is at the top right,
  // and its diagonal runs in towards the bottom left.
  let (mut image, origin) = turned(std::f64::consts::FRAC_PI_2, point(50.0, -50.0));
  let at = |image: &mut crate::editor::annotations::Annotation, x: f64, y: f64| {
    super::gesture::drag(
      image,
      AnnotationHandle::Radius,
      point(x, y),
      &origin,
      false,
      None,
    );
    image.style.radius
  };
  // Half the shorter side's travel in: the whole radius.
  assert!((at(&mut image, 50.0 - 27.5, -50.0 + 27.5) - 50.0).abs() < 1e-9);
  assert!((at(&mut image, 50.0 - 11.0, -50.0 + 11.0) - 20.0).abs() < 1e-9);
  // Outside the corner, none; past the middle, no more than the whole.
  assert_eq!(at(&mut image, 60.0, -60.0), 0.0);
  assert_eq!(at(&mut image, -40.0, 40.0), 50.0);
}

#[test]
fn the_turning_grip_stands_the_top_towards_the_hand() {
  let center = point(0.0, 0.0);
  assert_eq!(image_turn(center, point(0.0, -10.0), 1.0, false), 0.0);
  let right = image_turn(center, point(10.0, 0.0), 0.0, false);
  assert!((right - std::f64::consts::FRAC_PI_2).abs() < 1e-9);
  // Shift holds it to the eighth turns, a little off a diagonal included.
  let snapped = image_turn(center, point(10.0, -9.0), 0.0, true);
  assert!((snapped - std::f64::consts::FRAC_PI_4).abs() < 1e-9);
  // A press on the middle keeps the turn it had.
  assert_eq!(image_turn(center, center, 0.7, false), 0.7);
}

#[test]
fn a_turned_image_is_picked_inside_its_turned_box_only() {
  // A picture 40 wide and 20 tall, turned a quarter round: 20 wide and 40
  // tall on the canvas.
  let geometry = prepare_image(
    [100.0, 100.0],
    [100.0, 120.0],
    [90.0, 100.0],
    0.0,
    false,
    AnnotationReveal::WHOLE,
  );
  assert!(image_distance([100.0, 115.0], &geometry) <= 0.0);
  assert!(image_distance([115.0, 100.0], &geometry) > 0.0);
}

#[test]
fn an_arriving_image_grows_out_of_its_middle() {
  let reveal = AnnotationReveal {
    scale: 0.5,
    ..AnnotationReveal::WHOLE
  };
  let geometry = prepare_image([0.0, 0.0], [40.0, 0.0], [0.0, 20.0], 0.0, false, reveal);
  assert_eq!(geometry.b, [20.0, 0.0]);
  assert_eq!(geometry.c, [0.0, 10.0]);
  assert_eq!(geometry.width, 40.0);
}
