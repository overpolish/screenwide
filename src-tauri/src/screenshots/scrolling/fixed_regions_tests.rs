// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;

fn patterned(width: u32, height: u32) -> CapturedImage {
  let mut rgba = vec![255_u8; (width * height * 4) as usize];
  for y in 0..height {
    for x in 0..width {
      let offset = ((y * width + x) * 4) as usize;
      let value = ((x * 17 + y * 31) % 251) as u8;
      rgba[offset..offset + 3].fill(value);
    }
  }
  CapturedImage {
    height,
    rgba,
    width,
  }
}

fn filled(width: u32, height: u32, colour: [u8; 4]) -> CapturedImage {
  CapturedImage {
    height,
    rgba: colour.repeat((width * height) as usize),
    width,
  }
}

fn paint(
  image: &mut CapturedImage,
  rows: std::ops::Range<u32>,
  columns: std::ops::Range<u32>,
  colour: [u8; 4],
) {
  for y in rows {
    for x in columns.clone() {
      let offset = ((y * image.width + x) * 4) as usize;
      image.rgba[offset..offset + 4].copy_from_slice(&colour);
    }
  }
}

#[test]
fn votes_the_static_rail_colour_over_a_larger_changing_patch() {
  let dark = [20, 20, 20, 255];
  let mut previous = filled(64, 512, dark);
  let mut current = filled(64, 512, dark);
  // A bright card animating into the rail covers more of the band than the
  // rail's own still background, so an unfiltered vote would pick it.
  paint(&mut previous, 0..320, 0..32, [128, 128, 128, 255]);
  paint(&mut current, 0..320, 0..32, [250, 250, 250, 255]);

  assert_eq!(
    dominant_colour(&previous, &current, Axis::Vertical, 0, 32),
    dark
  );
  assert_eq!(
    vote(None, &current, Axis::Vertical, 0, 32),
    Some([250, 250, 250, 255])
  );
}

#[test]
fn falls_back_to_every_pixel_when_the_band_barely_holds_still() {
  let mut previous = filled(64, 64, [20, 20, 20, 255]);
  let current = filled(64, 64, [250, 250, 250, 255]);
  paint(&mut previous, 0..8, 0..32, [250, 250, 250, 255]);

  assert_eq!(
    dominant_colour(&previous, &current, Axis::Vertical, 0, 32),
    [250, 250, 250, 255]
  );
}

#[test]
fn excludes_a_band_that_moves_at_a_different_rate() {
  let source = patterned(160, 220);
  let mut previous = CapturedImage {
    height: 100,
    rgba: source.rgba[..160 * 100 * 4].to_vec(),
    width: 160,
  };
  let mut current = CapturedImage {
    height: 100,
    rgba: source.rgba[160 * 40 * 4..160 * 140 * 4].to_vec(),
    width: 160,
  };
  desynchronise(&mut previous, &mut current, &source, 0..40, 5);

  let regions = detect(&previous, &current, Axis::Vertical, Direction::Forward, 40);
  assert!(regions.bands.background(16, 50, &current).is_some());
  assert!(regions.bands.background(120, 50, &current).is_none());
}

/// Copies `columns` of every row from `source` into the pair, taking the
/// current frame's rows `offset` further down so the band moves at its own
/// rate - the signal an overlay scrollbar and a sticky rail both produce.
fn desynchronise(
  previous: &mut CapturedImage,
  current: &mut CapturedImage,
  source: &CapturedImage,
  columns: std::ops::Range<u32>,
  offset: u32,
) {
  for y in 0..previous.height {
    for x in columns.clone() {
      let target = ((y * previous.width + x) * 4) as usize;
      let moved = (((y + offset) * source.width + x) * 4) as usize;
      previous.rgba[target..target + 4].copy_from_slice(&source.rgba[target..target + 4]);
      current.rgba[target..target + 4].copy_from_slice(&source.rgba[moved..moved + 4]);
    }
  }
}

#[test]
fn masks_a_left_rail_but_never_the_scrollbar_edge() {
  let source = patterned(160, 220);
  let mut previous = CapturedImage {
    height: 100,
    rgba: source.rgba[..160 * 100 * 4].to_vec(),
    width: 160,
  };
  let mut current = CapturedImage {
    height: 100,
    rgba: source.rgba[160 * 40 * 4..160 * 140 * 4].to_vec(),
    width: 160,
  };
  desynchronise(&mut previous, &mut current, &source, 0..40, 5);
  desynchronise(&mut previous, &mut current, &source, 120..160, 5);

  let regions = detect(&previous, &current, Axis::Vertical, Direction::Forward, 40);
  assert!(regions.bands.background(16, 50, &current).is_some());
  assert!(regions.bands.background(140, 50, &current).is_none());
  // The strip is still reported, so composition rebuilds it from tile overlap
  // instead of painting over it.
  assert_eq!(regions.bands.right_edge_width(&current), 40);
}

#[test]
fn preserves_an_interior_band_with_a_poor_local_match() {
  let source = patterned(160, 220);
  let previous = CapturedImage {
    height: 100,
    rgba: source.rgba[..160 * 100 * 4].to_vec(),
    width: 160,
  };
  let mut current = CapturedImage {
    height: 100,
    rgba: source.rgba[160 * 40 * 4..160 * 140 * 4].to_vec(),
    width: 160,
  };
  for y in 0..100 {
    for x in 75..85 {
      let offset = ((y * 160 + x) * 4) as usize;
      current.rgba[offset..offset + 3].fill(0);
    }
  }

  let regions = detect(&previous, &current, Axis::Vertical, Direction::Forward, 40);
  assert!(regions.bands.background(80, 50, &current).is_none());
}

/// Builds a scrolled pair of viewports over one tall synthetic document.
fn scrolled_pair(width: u32, height: u32, shift: u32) -> (CapturedImage, CapturedImage) {
  let source = patterned(width, height + shift);
  let row = (width * 4) as usize;
  (
    CapturedImage {
      height,
      rgba: source.rgba[..row * height as usize].to_vec(),
      width,
    },
    CapturedImage {
      height,
      rgba: source.rgba[row * shift as usize..row * (height + shift) as usize].to_vec(),
      width,
    },
  )
}

#[test]
fn measures_a_sticky_header_as_a_leading_static_strip() {
  let (mut previous, mut current) = scrolled_pair(160, 200, 80);
  // A sticky header occupies identical viewport rows in both frames.
  paint(&mut previous, 0..24, 0..160, [30, 40, 50, 255]);
  paint(&mut current, 0..24, 0..160, [30, 40, 50, 255]);

  let regions = detect(&previous, &current, Axis::Vertical, Direction::Forward, 80);
  assert_eq!(regions.leading_strip, 24);
  assert_eq!(regions.trailing_strip, 0);
}

#[test]
fn reports_no_strips_when_both_edges_scroll() {
  let (previous, current) = scrolled_pair(160, 200, 80);

  let regions = detect(&previous, &current, Axis::Vertical, Direction::Forward, 80);
  assert_eq!(regions.leading_strip, 0);
  assert_eq!(regions.trailing_strip, 0);
}

#[test]
fn caps_a_static_strip_at_two_fifths_of_the_axis() {
  // A pair that barely changed would otherwise report the whole frame as
  // chrome and crop away real page content.
  let previous = patterned(160, 200);
  let current = patterned(160, 200);

  let regions = detect(&previous, &current, Axis::Vertical, Direction::Forward, 0);
  assert_eq!(regions.leading_strip, 80);
  assert_eq!(regions.trailing_strip, 80);
}
