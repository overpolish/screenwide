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
    radius: 0.0,
    tilt_x: 0.0,
    tilt_y: 0.0,
    layer_id: 0,
    index: 0,
    kind: 0,
    flags: 0,
  }
}

/// The grip under `point` with no lens over the annotation.
fn grip_at_point(
  image: PreviewSurfaceRect,
  item: &NativeAnnotationHandles,
  point: (f64, f64),
) -> Option<u32> {
  grip_seen(image, item, point, LensView::IDENTITY, |grip| {
    (grip, LensView::IDENTITY)
  })
  .map(|(handle, _)| handle)
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
fn a_grip_seen_through_a_lens_is_picked_where_it_shows() {
  // A lens enlarging twice about (100, 50) shows the start grip at (20, 50)
  // at (-60, 50), and a drag of it is read back through the lens.
  let view = LensView {
    scale: 0.5,
    offset: [50.0, 25.0],
  };
  let seen = |(x, y): (f64, f64)| {
    let [x, y] = view.seen([x, y]);
    ((x, y), view)
  };
  let item = arrow();
  assert_eq!(
    grip_seen(IMAGE, &item, (-60.0, 50.0), LensView::IDENTITY, seen),
    Some((0, view))
  );
  assert_eq!(
    grip_seen(IMAGE, &item, (20.0, 50.0), LensView::IDENTITY, seen),
    None
  );
}

#[test]
fn the_shaft_is_picked_over_the_width_it_is_drawn_at() {
  let item = arrow();
  assert_eq!(arrow_distance(IMAGE, &item, (100.0, 50.0)), 0.0);
  assert_eq!(arrow_distance(IMAGE, &item, (60.0, 50.0)), 0.0);
  // The stroke's own edge, 4 points out, is still the arrow.
  assert_eq!(arrow_distance(IMAGE, &item, (100.0, 54.0)), 0.0);
  // Just past it is not. There is no pointing slop around an annotation: the
  // halo would otherwise sit over blank picture beside the arrow.
  assert!(arrow_distance(IMAGE, &item, (100.0, 54.5)) > 0.0);
  assert!(arrow_distance(IMAGE, &item, (100.0, 90.0)) > 0.0);
}

#[test]
fn a_wider_stroke_is_picked_further_from_its_centreline() {
  // The width rides on the annotation rather than being read off its heads, so
  // a headless arrow is picked over the whole width it shows.
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

/// Two layers filling the picture: an upright one and one turned a quarter
/// in its plane, back to front as `stacked` lists them, the arrow on
/// `arrow_layer`, and either in hand or under the arrow.
fn stacked(stacked: [u32; 2], arrow_layer: u32, chosen: Option<u32>) -> SurfaceState {
  let whole = PreviewSelection {
    width: 1.0,
    height: 1.0,
    image_width: 1.0,
    image_height: 1.0,
    turnable: 1,
    ..Default::default()
  };
  let upright = PreviewSelection {
    layer_id: 1,
    ..whole
  };
  let turned = PreviewSelection {
    layer_id: 2,
    rotation_z: 90.0,
    ..whole
  };
  let mut state = SurfaceState {
    panes: vec![Some(crate::editing::Pane {
      base_rect: IMAGE,
      seen: true,
      ..Default::default()
    })],
    selection_targets: stacked
      .iter()
      .map(|layer| if *layer == 1 { upright } else { turned })
      .collect(),
    ..Default::default()
  };
  state.selection = chosen.and_then(|layer| layer_selection(&state, layer as i32));
  state.annotation.handles = vec![NativeAnnotationHandles {
    layer_id: arrow_layer as i32,
    ..arrow()
  }];
  state
}

/// [`stacked`] with the arrow's layer in front, where nothing hides it.
fn layered(arrow_layer: u32, chosen: Option<u32>) -> SurfaceState {
  stacked([3 - arrow_layer, arrow_layer], arrow_layer, chosen)
}

#[test]
fn an_annotation_is_picked_where_its_own_layer_draws_it_whichever_layer_is_chosen() {
  // The upright arrow runs along y = 50; the turned layer, in hand, reads
  // the press at (60, 50) as (100, 10) or (100, 90), off the arrow.
  let state = layered(1, Some(2));
  let point = crate::editing::turn::selected_layer_point(&state, (60.0, 50.0));
  assert_eq!(shaft_at_point(&state, point), Some(0));
  // On the turned layer the arrow is drawn upright from (100, 10) to
  // (100, 90): a press there picks it with nothing chosen and with the
  // upright layer in hand, and the press on its upright place does not.
  for chosen in [None, Some(1)] {
    let state = layered(2, chosen);
    let on = crate::editing::turn::selected_layer_point(&state, (100.0, 30.0));
    let off = crate::editing::turn::selected_layer_point(&state, (60.0, 50.0));
    assert_eq!(shaft_at_point(&state, on), Some(0), "chosen {chosen:?}");
    assert_eq!(shaft_at_point(&state, off), None, "chosen {chosen:?}");
  }
}

#[test]
fn an_annotation_a_layer_in_front_hides_is_not_picked_there() {
  // The turned layer stands in front over (50, -50) to (150, 150): the arrow
  // on the upright layer behind is picked only where it shows.
  for chosen in [None, Some(1), Some(2)] {
    let state = stacked([1, 2], 1, chosen);
    let hidden = crate::editing::turn::selected_layer_point(&state, (60.0, 50.0));
    let shown = crate::editing::turn::selected_layer_point(&state, (40.0, 50.0));
    assert_eq!(shaft_at_point(&state, hidden), None, "chosen {chosen:?}");
    assert_eq!(shaft_at_point(&state, shown), Some(0), "chosen {chosen:?}");
  }
}

#[test]
fn a_stroke_is_picked_on_its_line_and_never_by_its_box() {
  // A straight stroke from (20, 10) to (180, 90), 8 display points wide,
  // chosen with the select tool in hand: the case a box would pick most.
  let mut state = layered(1, Some(1));
  state.annotation.paths = vec![[0.1, 0.1], [0.5, 0.5], [0.9, 0.9]];
  state.annotation.handles = vec![NativeAnnotationHandles {
    kind: AnnotationKind::Draw.raw(),
    layer_id: 1,
    start_x: 0.1,
    start_y: 0.1,
    end_x: 0.9,
    end_y: 0.9,
    start_head: 0.0,
    end_head: 3.0,
    ..arrow()
  }];
  state.annotation.mode = MODE_SELECT;
  state.annotation.selected = 0;
  let on_line = crate::editing::turn::selected_layer_point(&state, (100.0, 50.0));
  let in_box = crate::editing::turn::selected_layer_point(&state, (160.0, 20.0));
  assert_eq!(shaft_at_point(&state, on_line), Some(0));
  assert_eq!(shaft_at_point(&state, in_box), None);
}
