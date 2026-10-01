// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;

fn region(
  col_start: u32,
  col_span: u32,
  grid_cols: u32,
  row_start: u32,
  row_span: u32,
) -> PlacedRegion {
  PlacedRegion {
    col_start,
    col_span,
    grid_cols,
    row_start,
    row_span,
  }
}

#[test]
fn fills_the_whole_work_area() {
  assert_eq!(
    region_rect((0.0, 25.0), (1_440.0, 875.0), &region(0, 2, 2, 0, 2), 0),
    ((0.0, 25.0), (1_440.0, 875.0))
  );
}

#[test]
fn halves_meet_on_one_shared_edge() {
  // An odd width is the seam case: fraction-scaled sizing would round both
  // halves to 720.5 -> 721 and overlap, or floor both and leave a 1px gap.
  let work_origin = (-1_441.0, 25.0);
  let work_size = (1_441.0, 875.0);
  let (left_origin, left_size) = region_rect(work_origin, work_size, &region(0, 1, 2, 0, 2), 0);
  let (right_origin, right_size) = region_rect(work_origin, work_size, &region(1, 1, 2, 0, 2), 0);

  assert_eq!(left_origin.0 + left_size.0, right_origin.0);
  assert_eq!(right_origin.0 + right_size.0, work_origin.0 + work_size.0);
  assert_eq!(left_size.0 + right_size.0, work_size.0);
}

#[test]
fn rows_meet_on_one_shared_edge() {
  let work_origin = (0.0, 25.0);
  let work_size = (1_440.0, 875.0);
  let (top_origin, top_size) = region_rect(work_origin, work_size, &region(0, 2, 2, 0, 1), 0);
  let (bottom_origin, bottom_size) = region_rect(work_origin, work_size, &region(0, 2, 2, 1, 1), 0);

  assert_eq!(top_origin.1 + top_size.1, bottom_origin.1);
  assert_eq!(bottom_origin.1 + bottom_size.1, work_origin.1 + work_size.1);
}

#[test]
fn thirds_tile_the_full_width() {
  let work_origin = (0.0, 0.0);
  let work_size = (1_000.0, 600.0);
  let widths: Vec<f64> = (0..3)
    .map(|column| {
      region_rect(work_origin, work_size, &region(column, 1, 3, 0, 2), 0)
        .1
         .0
    })
    .collect();

  assert_eq!(widths, vec![333.0, 333.0, 334.0]);
  assert_eq!(widths.iter().sum::<f64>(), work_size.0);
}

#[test]
fn a_two_thirds_span_starts_where_its_first_column_does() {
  assert_eq!(
    region_rect((0.0, 0.0), (1_000.0, 600.0), &region(1, 2, 3, 0, 2), 0),
    ((333.0, 0.0), (667.0, 600.0))
  );
}

#[test]
fn a_quarter_takes_one_cell_of_the_two_row_grid() {
  assert_eq!(
    region_rect(
      (-1_440.0, 25.0),
      (1_440.0, 874.0),
      &region(1, 1, 2, 1, 1),
      0
    ),
    ((-720.0, 462.0), (720.0, 437.0))
  );
}

#[test]
fn a_malformed_region_stays_on_the_monitor() {
  assert_eq!(
    region_rect((0.0, 0.0), (1_200.0, 800.0), &region(9, 0, 0, 9, 0), 0),
    ((0.0, 400.0), (1_200.0, 400.0))
  );
}

#[test]
fn a_zero_gap_places_a_region_exactly_where_it_always_did() {
  let work_origin = (-1_441.0, 25.0);
  let work_size = (1_441.0, 875.0);
  for placed in [
    region(0, 2, 2, 0, 2),
    region(0, 1, 2, 0, 2),
    region(1, 1, 2, 1, 1),
    region(1, 1, 3, 0, 1),
  ] {
    let (origin, size) = region_rect(work_origin, work_size, &placed, 0);
    let (left, right) = (
      edge(
        work_origin.0,
        work_size.0,
        placed.col_start,
        placed.grid_cols,
      ),
      edge(
        work_origin.0,
        work_size.0,
        placed.col_start + placed.col_span,
        placed.grid_cols,
      ),
    );

    assert_eq!(origin.0, left);
    assert_eq!(size.0, right - left);
  }
}

