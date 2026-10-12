// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;

#[test]
fn key_caps_advance_by_a_four_point_gap_at_the_backing_scale() {
  let labels = ["Ctrl".to_owned(), "⇧".to_owned(), "P".to_owned()];
  let raster = rasterize_labels(&labels, true, 12.0).unwrap();

  assert_eq!(raster.keys.len(), labels.len());
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
  let raster = rasterize_labels(&["Esc".to_owned()], false, 12.0).unwrap();

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
