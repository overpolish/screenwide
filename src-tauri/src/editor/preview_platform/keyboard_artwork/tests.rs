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
