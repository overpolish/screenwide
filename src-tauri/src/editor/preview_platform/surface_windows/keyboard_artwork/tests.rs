// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;

fn overlay_with(codes: &[u16]) -> KeyboardOverlay {
  let mut overlay = KeyboardOverlay {
    key_count: codes.len() as u32,
    ..Default::default()
  };
  for (index, code) in codes.iter().enumerate() {
    overlay.keys[index] = KeyboardKey {
      key_code: *code,
      visible: 1,
      alpha: 1.0,
      scale: 1.0,
      progress: 1.0,
      slot: index as u32,
      ..Default::default()
    };
  }
  overlay
}

#[test]
fn keyboard_constants_match_the_shader_register_packing() {
  assert_eq!(size_of::<KeyboardConstants>(), 560);
  assert_eq!(size_of::<KeyboardConstants>() % 16, 0);
}

#[test]
fn key_caps_advance_by_a_four_point_gap_at_the_backing_scale() {
  let labels = ["Ctrl".to_owned(), "⇧".to_owned(), "P".to_owned()];
  let raster = rasterize_keyboard(&labels, true, 12.0).unwrap();

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
  let raster = rasterize_keyboard(&["Esc".to_owned()], false, 12.0).unwrap();

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
fn legacy_modifier_masks_expand_into_separate_caps() {
  let mut overlay = overlay_with(&[35]);
  overlay.keys[0].modifier_mask = 0b0000_0011;

  let prepared = prepared_shortcut(&overlay);

  assert_eq!(
    prepared.iter().map(|(code, _)| *code).collect::<Vec<_>>(),
    vec![55, 59, 35]
  );
}

#[test]
fn backing_scale_covers_the_animated_excursion_within_its_clamp() {
  let overlay = overlay_with(&[35]);

  assert_eq!(keyboard_backing_scale(1080, &overlay), 12.0);
  assert_eq!(keyboard_backing_scale(8640, &overlay), 26.0);
  assert_eq!(keyboard_backing_scale(u32::MAX, &overlay), 64.0);
}
