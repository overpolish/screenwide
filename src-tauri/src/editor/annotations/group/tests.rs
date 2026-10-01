// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;
use crate::editor::annotations::arrow::model::new_arrow;
use crate::editor::annotations::counter::new_counter;
use crate::editor::annotations::snap::SnapField;

const SOURCE: (u32, u32) = (1920, 1080);

fn point(x: f64, y: f64) -> AnnotationPoint {
  AnnotationPoint { x, y }
}

fn member(annotation: Annotation, scale: f64) -> GroupMember {
  GroupMember {
    before: annotation,
    scale,
  }
}

fn centre(annotation: &Annotation) -> AnnotationPoint {
  let AnnotationShape::Counter { center, .. } = annotation.shape else {
    unreachable!()
  };
  center
}

#[test]
fn an_arrow_is_bounded_past_its_line_by_its_stroke() {
  let arrow = new_arrow(
    "a".to_owned(),
    point(100.0, 100.0),
    point(300.0, 100.0),
    None,
  );
  let bounds = annotation_bounds(&arrow, 2.0).unwrap();
  let reach = arrow.style.width * 2.0;
  assert_eq!(bounds.x, 100.0 - reach);
  assert_eq!(bounds.y, 100.0 - reach);
  assert_eq!(bounds.width, 200.0 + reach * 2.0);
  assert_eq!(bounds.height, reach * 2.0);
}

#[test]
fn a_magnifier_is_bounded_round_its_loupe_as_well_as_its_zoom_area() {
  let mut magnifier = new_arrow("m".to_owned(), point(0.0, 0.0), point(1.0, 1.0), None);
  // A 100 by 50 zoom area enlarged to a 400-pixel loupe standing well away.
  magnifier.shape = AnnotationShape::Magnify {
    start: point(100.0, 100.0),
    end: point(200.0, 150.0),
    loupe: point(800.0, 600.0),
    size: 400.0,
  };
  let bounds = annotation_bounds(&magnifier, 1.0).unwrap();
  assert_eq!((bounds.x, bounds.y), (100.0, 100.0));
  assert_eq!(bounds.x + bounds.width, 1000.0);
  assert_eq!(bounds.y + bounds.height, 700.0);
}

#[test]
fn a_group_carries_every_member_by_one_distance_on_screen() {
  let moving = GroupMove::new(
    point(500.0, 500.0),
    vec![
      member(
        new_counter("screen".to_owned(), point(100.0, 100.0), 1, None, None),
        1.0,
      ),
      // A camera picture drawn at half the screen's density.
      member(
        new_counter("camera".to_owned(), point(50.0, 50.0), 2, None, None),
        0.5,
      ),
    ],
    None,
  );
  let (moved, result) = moving.update(point(540.0, 520.0), SnapModifiers::default(), None);
  assert_eq!(centre(&moved[0]), point(140.0, 120.0));
  assert_eq!(centre(&moved[1]), point(70.0, 60.0));
  assert_eq!(result, SnapResult::default());
}

#[test]
fn shift_holds_a_group_to_the_axis_it_has_come_further_along() {
  let moving = GroupMove::new(
    point(0.0, 0.0),
    vec![member(
      new_counter("c".to_owned(), point(100.0, 100.0), 1, None, None),
      1.0,
    )],
    None,
  );
  let (moved, _) = moving.update(point(30.0, 10.0), SnapModifiers::from_bits(0b01), None);
  assert_eq!(centre(&moved[0]), point(130.0, 100.0));
}

#[test]
fn the_positional_modifier_lands_the_group_box_on_a_canvas_guide() {
  let members = [
    new_counter("a".to_owned(), point(900.0, 300.0), 1, None, None),
    new_counter("b".to_owned(), point(1000.0, 300.0), 2, None, None),
  ];
  // A box from 900 to 1000 across: its centre at 950 is 10 short of the
  // canvas's middle, well inside the reach.
  let held = SnapBox {
    x: 900.0,
    y: 290.0,
    width: 100.0,
    height: 20.0,
  };
  let moving = GroupMove::new(
    point(0.0, 0.0),
    members
      .iter()
      .cloned()
      .map(|annotation| member(annotation, 1.0))
      .collect(),
    Some(held),
  );
  let field = SnapField::without(SOURCE, &members, |_| true, 960.0);
  let request = SnapRequest {
    field: &field,
    threshold: 16.0,
  };
  let (moved, result) = moving.update(
    point(0.0, 0.0),
    SnapModifiers::from_bits(0b10),
    Some(request),
  );
  assert_eq!(centre(&moved[0]).x, 910.0);
  assert_eq!(centre(&moved[1]).x, 1010.0);
  assert!(result.guide_x.is_some());
}

#[test]
fn the_chrome_boxes_every_shown_member_and_the_whole_group() {
  let members = [
    new_counter("shown".to_owned(), point(100.0, 100.0), 1, None, None),
    new_counter("hidden".to_owned(), point(900.0, 500.0), 2, None, None),
  ];
  let boxes = group_boxes(
    &members.iter().collect::<Vec<_>>(),
    |id| id == "shown",
    SOURCE,
    2.0,
    0,
  );
  assert_eq!(boxes.len(), 2);
  assert_eq!(boxes[0].kind, GROUP_BOX_MEMBER);
  assert!(boxes[0].right < 0.1);
  assert_eq!(boxes[1].kind, GROUP_BOX_GROUP);
  // The group's box reaches the hidden member too.
  assert!(boxes[1].right > 900.0 / 1920.0);
}
