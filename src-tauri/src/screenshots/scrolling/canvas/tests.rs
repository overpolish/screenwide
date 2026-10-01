// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;

fn solid(width: u32, height: u32, colour: [u8; 4]) -> CapturedImage {
  CapturedImage {
    height,
    rgba: colour.repeat((width * height) as usize),
    width,
  }
}

fn at(image: &CapturedImage, x: u32, y: u32) -> [u8; 4] {
  let offset = ((y * image.width + x) * 4) as usize;
  image.rgba[offset..offset + 4].try_into().unwrap()
}

#[test]
fn a_later_tiles_real_pixels_replace_an_earlier_masked_fill() {
  let fill = [10, 10, 10, 255];
  let tiles = vec![
    Tile {
      crop: Crop::default(),
      fixed_bands: Some(fixed_regions::FixedBands::masking(
        Axis::Vertical,
        &[(0, fill)],
      )),
      image: Arc::new(solid(32, 8, [255, 0, 0, 255])),
      x: 0,
      y: 0,
    },
    Tile {
      crop: Crop::default(),
      fixed_bands: None,
      image: Arc::new(solid(32, 8, [0, 255, 0, 255])),
      x: 0,
      y: 4,
    },
  ];
  let image = compose(tiles).unwrap();
  assert_eq!((image.width, image.height), (32, 12));
  // The masked band, in rows the terminal tile also covers.
  assert_eq!(at(&image, 0, 5), [0, 255, 0, 255]);
  // The masked band, in rows only the first tile covers.
  assert_eq!(at(&image, 0, 1), fill);
  // An unmasked band: the first real writer still wins the overlap.
  assert_eq!(at(&image, 5, 5), [255, 0, 0, 255]);
}

const PAGE: [u8; 4] = [24, 24, 24, 255];
const CONTENT: [u8; 4] = [240, 240, 240, 255];
const THUMB: [u8; 4] = [90, 90, 90, 255];
/// Inside the detected right-edge strip, which bands 30 and 31 of a 64px-wide
/// frame put at column 60 and up.
const STRIP_COLUMN: u32 = 61;
/// Document rows carrying a distinctive strip pixel - the stand-in for a source
/// control panel's status letters. Row 54 lies where only the terminal tile
/// reaches.
const CONTENT_ROWS: [u32; 5] = [14, 20, 25, 40, 54];

fn paint_block(
  image: &mut CapturedImage,
  rows: std::ops::Range<u32>,
  columns: std::ops::Range<u32>,
  colour: [u8; 4],
) {
  for row in rows {
    for column in columns.clone() {
      let offset = ((row * image.width + column) * 4) as usize;
      image.rgba[offset..offset + 4].copy_from_slice(&colour);
    }
  }
}

/// Three 64x32 tiles scrolling down in steps of 12, so canvas rows 12..44 are
/// covered by at least two of them and rows 24..32 by all three. Content pixels
/// sit at the given *document* rows in every tile that reaches them, while each
/// tile's thumb block is given in canvas rows so a caller can choose which
/// document rows the thumb hides in which tile. An empty range paints no thumb.
fn scrollbar_tiles(content_rows: &[u32], thumbs: [std::ops::Range<u32>; 3]) -> Vec<Tile> {
  let origins = [0_u32, 12, 24];
  origins
    .into_iter()
    .zip(thumbs)
    .map(|(origin, thumb)| {
      let mut image = solid(64, 32, PAGE);
      let height = image.height;
      let covers = |row: u32| row >= origin && row < origin + height;
      for row in content_rows.iter().copied().filter(|row| covers(*row)) {
        let local = row - origin;
        paint_block(
          &mut image,
          local..local + 1,
          STRIP_COLUMN..STRIP_COLUMN + 1,
          CONTENT,
        );
      }
      for row in thumb.filter(|row| covers(*row)) {
        let local = row - origin;
        paint_block(&mut image, local..local + 1, 60..64, THUMB);
      }
      Tile {
        crop: Crop::default(),
        fixed_bands: Some(fixed_regions::FixedBands::masking(
          Axis::Vertical,
          &[(30, FILL), (31, FILL)],
        )),
        image: Arc::new(image),
        x: 0,
        y: origin as i32,
      }
    })
    .collect()
}

#[test]
fn the_right_edge_strip_is_rebuilt_from_what_overlapping_tiles_agree_on() {
  let image = compose(scrollbar_tiles(&CONTENT_ROWS, [0..8, 22..30, 44..52])).unwrap();
  assert_eq!((image.width, image.height), (64, 56));
  // Content survives on rows two tiles agree on, including row 25 where the
  // middle tile's thumb sits on top of it.
  for row in [14, 20, 25, 40] {
    assert_eq!(at(&image, STRIP_COLUMN, row), CONTENT, "row {row}");
  }
  // No thumb colour anywhere in the multiply covered band.
  for row in 12..44 {
    for column in 58..64 {
      assert_ne!(at(&image, column, row), THUMB, "row {row} column {column}");
    }
  }
}