#[test]
fn adjacent_halves_sit_exactly_one_gap_apart() {
  let work_origin = (0.0, 0.0);
  let work_size = (1_000.0, 600.0);
  let (left_origin, left_size) = region_rect(work_origin, work_size, &region(0, 1, 2, 0, 2), 8);
  let (right_origin, right_size) = region_rect(work_origin, work_size, &region(1, 1, 2, 0, 2), 8);

  assert_eq!(right_origin.0 - (left_origin.0 + left_size.0), 8.0);
  // And the outer edges are inset by the whole gap, not half of it.
  assert_eq!(left_origin.0, 8.0);
  assert_eq!(right_origin.0 + right_size.0, 992.0);
}

#[test]
fn an_odd_gap_stays_on_whole_pixels() {
  let work_origin = (0.0, 0.0);
  let work_size = (1_000.0, 600.0);
  let (top_origin, top_size) = region_rect(work_origin, work_size, &region(0, 2, 2, 0, 1), 9);
  let (bottom_origin, _) = region_rect(work_origin, work_size, &region(0, 2, 2, 1, 1), 9);

  assert_eq!(top_origin.1, 9.0);
  assert_eq!(top_origin.1 + top_size.1, 296.0);
  assert_eq!(bottom_origin.1, 305.0);
  assert_eq!(bottom_origin.1 - (top_origin.1 + top_size.1), 9.0);
}

#[test]
fn a_full_region_is_inset_from_every_work_area_edge() {
  assert_eq!(
    region_rect((0.0, 25.0), (1_440.0, 875.0), &region(0, 2, 2, 0, 2), 12),
    ((12.0, 37.0), (1_416.0, 851.0))
  );
}

#[test]
fn a_gap_wider_than_the_region_collapses_it_rather_than_inverting_it() {
  let (_, size) = region_rect((0.0, 0.0), (40.0, 40.0), &region(0, 1, 2, 0, 1), 32);

  assert_eq!(size, (0.0, 0.0));
}

fn gravity(
  col_start: u32,
  col_span: u32,
  grid_cols: u32,
  row_start: u32,
  row_span: u32,
) -> (Gravity, Gravity) {
  let pull = region_gravity(&region(col_start, col_span, grid_cols, row_start, row_span));
  (pull.horizontal, pull.vertical)
}

#[test]
fn halves_pull_towards_the_edge_they_were_thrown_at() {
  assert_eq!(gravity(0, 1, 2, 0, 2), (Gravity::Start, Gravity::Center));
  assert_eq!(gravity(1, 1, 2, 0, 2), (Gravity::End, Gravity::Center));
}

#[test]
fn a_full_region_has_no_edge_to_prefer() {
  assert_eq!(gravity(0, 2, 2, 0, 2), (Gravity::Center, Gravity::Center));
}

#[test]
fn full_width_rows_pull_only_vertically() {
  assert_eq!(gravity(0, 2, 2, 0, 1), (Gravity::Center, Gravity::Start));
  assert_eq!(gravity(0, 2, 2, 1, 1), (Gravity::Center, Gravity::End));
}

#[test]
fn quarters_pull_on_both_axes() {
  assert_eq!(gravity(0, 1, 2, 0, 1), (Gravity::Start, Gravity::Start));
  assert_eq!(gravity(1, 1, 2, 1, 1), (Gravity::End, Gravity::End));
}

#[test]
fn the_middle_third_centers_and_the_outer_thirds_do_not() {
  assert_eq!(gravity(0, 1, 3, 0, 2), (Gravity::Start, Gravity::Center));
  assert_eq!(gravity(1, 1, 3, 0, 2), (Gravity::Center, Gravity::Center));
  assert_eq!(gravity(2, 1, 3, 0, 2), (Gravity::End, Gravity::Center));
}

#[test]
fn two_thirds_spans_keep_the_edge_they_reach() {
  assert_eq!(gravity(0, 2, 3, 0, 2), (Gravity::Start, Gravity::Center));
  assert_eq!(gravity(1, 2, 3, 0, 2), (Gravity::End, Gravity::Center));
}

#[test]
fn a_malformed_region_still_yields_a_gravity() {
  assert_eq!(gravity(9, 0, 0, 9, 0), (Gravity::Center, Gravity::End));
}
