// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::super::band_needs::Read;
use super::*;

const LIMIT: u32 = 8192;

fn upright(source: (u32, u32), image_rect: [f64; 4]) -> LayerNeeds {
  LayerNeeds {
    source,
    image_rect,
    inverse_turn: None,
    reads: Vec::new(),
    margin: 0,
    overlap: 0,
    reads_past: 0,
  }
}

/// Every canvas pixel is kept by exactly one band, and every target and
/// window fits the device.
fn assert_covers(bands: &[Band], canvas: (u32, u32)) {
  let mut kept = vec![0_u8; canvas.0 as usize * canvas.1 as usize];
  for band in bands {
    let target = band.target;
    assert!(
      target.size.0 <= LIMIT && target.size.1 <= LIMIT,
      "{target:?}"
    );
    let [x0, y0, x1, y1] = band.kept;
    assert!(x0 >= target.origin.0 && y0 >= target.origin.1);
    assert!(x1 <= target.origin.0 + target.size.0 && y1 <= target.origin.1 + target.size.1);
    for window in band.windows.iter().flatten() {
      assert!(window[2] - window[0] <= LIMIT && window[3] - window[1] <= LIMIT);
    }
    for y in y0..y1 {
      for x in x0..x1 {
        kept[(y * canvas.0 + x) as usize] += 1;
      }
    }
  }
  assert!(kept.iter().all(|&count| count == 1));
}

#[test]
fn a_canvas_that_fits_is_one_band_from_whole_pictures() {
  let bands = plan(
    (1000, 800),
    LIMIT,
    &[upright((1000, 800), [0.0, 0.0, 1000.0, 800.0])],
  );
  assert_eq!(bands.len(), 1);
  assert_eq!(bands[0].kept, [0, 0, 1000, 800]);
  assert_eq!(bands[0].windows, vec![None]);
}

#[test]
fn a_tall_picture_is_read_through_windows_holding_the_rows_each_band_shows() {
  let canvas = (1100, 20_100);
  let layer = upright((1000, 20_000), [50.0, 50.0, 1000.0, 20_000.0]);
  let bands = plan(canvas, LIMIT, &[layer]);
  assert!(bands.len() > 2);
  assert_covers(&bands, canvas);
  for band in &bands {
    let window = band.windows[0].expect("the picture is too tall to upload whole");
    let shown_top = band.kept[1].saturating_sub(50).min(20_000);
    let shown_bottom = band.kept[3].saturating_sub(50).min(20_000);
    assert!(
      window[1] <= shown_top && window[3] >= shown_bottom,
      "{band:?}"
    );
  }
}

#[test]
fn a_picture_drawn_smaller_than_it_is_takes_bands_its_windows_fit() {
  // The canvas fits the device, but the picture is drawn at a quarter of
  // its size, so a band shows four times its own height of it.
  let canvas = (500, 6000);
  let layer = upright((2000, 24_000), [0.0, 0.0, 500.0, 6000.0]);
  let bands = plan(canvas, LIMIT, &[layer]);
  assert!(bands.len() >= 3);
  assert_covers(&bands, canvas);
}

#[test]
fn a_lens_taller_than_the_device_is_read_where_each_band_shows_it() {
  // 18,000 rows of lens at twice the size: each of its pixels shows the
  // picture half as far from its middle, row 10,000.
  let canvas = (1000, 20_000);
  let mut layer = upright((1000, 20_000), [0.0, 0.0, 1000.0, 20_000.0]);
  layer.reads.push(Read::Lens {
    bounds: [100.0, 1000.0, 400.0, 19_000.0],
    zoom: 2.0,
    bend: 0.0,
    pad: 2.0,
  });
  let bands = plan(canvas, LIMIT, &[layer]);
  assert_covers(&bands, canvas);
  for band in &bands {
    let window = band.windows[0].expect("windowed");
    let shows = |row: u32| 10_000.0 + (f64::from(row.clamp(1000, 19_000)) - 10_000.0) / 2.0;
    let (top, bottom) = (shows(band.kept[1]), shows(band.kept[3]));
    assert!(
      f64::from(window[1]) <= top && f64::from(window[3]) >= bottom,
      "{band:?} reads rows {top} to {bottom}"
    );
  }
}

#[test]
fn a_focus_blur_draws_each_band_past_the_part_it_keeps() {
  let canvas = (1000, 20_000);
  let mut layer = upright((1000, 20_000), [0.0, 0.0, 1000.0, 20_000.0]);
  layer.overlap = 200;
  let bands = plan(canvas, LIMIT, &[layer]);
  assert_covers(&bands, canvas);
  let middle = &bands[1];
  assert_eq!(middle.target.origin.1 + 200, middle.kept[1]);
  assert_eq!(
    middle.target.origin.1 + middle.target.size.1,
    middle.kept[3] + 200
  );
}

#[test]
fn a_turned_picture_reads_the_rows_its_turn_carries_into_each_band() {
  // Mirrored top to bottom about the canvas's middle: the top band shows
  // the picture's bottom rows.
  let canvas = (1000, 20_000);
  let mut layer = upright((1000, 20_000), [0.0, 0.0, 1000.0, 20_000.0]);
  layer.inverse_turn = Some([[1.0, 0.0, 0.0], [0.0, -1.0, 20_000.0], [0.0, 0.0, 1.0]]);
  let bands = plan(canvas, LIMIT, &[layer]);
  assert_covers(&bands, canvas);
  let top = bands[0].windows[0].expect("windowed");
  assert_eq!(top[3], 20_000);
  assert!(top[1] >= 20_000 - bands[0].target.size.1 - 4);
}
