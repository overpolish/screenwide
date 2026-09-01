// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later
use super::super::regions::{self, GlideRegion};
fn r(c: u32, s: u32, sp: u32, y: u32, ys: u32) -> GlideRegion {
  GlideRegion {
    grid_cols: c,
    col_start: s,
    col_span: sp,
    row_start: y,
    row_span: ys,
  }
}
#[test]
fn bottom_halves() {
  assert_eq!(regions::bottom_row_region(2), r(2, 0, 2, 1, 1))
}
#[test]
fn bottom_thirds() {
  assert_eq!(regions::bottom_row_region(3), r(3, 0, 3, 1, 1))
}
#[test]
fn regrid_half_to_two_thirds() {
  assert_eq!(
    regions::regrid_region(r(2, 1, 1, 0, 2), true),
    r(3, 1, 2, 0, 2)
  )
}
#[test]
fn regrid_full_to_full() {
  assert_eq!(
    regions::regrid_region(r(2, 0, 2, 0, 2), true),
    r(3, 0, 3, 0, 2)
  )
}
#[test]
fn regrid_middle_to_half() {
  assert_eq!(
    regions::regrid_region(r(3, 1, 1, 0, 2), false),
    r(2, 0, 2, 0, 2)
  )
}
#[test]
fn two_col_full_right() {
  assert_eq!(regions::step_columns(r(2, 0, 2, 0, 2), 1), r(2, 1, 1, 0, 2))
}
#[test]
fn two_col_full_left() {
  assert_eq!(
    regions::step_columns(r(2, 1, 1, 0, 2), -1),
    r(2, 0, 1, 0, 2)
  )
}
#[test]
fn two_col_clamps_right() {
  assert_eq!(regions::step_columns(r(2, 1, 1, 0, 2), 1), r(2, 1, 1, 0, 2))
}
#[test]
fn two_col_row_caterpillar() {
  assert_eq!(regions::step_columns(r(2, 0, 1, 1, 1), 1), r(2, 0, 2, 1, 1))
}
#[test]
fn row_grows_full() {
  assert_eq!(regions::step_rows(r(2, 1, 1, 1, 1), -1), r(2, 1, 1, 0, 2))
}
#[test]
fn row_moves_far() {
  assert_eq!(regions::step_rows(r(2, 1, 1, 0, 1), 1), r(2, 1, 1, 0, 2))
}
#[test]
fn row_clamps() {
  assert_eq!(regions::step_rows(r(2, 1, 1, 1, 1), 1), r(2, 1, 1, 1, 1))
}
#[test]
fn ladder_first_right() {
  assert_eq!(regions::step_columns(r(3, 2, 1, 0, 2), 1), r(3, 2, 1, 0, 2))
}
#[test]
fn ladder_right_to_two() {
  assert_eq!(
    regions::step_columns(r(3, 2, 1, 0, 2), -1),
    r(3, 1, 2, 0, 2)
  )
}
#[test]
fn ladder_middle_to_left() {
  assert_eq!(
    regions::step_columns(r(3, 1, 2, 0, 2), -1),
    r(3, 1, 1, 0, 2)
  )
}
#[test]
fn ladder_left_clamps() {
  assert_eq!(
    regions::step_columns(r(3, 0, 1, 0, 2), -1),
    r(3, 0, 1, 0, 2)
  )
}
