// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! What a press over the picture lands on. Pure of the composition device, so
//! these run without a D3D11 adapter.

use super::*;

/// A 200x100 picture at the origin, so a normalised handle at (0.5, 0.5)
/// lands on (100, 50) and display points are easy to read.
const IMAGE: PreviewSurfaceRect = PreviewSurfaceRect {
  x: 0.0,
  y: 0.0,
  width: 200.0,
  height: 100.0,
};

/// A straight arrow across the middle of the picture, headless unless a test
/// asks for a head. Its stroke is a twentieth of the picture's width, so it
/// is drawn 8 display points across: 4 either side of the centreline.
fn arrow() -> NativeAnnotationHandles {
  NativeAnnotationHandles {
    start_x: 0.1,
    start_y: 0.5,
    middle_x: 0.5,
    middle_y: 0.5,
    end_x: 0.9,
    end_y: 0.5,
    start_head: 0.0,
    end_head: 0.0,
    width: 0.04,
    layer_id: 0,
    index: 0,
  }
}

#[test]
fn each_grip_is_picked_over_its_own_hit_box() {
  let item = arrow();
  // The three grips sit at x = 20, 100 and 180, all at y = 50.
  assert_eq!(grip_at_point(IMAGE, &item, (20.0, 50.0)), Some(0));
  assert_eq!(grip_at_point(IMAGE, &item, (100.0, 50.0)), Some(1));
  assert_eq!(grip_at_point(IMAGE, &item, (180.0, 50.0)), Some(2));
}

#[test]
fn the_grip_hit_box_is_square_and_stops_at_its_edge() {
  let item = arrow();
  // A square box, not a radius: the corner of the box still counts, which is
  // what the selection handles do.
  assert_eq!(
    grip_at_point(IMAGE, &item, (20.0 + HANDLE_HIT, 50.0 + HANDLE_HIT)),
    Some(0)
  );
  assert_eq!(
    grip_at_point(IMAGE, &item, (20.0 + HANDLE_HIT + 0.5, 50.0)),
    None
  );
  assert_eq!(
    grip_at_point(IMAGE, &item, (20.0, 50.0 - HANDLE_HIT - 0.5)),
    None
  );
}

#[test]
fn a_press_away_from_every_grip_picks_none() {
  assert_eq!(grip_at_point(IMAGE, &arrow(), (100.0, 5.0)), None);
}

#[test]
fn the_shaft_is_picked_over_the_width_it_is_drawn_at() {
  let item = arrow();
  assert_eq!(arrow_distance(IMAGE, &item, (100.0, 50.0)), 0.0);
  assert_eq!(arrow_distance(IMAGE, &item, (60.0, 50.0)), 0.0);
  // The stroke's own edge, 4 points out, is still the arrow.
  assert_eq!(arrow_distance(IMAGE, &item, (100.0, 54.0)), 0.0);
  // Just past it is not. There is no pointing slop around a mark: the halo
  // would otherwise sit over blank picture beside the arrow.
  assert!(arrow_distance(IMAGE, &item, (100.0, 54.5)) > 0.0);
  assert!(arrow_distance(IMAGE, &item, (100.0, 90.0)) > 0.0);
}

#[test]
fn a_wider_stroke_is_picked_further_from_its_centreline() {
  // The width rides on the mark rather than being read off its heads, so a
  // headless arrow is picked over the whole width it shows.
  let mut item = arrow();
  item.width = 0.2;
  assert_eq!(arrow_distance(IMAGE, &item, (100.0, 70.0)), 0.0);
  assert!(arrow_distance(IMAGE, &arrow(), (100.0, 70.0)) > 0.0);
}

#[test]
fn a_bent_shaft_is_picked_where_it_actually_runs() {
  let mut item = arrow();
  // Pull the middle handle up: the curve now passes through (100, 20), and
  // the straight chord it used to lie on is no longer part of the arrow.
  item.middle_y = 0.2;
  assert_eq!(arrow_distance(IMAGE, &item, (100.0, 20.0)), 0.0);
  assert!(arrow_distance(IMAGE, &item, (100.0, 50.0)) > 0.0);
}

#[test]
fn a_head_is_part_of_the_arrow_it_points_with() {
  let mut item = arrow();
  // A head on the end reaches back four stroke widths and is four wide at
  // its base, so it stands well outside the shaft.
  item.end_head = 0.1;
  let inside_the_head = (160.0, 55.0);
  assert_eq!(
    arrow_distance(IMAGE, &item, inside_the_head),
    0.0,
    "a press on the head must pick the arrow"
  );
  assert!(
    arrow_distance(IMAGE, &arrow(), inside_the_head) > 0.0,
    "without the head the same press must miss"
  );
}
