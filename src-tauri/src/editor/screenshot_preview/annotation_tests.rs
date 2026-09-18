// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::annotation::*;
use crate::editor::annotations::bend::{control_through_midpoint, curve_midpoint};
use crate::editor::annotations::handles::*;
use crate::editor::annotations::model::*;
use crate::editor::annotations::{Annotation, AnnotationPoint, AnnotationShape, AnnotationStyle};

const A: AnnotationPoint = AnnotationPoint { x: 10.0, y: 20.0 };
const C: AnnotationPoint = AnnotationPoint { x: 110.0, y: 60.0 };

#[test]
fn the_middle_handle_solves_back_to_the_point_it_was_dragged_to() {
  let middle = AnnotationPoint { x: 40.0, y: 90.0 };
  let control = control_through_midpoint(A, middle, C);
  let solved = curve_midpoint(A, control, C);
  assert!((solved.x - middle.x).abs() < 1e-9, "{solved:?}");
  assert!((solved.y - middle.y).abs() < 1e-9, "{solved:?}");
}

#[test]
fn a_straight_arrow_puts_its_middle_handle_on_the_midpoint() {
  let arrow = new_arrow("a".to_owned(), A, C, None);
  let AnnotationShape::Arrow {
    start,
    control,
    end,
  } = arrow.shape
  else {
    unreachable!()
  };
  assert_eq!(control, AnnotationPoint { x: 60.0, y: 40.0 });
  assert_eq!(curve_midpoint(start, control, end), control);
  assert_eq!(arrow.style.width, NEW_ARROW_WIDTH);
  assert_eq!(
    arrow.style.head,
    crate::editor::annotations::AnnotationHead::End
  );
}

#[test]
fn normalised_and_source_space_are_inverses() {
  let source = (1_920, 1_080);
  let point = source_point(0.25, 0.75, source);
  assert_eq!(point, AnnotationPoint { x: 480.0, y: 810.0 });
  assert_eq!(normalised_point(point, source), (0.25, 0.75));
}

#[test]
fn handles_report_the_curve_point_rather_than_the_control_point() {
  let annotation = new_arrow("a".to_owned(), A, C, None);
  let bent = Annotation {
    shape: AnnotationShape::Arrow {
      start: A,
      control: AnnotationPoint { x: 60.0, y: 0.0 },
      end: C,
    },
    ..annotation
  };
  let handles = annotation_handles(&[bent], (100, 100), 100.0);
  assert_eq!(handles.len(), 1);
  assert!((handles[0].start_x - 0.1).abs() < 1e-9);
  // 0.25 * 20 + 0.5 * 0 + 0.25 * 60 = 20
  assert!((handles[0].middle_y - 0.2).abs() < 1e-9, "{handles:?}");
  assert!((handles[0].middle_x - 0.6).abs() < 1e-9, "{handles:?}");
}

/// The head travels as a share of the picture, so the native chrome can put
/// it on screen without knowing what an output pixel is: four stroke widths
/// against the width the picture is drawn at. An end without a head is zero.
#[test]
fn head_reach_is_four_strokes_as_a_share_of_the_drawn_picture() {
  let arrow = new_arrow("a".to_owned(), A, C, None);
  let straight = annotation_handles(std::slice::from_ref(&arrow), (100, 100), 800.0);
  assert!((straight[0].end_head - 8.0 * 4.0 / 800.0).abs() < 1e-12);
  assert_eq!(straight[0].start_head, 0.0);
  let both = Annotation {
    style: AnnotationStyle {
      head: crate::editor::annotations::AnnotationHead::Both,
      ..arrow.style.clone()
    },
    ..arrow.clone()
  };
  let handles = annotation_handles(&[both], (100, 100), 800.0);
  assert_eq!(handles[0].start_head, handles[0].end_head);
  let none = Annotation {
    style: AnnotationStyle {
      head: crate::editor::annotations::AnnotationHead::None,
      ..arrow.style.clone()
    },
    ..arrow
  };
  let handles = annotation_handles(&[none], (100, 100), 800.0);
  assert_eq!(handles[0].end_head, 0.0);
  // The stroke rides separately: a headless annotation is still picked, and
  // haloed, over the width it shows.
  assert!((handles[0].width - 8.0 / 800.0).abs() < 1e-12);
}

#[test]
fn the_hover_halo_eases_from_three_points_to_eight() {
  assert!((hover_width_points(0.0) - 3.0).abs() < 1e-9);
  assert!((hover_width_points(1.0) - 8.0).abs() < 1e-9);
  // Ease-out cubic is past its halfway point at half the time.
  assert!(hover_width_points(0.5) > 5.5, "{}", hover_width_points(0.5));
}

/// The very first arrow after a fresh launch wears the palette's yellow.
#[test]
fn a_fresh_arrow_with_nothing_settled_on_wears_the_default() {
  let arrow = new_arrow("a".to_owned(), A, C, None);
  assert_eq!(arrow.style.color, "#ffcc00");
  assert_eq!(arrow.style.width, NEW_ARROW_WIDTH);
  assert_eq!(
    arrow.style.head,
    crate::editor::annotations::AnnotationHead::End
  );
}

#[test]
fn a_fresh_arrow_wears_the_style_it_is_given() {
  let style = AnnotationStyle {
    color: "#ff9500".to_owned(),
    head: crate::editor::annotations::AnnotationHead::Both,
    width: 12.0,
  };
  let arrow = new_arrow("a".to_owned(), A, C, Some(&style));
  assert_eq!(arrow.style, style);
}
