// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;

// The production API takes frames that already carry their planes; these
// shadow the module functions with image-based wrappers so the tests can stay
// written in terms of plain pixels.
fn frame(image: &CapturedImage) -> MatchFrame {
  MatchFrame::new(Arc::new(image.clone()))
}

fn find_shift(
  previous: &CapturedImage,
  current: &CapturedImage,
  axis: Axis,
  direction: Direction,
  expected: u32,
) -> Option<u32> {
  super::find_shift(&frame(previous), &frame(current), axis, direction, expected)
}

fn frames_are_same(previous: &CapturedImage, current: &CapturedImage) -> bool {
  super::frames_are_same(&frame(previous), &frame(current))
}

fn content(width: u32, height: u32) -> CapturedImage {
  let mut rgba = vec![255; (width * height * 4) as usize];
  for y in 0..height {
    for x in 0..width {
      let offset = ((y * width + x) * 4) as usize;
      let value = ((x * 17 + y * 31 + (x / 7) * (y / 11)) % 211) as u8;
      rgba[offset] = value;
      rgba[offset + 1] = value.wrapping_mul(3);
      rgba[offset + 2] = value.wrapping_mul(7);
      rgba[offset + 3] = 255;
    }
  }
  CapturedImage {
    height,
    rgba,
    width,
  }
}

fn viewport(source: &CapturedImage, x: u32, y: u32, width: u32, height: u32) -> CapturedImage {
  let mut rgba = Vec::with_capacity((width * height * 4) as usize);
  for row in y..y + height {
    let start = ((row * source.width + x) * 4) as usize;
    rgba.extend_from_slice(&source.rgba[start..start + (width * 4) as usize]);
  }
  CapturedImage {
    height,
    rgba,
    width,
  }
}

/// A strictly periodic line rhythm carrying aperiodic per-line content, which
/// is what a page of text looks like to the matcher: the coarse sweep sees a
/// trough at every line multiple and only the line contents pick the true one.
fn text_lines(width: u32, height: u32) -> CapturedImage {
  let mut rgba = vec![255; (width * height * 4) as usize];
  for y in 0..height {
    let line = y / 20;
    let row_in_line = y % 20;
    for x in 0..width {
      let offset = ((y * width + x) * 4) as usize;
      let value = if row_in_line < 12 {
        (((x * 7 + line * 53 + (x / 5) * (line % 7)) % 199) as u8)
          .saturating_add((row_in_line * 3) as u8)
      } else {
        (238 - row_in_line * 3) as u8
      };
      rgba[offset..offset + 3].fill(value);
      rgba[offset + 3] = 255;
    }
  }
  CapturedImage {
    height,
    rgba,
    width,
  }
}

/// Smooth, non-repeating content built from an interpolated value-noise
/// lattice. Unlike a modular pattern it has no strong aliases, so it stands in
/// for an ordinary page rather than for a worst case.
fn photographic(width: u32, height: u32) -> CapturedImage {
  const CELL: u32 = 4;
  fn lattice(column: u32, row: u32) -> f64 {
    let mut value = column
      .wrapping_mul(374_761_393)
      .wrapping_add(row.wrapping_mul(668_265_263));
    value ^= value >> 13;
    value = value.wrapping_mul(1_274_126_177);
    value ^= value >> 16;
    f64::from(value % 1000) / 1000.0
  }

  let mut rgba = vec![255; (width * height * 4) as usize];
  for y in 0..height {
    let (row, row_fraction) = (y / CELL, f64::from(y % CELL) / f64::from(CELL));
    for x in 0..width {
      let (column, column_fraction) = (x / CELL, f64::from(x % CELL) / f64::from(CELL));
      let top =
        lattice(column, row) * (1.0 - column_fraction) + lattice(column + 1, row) * column_fraction;
      let bottom = lattice(column, row + 1) * (1.0 - column_fraction)
        + lattice(column + 1, row + 1) * column_fraction;
      let value = (20.0 + 215.0 * (top * (1.0 - row_fraction) + bottom * row_fraction)) as u8;
      let offset = ((y * width + x) * 4) as usize;
      rgba[offset..offset + 3].fill(value);
    }
  }
  CapturedImage {
    height,
    rgba,
    width,
  }
}

#[test]
fn finds_vertical_forward_scroll() {
  let source = content(180, 320);
  let previous = viewport(&source, 0, 0, 180, 140);
  let current = viewport(&source, 0, 73, 180, 140);
  assert_eq!(
    find_shift(&previous, &current, Axis::Vertical, Direction::Forward, 80),
    Some(73)
  );
}