#[test]
fn strip_content_every_covering_tile_agrees_on_survives_verbatim() {
  let image = compose(scrollbar_tiles(&CONTENT_ROWS, [0..0, 0..0, 0..0])).unwrap();
  for row in [14, 20, 25, 40] {
    assert_eq!(at(&image, STRIP_COLUMN, row), CONTENT, "row {row}");
  }
  // The page beside it survives too, rather than being flattened to a fill.
  assert_eq!(at(&image, STRIP_COLUMN, 15), PAGE);
}

#[test]
fn strip_content_survives_wherever_the_thumb_falls() {
  // The regression this reproduces: a right-edge band reads as fixed chrome
  // because the thumb crossed it, and painting it over erased the status
  // letters for the tile's whole document range. Nothing may be masked here -
  // row 5 has no second tile to redeem it, and on row 40 the only later tile is
  // the terminal one, whose own thumb would have been redeemed in its place.
  let rows = [5_u32, 14, 25, 40];
  let image = compose(scrollbar_tiles(&rows, [18..26, 12..20, 40..48])).unwrap();
  for row in rows {
    assert_eq!(at(&image, STRIP_COLUMN, row), CONTENT, "row {row}");
  }
  // Row 25 is thumb-covered in the first tile, and the two later tiles outvote
  // it - the majority the scroll overlap exists to provide.
  for row in 24..32 {
    for column in 58..64 {
      assert_ne!(at(&image, column, row), THUMB, "row {row} column {column}");
    }
  }
}

#[test]
fn a_strip_pixel_only_one_tile_covers_keeps_that_tiles_value() {
  let image = compose(scrollbar_tiles(&CONTENT_ROWS, [0..8, 22..30, 44..52])).unwrap();
  // Row 54 lies past the middle tile, so the terminal tile alone owns it.
  assert_eq!(at(&image, STRIP_COLUMN, 54), CONTENT);
  assert_eq!(at(&image, 63, 54), PAGE);
}

const FILL: [u8; 4] = [10, 10, 10, 255];
const HEADER: [u8; 4] = [255, 0, 255, 255];
const FOOTER: [u8; 4] = [0, 255, 255, 255];

fn paint_rows(image: &mut CapturedImage, rows: std::ops::Range<u32>, colour: [u8; 4]) {
  for row in rows {
    for column in 0..image.width {
      let offset = ((row * image.width + column) * 4) as usize;
      image.rgba[offset..offset + 4].copy_from_slice(&colour);
    }
  }
}

fn step(
  image: CapturedImage,
  axis: Axis,
  direction: Direction,
  expected: u32,
  shift: Option<u32>,
  regions: fixed_regions::FixedRegions,
) -> MatchedStep {
  MatchedStep {
    image: Arc::new(image),
    movement: Some(Movement {
      axis,
      direction,
      expected,
    }),
    outcome: Some(PairOutcome {
      regions,
      matched: matcher::ShiftMatch { shift },
    }),
  }
}

fn vertical_step(
  image: CapturedImage,
  shift: u32,
  regions: fixed_regions::FixedRegions,
) -> MatchedStep {
  step(
    image,
    Axis::Vertical,
    Direction::Forward,
    shift,
    Some(shift),
    regions,
  )
}

/// Three tiles scrolling down: a plain first tile, a middle tile with a
/// masked sidebar band, and a terminal tile whose top rows are sticky header
/// chrome. `header_height` is the leading static strip the middle-to-terminal
/// pair reports; passing 0 reproduces the behaviour before the crop existed.
fn compose_terminal_header_case(header_height: u32) -> CapturedImage {
  let shift = 128;
  let masked = || fixed_regions::FixedRegions {
    bands: fixed_regions::FixedBands::masking(Axis::Vertical, &[(0, FILL)]),
    leading_strip: 0,
    trailing_strip: 0,
  };
  let mut terminal = solid(32, 256, FOOTER);
  paint_rows(&mut terminal, 0..32, HEADER);
  let frames = vec![
    MatchedStep {
      image: Arc::new(solid(32, 256, [80, 80, 80, 255])),
      movement: None,
      outcome: None,
    },
    vertical_step(solid(32, 256, [120, 120, 120, 255]), shift, masked()),
    vertical_step(
      terminal,
      shift,
      fixed_regions::FixedRegions {
        bands: fixed_regions::FixedBands::masking(Axis::Vertical, &[]),
        leading_strip: header_height,
        trailing_strip: 0,
      },
    ),
  ];
  align_and_compose(frames).unwrap()
}

#[test]
fn a_terminal_tiles_sticky_header_never_redeems_a_placeholder() {
  let image = compose_terminal_header_case(32);
  assert_eq!((image.width, image.height), (32, 512));
  // Rows the masked middle tile alone covers keep their placeholder fill.
  assert_eq!(at(&image, 0, 270), FILL);
  // Rows the terminal tile's real document pixels also cover are redeemed.
  assert_eq!(at(&image, 0, 300), FOOTER);
}

#[test]
fn without_the_leading_crop_the_header_leaks_into_the_masked_band() {
  let image = compose_terminal_header_case(0);
  assert_eq!(at(&image, 0, 270), HEADER);
}

