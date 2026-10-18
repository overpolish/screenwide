// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;
use labels::Icon;

#[test]
fn key_caps_advance_by_a_four_point_gap_at_the_backing_scale() {
  let caps = [
    Cap::Text("Ctrl".to_owned()),
    Cap::Icon(Icon::Shift),
    Cap::Text("P".to_owned()),
  ];
  let raster = rasterize_caps(&caps, true, 12.0).unwrap();

  assert_eq!(raster.keys.len(), caps.len());
  assert_eq!(raster.keys[0].0, 0);
  for key in &raster.keys {
    assert!(key.1 > 0, "every key cap has a positive width");
  }
  for pair in raster.keys.windows(2) {
    let gap = i64::from(pair[1].0) - i64::from(pair[0].0 + pair[0].1);
    assert_eq!(gap, (DESIGN_GAP * 12.0) as i64);
  }
  let last = raster.keys.last().copied().unwrap();
  assert!(last.0 + last.1 <= raster.size.0);
  assert_eq!(raster.size.1, (DESIGN_HEIGHT * 12.0) as u32);
}

#[test]
fn artwork_pixels_stay_premultiplied() {
  let raster = rasterize_caps(&[Cap::Text("Esc".to_owned())], false, 12.0).unwrap();

  assert_eq!(
    raster.pixels.len(),
    raster.size.0 as usize * raster.size.1 as usize * 4
  );
  assert!(raster
    .pixels
    .chunks_exact(4)
    .all(|pixel| pixel[0] <= pixel[3] && pixel[1] <= pixel[3] && pixel[2] <= pixel[3]));
  assert!(raster
    .pixels
    .chunks_exact(4)
    .any(|pixel| pixel[3] > 0 && pixel[3] < 255));
  assert!(raster.pixels.chunks_exact(4).any(|pixel| pixel[3] == 0));
}

#[test]
fn a_label_is_drawn_inside_its_cap() {
  let raster = rasterize_caps(&[Cap::Text("M".to_owned())], true, 12.0).unwrap();
  let (width, height) = raster.size;
  // Dark text over the light cap: some pixel well inside is much darker
  // than the cap's own fill.
  let darkest = (height / 4..height * 3 / 4)
    .flat_map(|row| (width / 4..width * 3 / 4).map(move |column| (row * width + column) as usize))
    .map(|index| raster.pixels[index * 4])
    .min()
    .unwrap();
  assert!(darkest < 120, "the label's ink shows: {darkest}");
}