#[test]
fn finds_horizontal_backward_scroll() {
  let source = content(360, 150);
  let previous = viewport(&source, 91, 0, 160, 150);
  let current = viewport(&source, 38, 0, 160, 150);
  assert_eq!(
    find_shift(
      &previous,
      &current,
      Axis::Horizontal,
      Direction::Backward,
      60
    ),
    Some(53)
  );
}

#[test]
fn ignores_a_small_dynamic_patch_when_detecting_stagnation() {
  let first = content(160, 120);
  let mut second = first.clone();
  for y in 4..12 {
    for x in 4..12 {
      let offset = ((y * second.width + x) * 4) as usize;
      second.rgba[offset..offset + 3].fill(0);
    }
  }
  assert!(frames_are_same(&first, &second));
}

#[test]
fn ignores_a_fixed_sidebar_when_aligning_scrolled_content() {
  let source = content(220, 360);
  let mut previous = viewport(&source, 0, 0, 220, 150);
  let mut current = viewport(&source, 0, 79, 220, 150);
  for image in [&mut previous, &mut current] {
    for y in 0..image.height {
      for x in 0..70 {
        let offset = ((y * image.width + x) * 4) as usize;
        let value = ((x * 11 + y * 19) % 251) as u8;
        image.rgba[offset..offset + 3].fill(value);
      }
    }
  }
  assert_eq!(
    find_shift(&previous, &current, Axis::Vertical, Direction::Forward, 80),
    Some(79)
  );
}

#[test]
fn finds_a_shift_in_periodic_content_far_from_the_expected_amount() {
  let source = text_lines(180, 320);
  let previous = viewport(&source, 0, 0, 180, 200);
  let current = viewport(&source, 0, 97, 180, 200);
  assert_eq!(
    find_shift(&previous, &current, Axis::Vertical, Direction::Forward, 20),
    Some(97)
  );
}

/// Paints a deterministic full-width band whose pixels depend only on the
/// column and the row *within* the band, so the same band painted at two
/// different offsets is pixel-identical content that merely moved.
fn paint_band(image: &mut CapturedImage, top: u32, height: u32, seed: u32) {
  for row in 0..height {
    let y = top + row;
    for x in 0..image.width {
      let offset = ((y * image.width + x) * 4) as usize;
      let value = ((x * 13 + row * 41 + seed * 97) % 233) as u8;
      image.rgba[offset..offset + 3].fill(value);
    }
  }
}

/// Paints a full-height sticky sidebar down the left of the frame. `raised` is
/// how far the sidebar has slid up the viewport, so two frames with different
/// values show the same sidebar pixels at different y offsets.
fn paint_sidebar(image: &mut CapturedImage, width: u32, raised: u32) {
  for y in 0..image.height {
    let row = y + raised;
    for x in 0..width {
      let offset = ((y * image.width + x) * 4) as usize;
      let value = ((x * 23 + row * 13 + (x / 3) * (row / 5)) % 241) as u8;
      image.rgba[offset..offset + 3].fill(value);
    }
  }
}

/// A dismissible banner above a sticky layout: on the first scroll the banner
/// leaves and the whole sticky layout — a full-width header and the sidebar
/// anchored under it — slides up the viewport. That chrome *moved*, so it
/// passes the viewport-change gate, and it contaminates the sample grid in a
/// cross shape: the header poisons every cross-axis band, the sidebar poisons
/// every along-axis segment. Only a region excluded along both axes at once is
/// clean document.
#[test]
fn ignores_a_sticky_layout_that_slides_up_when_a_banner_scrolls_away() {
  let source = content(180, 400);
  let mut previous = viewport(&source, 0, 0, 180, 200);
  let mut current = viewport(&source, 0, 97, 180, 200);
  paint_band(&mut previous, 0, 45, 5);
  paint_band(&mut previous, 45, 45, 1);
  paint_band(&mut current, 0, 45, 1);
  paint_sidebar(&mut previous, 70, 0);
  paint_sidebar(&mut current, 70, 45);
  assert_eq!(
    find_shift(&previous, &current, Axis::Vertical, Direction::Forward, 92),
    Some(97)
  );
}

#[test]
fn falls_back_to_the_full_sweep_when_the_expected_amount_is_wrong() {
  let source = photographic(180, 320);
  let previous = viewport(&source, 0, 0, 180, 140);
  let current = viewport(&source, 0, 73, 180, 140);
  assert_eq!(
    find_shift(&previous, &current, Axis::Vertical, Direction::Forward, 24),
    Some(73)
  );
}