#[test]
fn clamps_a_leading_crop_that_would_break_the_pairs_overlap() {
  // The whole frame reads as static, so an unclamped crop would leave the
  // terminal tile starting below where the previous tile stops, opening a
  // band of untouched canvas between them.
  let image = compose_terminal_header_case(200);
  assert!(image.rgba.chunks_exact(4).all(|pixel| pixel[3] != 0));
}

const FIRST: [u8; 4] = [255, 0, 0, 255];
const SECOND: [u8; 4] = [0, 255, 0, 255];
const THIRD: [u8; 4] = [0, 0, 255, 255];

fn plain_regions(
  axis: Axis,
  leading_strip: u32,
  trailing_strip: u32,
) -> fixed_regions::FixedRegions {
  fixed_regions::FixedRegions {
    bands: fixed_regions::FixedBands::masking(axis, &[]),
    leading_strip,
    trailing_strip,
  }
}

#[test]
fn a_rejected_pair_falls_back_to_the_scroll_we_asked_for() {
  let rejected_expected = 100;
  let matched_shift = 128;
  let frames = vec![
    MatchedStep {
      image: Arc::new(solid(32, 256, FIRST)),
      movement: None,
      outcome: None,
    },
    step(
      solid(32, 256, SECOND),
      Axis::Vertical,
      Direction::Forward,
      rejected_expected,
      None,
      plain_regions(Axis::Vertical, 0, 0),
    ),
    vertical_step(
      solid(32, 256, THIRD),
      matched_shift,
      plain_regions(Axis::Vertical, 0, 0),
    ),
  ];
  let image = align_and_compose(frames).unwrap();
  // 100 + 128 + one frame: the rejected pair contributes its expected scroll.
  assert_eq!((image.width, image.height), (32, 484));
  assert_eq!(at(&image, 0, 50), FIRST);
  // Below the first tile's trailing crop, so only the second tile can own it.
  assert_eq!(at(&image, 0, 200), SECOND);
  assert_eq!(at(&image, 0, 290), SECOND);
  assert_eq!(at(&image, 0, 300), THIRD);
}

/// A serpentine row travelling right-to-left, with a deep static strip at the
/// axis origin (the left edge) and a shallow one at the far edge.
fn serpentine_pair(direction: Direction) -> CapturedImage {
  let shift = 100;
  let frames = vec![
    MatchedStep {
      image: Arc::new(solid(256, 16, FIRST)),
      movement: None,
      outcome: None,
    },
    step(
      solid(256, 16, SECOND),
      Axis::Horizontal,
      direction,
      shift,
      Some(shift),
      plain_regions(Axis::Horizontal, 96, 8),
    ),
  ];
  align_and_compose(frames).unwrap()
}

#[test]
fn a_backward_pair_crops_the_chrome_on_its_overlapping_side() {
  let image = serpentine_pair(Direction::Backward);
  assert_eq!((image.width, image.height), (356, 16));
  // The previous tile's left crop follows the leading strip (96, above the
  // 64px floor) and the current tile's right crop the trailing strip (8), so
  // these columns belong to the current tile alone.
  assert_eq!(at(&image, 180, 8), SECOND);
  // New territory the previous tile never covered.
  assert_eq!(at(&image, 50, 8), SECOND);
  // Deep inside the previous tile, past the current tile's cropped edge.
  assert_eq!(at(&image, 300, 8), FIRST);
  assert!(image.rgba.chunks_exact(4).all(|pixel| pixel[3] != 0));
}

#[test]
fn a_forward_pair_keeps_cropping_the_current_tiles_origin_edge() {
  let image = serpentine_pair(Direction::Forward);
  assert_eq!((image.width, image.height), (356, 16));
  // The previous tile keeps the 64px floor (trailing strip 8) and the current
  // tile's leading crop of 96 clamps to 84, so it starts at column 184.
  assert_eq!(at(&image, 50, 8), FIRST);
  assert_eq!(at(&image, 170, 8), FIRST);
  assert_eq!(at(&image, 200, 8), SECOND);
  assert!(image.rgba.chunks_exact(4).all(|pixel| pixel[3] != 0));
}

#[test]
fn composes_overlapping_tiles_on_a_two_dimensional_canvas() {
  let tiles = vec![
    Tile {
      crop: Crop::default(),
      fixed_bands: None,
      image: Arc::new(solid(4, 4, [255, 0, 0, 255])),
      x: 0,
      y: 0,
    },
    Tile {
      crop: Crop::default(),
      fixed_bands: None,
      image: Arc::new(solid(4, 4, [0, 255, 0, 255])),
      x: 2,
      y: 0,
    },
    Tile {
      crop: Crop::default(),
      fixed_bands: None,
      image: Arc::new(solid(4, 4, [0, 0, 255, 255])),
      x: 2,
      y: 2,
    },
  ];
  let image = compose(tiles).unwrap();
  assert_eq!((image.width, image.height), (6, 6));
  assert_eq!(&image.rgba[0..4], &[255, 0, 0, 255]);
  let bottom_right = (((5 * image.width) + 5) * 4) as usize;
  assert_eq!(
    &image.rgba[bottom_right..bottom_right + 4],
    &[0, 0, 255, 255]
  );
}
