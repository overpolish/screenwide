// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;

fn v1() -> KeyboardCompositor {
  KeyboardCompositor::from_shortcuts(vec![Shortcut {
    keys: vec![KeyPress {
      key_code: 0,
      modifier_mask: 1,
      down_us: 1_000_000,
      up_us: None,
    }],
  }])
}

#[test]
fn v1_fallback_is_one_key() {
  let overlay = v1().evaluate(1_100, Default::default()).unwrap();
  assert_eq!(overlay.key_count, 1);
  assert_eq!(overlay.keys[0].key_code, 0);
}

#[test]
fn native_payload_layout_is_stable() {
  assert_eq!(std::mem::size_of::<KeyboardKey>(), 52);
  assert_eq!(std::mem::size_of::<KeyboardOverlay>(), 452);
}

#[test]
fn v2_keys_stagger_independently() {
  let records = vec![
    serde_json::json!({"type":"keyDown","keyCode":55,"timestampUs":1_000_000,"modifiers":["command"]}),
    serde_json::json!({"type":"keyDown","keyCode":0,"timestampUs":1_100_000,"modifiers":["command"]}),
  ];
  let compositor = KeyboardCompositor::from_shortcuts(reconstruct_v2(&records));
  let overlay = compositor.evaluate(1_150, Default::default()).unwrap();
  assert_eq!(overlay.key_count, 2);
  assert_eq!(overlay.keys[0].key_code, 55);
  let first = overlay.keys.iter().find(|key| key.key_code == 55).unwrap();
  let second = overlay.keys.iter().find(|key| key.key_code == 0).unwrap();
  assert!(second.progress < first.progress);
}

#[test]
fn modifiers_follow_press_order_and_the_primary_key_stays_last() {
  let records = vec![
    serde_json::json!({"type":"keyDown","keyCode":60,"timestampUs":1_000_000,"modifiers":["shift"]}),
    serde_json::json!({"type":"keyDown","keyCode":55,"timestampUs":1_100_000,"modifiers":["shift","command"]}),
    serde_json::json!({"type":"keyDown","keyCode":0,"timestampUs":1_200_000,"modifiers":["shift","command"]}),
    serde_json::json!({"type":"keyUp","keyCode":0,"timestampUs":1_300_000,"modifiers":["shift","command"]}),
    serde_json::json!({"type":"keyUp","keyCode":55,"timestampUs":1_400_000,"modifiers":["shift"]}),
    serde_json::json!({"type":"keyUp","keyCode":60,"timestampUs":1_500_000,"modifiers":[]}),
    serde_json::json!({"type":"keyDown","keyCode":55,"timestampUs":4_000_000,"modifiers":["command"]}),
    serde_json::json!({"type":"keyDown","keyCode":60,"timestampUs":4_100_000,"modifiers":["command","shift"]}),
    serde_json::json!({"type":"keyDown","keyCode":0,"timestampUs":4_200_000,"modifiers":["command","shift"]}),
  ];
  let compositor = KeyboardCompositor::from_shortcuts(reconstruct_v2(&records));

  let shift_first = compositor.evaluate(1_800, Default::default()).unwrap();
  let shift_first_codes = shift_first.keys[..shift_first.key_count as usize]
    .iter()
    .filter(|key| key.alpha > 0.0)
    .map(|key| key.key_code)
    .collect::<Vec<_>>();
  assert_eq!(shift_first_codes, vec![60, 55, 0]);

  let command_first = compositor.evaluate(4_800, Default::default()).unwrap();
  let command_first_codes = command_first.keys[..command_first.key_count as usize]
    .iter()
    .filter(|key| key.alpha > 0.0)
    .map(|key| key.key_code)
    .collect::<Vec<_>>();
  assert_eq!(command_first_codes, vec![55, 60, 0]);
}

#[test]
fn the_recorded_two_modifiers_move_together_when_the_primary_arrives() {
  let records = vec![
    serde_json::json!({"type":"keyDown","keyCode":56,"timestampUs":3_164_380,"modifiers":["shift"]}),
    serde_json::json!({"type":"keyDown","keyCode":55,"timestampUs":3_380_366,"modifiers":["shift","command"]}),
    serde_json::json!({"type":"keyDown","keyCode":1,"timestampUs":3_797_036,"modifiers":["shift","command"]}),
  ];
  let compositor = KeyboardCompositor::from_shortcuts(reconstruct_v2(&records));

  let assembling = compositor.evaluate(4_100, Default::default()).unwrap();
  assert_eq!(visible_codes(&assembling), vec![56, 55, 1]);
  assert_shared_layout(&assembling, &[56, 55, 1]);
}

#[test]
fn the_recorded_shift_retirement_reflows_while_it_fades_without_a_late_jump() {
  let records = vec![
    serde_json::json!({"type":"keyDown","keyCode":56,"timestampUs":3_164_380,"modifiers":["shift"]}),
    serde_json::json!({"type":"keyDown","keyCode":55,"timestampUs":3_380_366,"modifiers":["shift","command"]}),
    serde_json::json!({"type":"keyDown","keyCode":1,"timestampUs":3_797_036,"modifiers":["shift","command"]}),
    serde_json::json!({"type":"keyUp","keyCode":1,"timestampUs":3_888_610,"modifiers":["shift","command"]}),
    serde_json::json!({"type":"keyUp","keyCode":56,"timestampUs":4_191_382,"modifiers":["command"]}),
    serde_json::json!({"type":"keyDown","keyCode":1,"timestampUs":4_891_185,"modifiers":["command"]}),
    serde_json::json!({"type":"keyUp","keyCode":1,"timestampUs":4_989_092,"modifiers":["command"]}),
    serde_json::json!({"type":"keyUp","keyCode":55,"timestampUs":5_122_310,"modifiers":[]}),
  ];
  let compositor = KeyboardCompositor::from_shortcuts(reconstruct_v2(&records));

  let reflowing = compositor.evaluate(5_000, Default::default()).unwrap();
  let command = reflowing
    .keys
    .iter()
    .find(|key| key.key_code == 55 && key.alpha > 0.0)
    .unwrap();
  let shift = reflowing
    .keys
    .iter()
    .find(|key| key.key_code == 56)
    .unwrap();
  let second_s = reflowing
    .keys
    .iter()
    .find(|key| key.key_code == 1 && key.visible == 1)
    .unwrap();
  assert_ne!(command.layout_from_mask, command.layout_to_mask);
  assert_eq!(command.layout_from_mask, second_s.layout_from_mask);
  assert_eq!(command.layout_to_mask, second_s.layout_to_mask);
  assert_eq!(command.layout_progress, second_s.layout_progress);
  // Exit timing is shared with visibility; the detached key's alpha and
  // scale should still track one another during the shorter exit.
  assert!((shift.alpha - shift.scale).abs() < 0.001);

  let leaving = compositor.evaluate(6_100, Default::default()).unwrap();
  assert_eq!(visible_codes(&leaving), vec![55, 1]);
  for key in &leaving.keys[..leaving.key_count as usize] {
    assert_eq!(key.layout_from_mask, key.layout_to_mask);
  }
}

#[test]
fn four_rapid_modifiers_join_one_primary_layout_transaction() {
  let records = vec![
    serde_json::json!({"type":"keyDown","keyCode":56,"timestampUs":1_000_000,"modifiers":["shift"]}),
    serde_json::json!({"type":"keyDown","keyCode":55,"timestampUs":1_100_000,"modifiers":["shift","command"]}),
    serde_json::json!({"type":"keyDown","keyCode":58,"timestampUs":1_200_000,"modifiers":["shift","command","option"]}),
    serde_json::json!({"type":"keyDown","keyCode":59,"timestampUs":1_300_000,"modifiers":["shift","command","option","control"]}),
    serde_json::json!({"type":"keyDown","keyCode":0,"timestampUs":1_400_000,"modifiers":["shift","command","option","control"]}),
  ];
  let compositor = KeyboardCompositor::from_shortcuts(reconstruct_v2(&records));

  let assembling = compositor.evaluate(1_800, Default::default()).unwrap();
  assert_eq!(visible_codes(&assembling), vec![56, 55, 58, 59, 0]);
  assert_shared_layout(&assembling, &[56, 55, 58, 59, 0]);
}

#[test]
fn a_late_modifier_inserts_before_the_primary_on_the_shared_clock() {
  let records = vec![
    serde_json::json!({"type":"keyDown","keyCode":56,"timestampUs":1_000_000,"modifiers":["shift"]}),
    serde_json::json!({"type":"keyDown","keyCode":55,"timestampUs":1_700_000,"modifiers":["shift","command"]}),
    serde_json::json!({"type":"keyDown","keyCode":8,"timestampUs":2_400_000,"modifiers":["shift","command"]}),
    serde_json::json!({"type":"keyDown","keyCode":58,"timestampUs":3_200_000,"modifiers":["shift","command","option"]}),
  ];
  let compositor = KeyboardCompositor::from_shortcuts(reconstruct_v2(&records));

  let inserting = compositor.evaluate(3_300, Default::default()).unwrap();
  assert_eq!(visible_codes(&inserting), vec![56, 55, 58, 8]);
  assert_shared_layout(&inserting, &[56, 55, 58, 8]);
}

fn visible_codes(overlay: &KeyboardOverlay) -> Vec<u16> {
  overlay.keys[..overlay.key_count as usize]
    .iter()
    .filter(|key| key.alpha > 0.0)
    .map(|key| key.key_code)
    .collect()
}

fn assert_shared_layout(overlay: &KeyboardOverlay, key_codes: &[u16]) {
  let tracks = key_codes
    .iter()
    .map(|code| {
      let key = overlay
        .keys
        .iter()
        .find(|key| key.key_code == *code && key.alpha > 0.0)
        .unwrap();
      (
        key.layout_from_mask,
        key.layout_to_mask,
        key.layout_progress,
      )
    })
    .collect::<Vec<_>>();
  assert!(tracks.windows(2).all(|pair| pair[0] == pair[1]));
}

#[test]
fn a_released_letter_exits_after_idle_while_its_modifier_stays_held() {
  let records = vec![
    serde_json::json!({"type":"keyDown","keyCode":55,"timestampUs":1_000_000,"modifiers":["command"]}),
    serde_json::json!({"type":"keyDown","keyCode":0,"timestampUs":1_100_000,"modifiers":["command"]}),
    serde_json::json!({"type":"keyUp","keyCode":0,"timestampUs":1_200_000,"modifiers":["command"]}),
  ];
  let compositor = KeyboardCompositor::from_shortcuts(reconstruct_v2(&records));

  let holding = compositor.evaluate(3_000, Default::default()).unwrap();
  let visible = holding.keys[..holding.key_count as usize]
    .iter()
    .filter(|key| key.alpha > 0.0)
    .collect::<Vec<_>>();
  assert_eq!(visible.len(), 1);
  assert_eq!(visible[0].key_code, 55);
  assert_eq!(visible[0].visible, 1);
}

#[test]
fn the_whole_row_leaves_together_once_the_last_key_is_released() {
  let records = vec![
    serde_json::json!({"type":"keyDown","keyCode":55,"timestampUs":1_000_000,"modifiers":["command"]}),
    serde_json::json!({"type":"keyDown","keyCode":0,"timestampUs":1_100_000,"modifiers":["command"]}),
    serde_json::json!({"type":"keyUp","keyCode":0,"timestampUs":1_200_000,"modifiers":["command"]}),
    serde_json::json!({"type":"keyUp","keyCode":55,"timestampUs":1_400_000,"modifiers":[]}),
  ];
  let compositor = KeyboardCompositor::from_shortcuts(reconstruct_v2(&records));
  let held = compositor.evaluate(2_100, Default::default()).unwrap();
  assert_eq!(held.key_count, 2);
  assert!(held.keys[..2].iter().all(|key| key.visible == 1));

  let leaving = compositor.evaluate(2_200, Default::default()).unwrap();
  assert_eq!(leaving.key_count, 2);
  assert!(leaving.keys[..2].iter().all(|key| key.visible == 2));
  assert_eq!(leaving.keys[0].progress, leaving.keys[1].progress);
  assert!(leaving.keys[..2]
    .iter()
    .all(|key| key.layout_from_mask == key.layout_to_mask));

  assert!(compositor.evaluate(2_750, Default::default()).is_none());
}

#[test]
fn adding_a_primary_animates_a_new_layout_slot_for_six_tenths() {
  let records = vec![
    serde_json::json!({"type":"keyDown","keyCode":55,"timestampUs":7_276_454,"modifiers":["command"]}),
    serde_json::json!({"type":"keyDown","keyCode":8,"timestampUs":8_376_825,"modifiers":["command"]}),
  ];
  let compositor = KeyboardCompositor::from_shortcuts(reconstruct_v2(&records));

  let start = compositor.evaluate(8_376, Default::default()).unwrap();
  assert_eq!(start.key_count, 1);
  let first_frame = compositor.evaluate(8_377, Default::default()).unwrap();
  let command = first_frame
    .keys
    .iter()
    .find(|key| key.key_code == 55)
    .unwrap();
  let letter = first_frame
    .keys
    .iter()
    .find(|key| key.key_code == 8)
    .unwrap();
  assert!(command.layout_progress < 0.01);
  assert_ne!(command.layout_from_mask, command.layout_to_mask);
  assert_eq!(letter.layout_progress, command.layout_progress);
  assert_eq!(letter.layout_from_mask, command.layout_from_mask);
  assert_eq!(letter.layout_to_mask, command.layout_to_mask);

  let middle = compositor.evaluate(8_677, Default::default()).unwrap();
  let command = middle.keys.iter().find(|key| key.key_code == 55).unwrap();
  assert!((command.layout_progress - 0.5).abs() < 0.01);

  let settled = compositor.evaluate(8_977, Default::default()).unwrap();
  let command = settled.keys.iter().find(|key| key.key_code == 55).unwrap();
  assert_eq!(command.layout_progress, 1.0);
}

#[test]
fn an_outgoing_chords_unused_slot_collapses_and_cannot_be_reused_later() {
  let records = vec![
    serde_json::json!({"type":"keyDown","keyCode":55,"timestampUs":5_272_233,"modifiers":["command"]}),
    serde_json::json!({"type":"keyDown","keyCode":0,"timestampUs":5_382_186,"modifiers":["command"]}),
    serde_json::json!({"type":"keyUp","keyCode":0,"timestampUs":5_473_690,"modifiers":["command"]}),
    serde_json::json!({"type":"keyDown","keyCode":8,"timestampUs":6_164_356,"modifiers":["command"]}),
    serde_json::json!({"type":"keyUp","keyCode":8,"timestampUs":6_225_880,"modifiers":["command"]}),
    serde_json::json!({"type":"keyUp","keyCode":55,"timestampUs":6_273_893,"modifiers":[]}),
    serde_json::json!({"type":"keyDown","keyCode":55,"timestampUs":7_276_454,"modifiers":["command"]}),
    serde_json::json!({"type":"keyDown","keyCode":8,"timestampUs":8_376_825,"modifiers":["command"]}),
  ];
  let compositor = KeyboardCompositor::from_shortcuts(reconstruct_v2(&records));

  let collapse = compositor.evaluate(7_700, Default::default()).unwrap();
  let outgoing = collapse.keys.iter().find(|key| key.key_code == 8).unwrap();
  assert_eq!(outgoing.alpha, 0.0);
  assert_eq!(outgoing.layout_from_mask, outgoing.layout_to_mask);

  let addition = compositor.evaluate(8_377, Default::default()).unwrap();
  let arriving = addition.keys.iter().find(|key| key.key_code == 8).unwrap();
  let command = addition.keys.iter().find(|key| key.key_code == 55).unwrap();
  assert!(arriving.progress < 0.01);
  assert_eq!(arriving.layout_from_mask, command.layout_from_mask);
  assert_eq!(arriving.layout_to_mask, command.layout_to_mask);
  assert_eq!(arriving.layout_progress, command.layout_progress);
  assert!(command.layout_progress > 0.0 && command.layout_progress < 1.0);
  assert_ne!(command.layout_from_mask, command.layout_to_mask);
}

#[test]
fn an_initial_key_uses_the_full_six_tenths_entrance() {
  let compositor = KeyboardCompositor::from_shortcuts(vec![Shortcut {
    keys: vec![KeyPress {
      key_code: 60,
      modifier_mask: 1 << 3,
      down_us: 1_000_000,
      up_us: None,
    }],
  }]);
  let middle = compositor.evaluate(1_300, Default::default()).unwrap();
  assert!((middle.keys[0].progress - 0.5).abs() < 0.001);
}

#[test]
fn repressing_a_released_modifier_crossfades_in_its_existing_slot() {
  let records = vec![
    serde_json::json!({"type":"keyDown","keyCode":55,"timestampUs":1_000_000,"modifiers":["command"]}),
    serde_json::json!({"type":"keyUp","keyCode":55,"timestampUs":1_200_000,"modifiers":[]}),
    serde_json::json!({"type":"keyDown","keyCode":55,"timestampUs":2_000_000,"modifiers":["command"]}),
  ];
  let compositor = KeyboardCompositor::from_shortcuts(reconstruct_v2(&records));
  assert_eq!(compositor.visuals_snapshot().len(), 2);
  let overlay = compositor.evaluate(2_100, Default::default()).unwrap();
  assert_eq!(overlay.key_count, 2);
  assert_eq!(overlay.keys[0].slot, overlay.keys[1].slot);
  assert!(overlay.keys[..2].iter().any(|key| key.visible == 2));
  assert!(overlay.keys[..2].iter().any(|key| key.visible == 1));
}

#[test]
fn a_second_letter_swaps_into_the_slot_of_the_released_one() {
  // Copy, then paste, with Command held throughout: the C leaves as the V
  // arrives instead of the row growing to "Command C V".
  let records = vec![
    serde_json::json!({"type":"keyDown","keyCode":55,"timestampUs":1_000_000,"modifiers":["command"]}),
    serde_json::json!({"type":"keyDown","keyCode":8,"timestampUs":1_100_000,"modifiers":["command"]}),
    serde_json::json!({"type":"keyUp","keyCode":8,"timestampUs":1_200_000,"modifiers":["command"]}),
    serde_json::json!({"type":"keyDown","keyCode":9,"timestampUs":2_100_000,"modifiers":["command"]}),
  ];
  let compositor = KeyboardCompositor::from_shortcuts(reconstruct_v2(&records));

  let swapping = compositor.evaluate(2_230, Default::default()).unwrap();
  assert_eq!(swapping.key_count, 3);
  let command = swapping.keys.iter().find(|key| key.key_code == 55).unwrap();
  let leaving = swapping.keys.iter().find(|key| key.key_code == 8).unwrap();
  let arriving = swapping.keys.iter().find(|key| key.key_code == 9).unwrap();
  assert_eq!(command.visible, 1);
  assert_eq!(command.layout_from_mask, command.layout_to_mask);
  assert_eq!(leaving.visible, 2);
  assert_eq!(arriving.visible, 1);
  assert_eq!(leaving.slot, arriving.slot);
  // The outgoing label fades out even in pop mode so the two never overlap
  // into one wide key.
  assert!(leaving.alpha < 1.0);

  let settled = compositor.evaluate(2_700, Default::default()).unwrap();
  assert_eq!(settled.key_count, 2);
  assert_eq!(settled.keys[1].key_code, 9);
}

#[test]
fn replacement_labels_trade_places_instead_of_double_exposing() {
  let records = vec![
    serde_json::json!({"type":"keyDown","keyCode":55,"timestampUs":1_000_000,"modifiers":["command"]}),
    serde_json::json!({"type":"keyDown","keyCode":8,"timestampUs":1_100_000,"modifiers":["command"]}),
    serde_json::json!({"type":"keyUp","keyCode":8,"timestampUs":1_200_000,"modifiers":["command"]}),
    serde_json::json!({"type":"keyDown","keyCode":9,"timestampUs":1_500_000,"modifiers":["command"]}),
  ];
  let compositor = KeyboardCompositor::from_shortcuts(reconstruct_v2(&records));

  let start = compositor.evaluate(1_500, Default::default()).unwrap();
  let leaving = start.keys.iter().find(|key| key.key_code == 8).unwrap();
  let arriving = start.keys.iter().find(|key| key.key_code == 9).unwrap();
  assert_eq!(leaving.alpha, 1.0);
  assert_eq!(arriving.alpha, 0.0);

  let middle = compositor.evaluate(1_700, Default::default()).unwrap();
  let leaving = middle.keys.iter().find(|key| key.key_code == 8).unwrap();
  let arriving = middle.keys.iter().find(|key| key.key_code == 9).unwrap();
  assert_eq!(leaving.slot, arriving.slot);
  assert!(leaving.alpha < 0.5);
  assert!(arriving.alpha < 0.9);
}

#[test]
fn a_new_chord_replaces_a_released_modifier_instead_of_extending_its_tail() {
  let records = vec![
    serde_json::json!({"type":"keyDown","keyCode":60,"timestampUs":1_000_000,"modifiers":["shift"]}),
    serde_json::json!({"type":"keyUp","keyCode":60,"timestampUs":1_100_000,"modifiers":[]}),
    serde_json::json!({"type":"keyDown","keyCode":55,"timestampUs":1_500_000,"modifiers":["command"]}),
    serde_json::json!({"type":"keyDown","keyCode":0,"timestampUs":1_600_000,"modifiers":["command"]}),
  ];
  let compositor = KeyboardCompositor::from_shortcuts(reconstruct_v2(&records));
  let transition = compositor.evaluate(1_700, Default::default()).unwrap();
  let shift = transition
    .keys
    .iter()
    .find(|key| key.key_code == 60)
    .unwrap();
  let command = transition
    .keys
    .iter()
    .find(|key| key.key_code == 55)
    .unwrap();
  let letter = transition
    .keys
    .iter()
    .find(|key| key.key_code == 0)
    .unwrap();
  assert_eq!(shift.visible, 2);
  assert_eq!(shift.slot, command.slot);
  assert_ne!(shift.layout_from_mask, shift.layout_to_mask);
  assert_ne!(command.layout_from_mask, command.layout_to_mask);
  assert!(command.layout_progress > 0.0 && command.layout_progress < 1.0);
  assert_eq!(shift.layout_from_mask, command.layout_from_mask);
  assert_eq!(shift.layout_to_mask, command.layout_to_mask);
  assert_eq!(shift.layout_progress, command.layout_progress);
  assert_ne!(command.slot, letter.slot);
  assert!(shift.alpha < 1.0);

  let settled = compositor.evaluate(2_100, Default::default()).unwrap();
  assert_eq!(settled.key_count, 2);
  assert!(settled.keys[..2].iter().all(|key| key.key_code != 60));
}

#[test]
fn a_new_primary_replaces_the_previous_primary_even_if_it_is_still_held() {
  let records = vec![
    serde_json::json!({"type":"keyDown","keyCode":8,"timestampUs":1_000_000,"modifiers":[]}),
    serde_json::json!({"type":"keyDown","keyCode":9,"timestampUs":1_100_000,"modifiers":[]}),
  ];
  let compositor = KeyboardCompositor::from_shortcuts(reconstruct_v2(&records));
  let overlay = compositor.evaluate(1_200, Default::default()).unwrap();
  assert_eq!(overlay.key_count, 2);
  assert_eq!(overlay.keys[0].slot, overlay.keys[1].slot);
  assert!(overlay.keys[..2].iter().any(|key| key.visible == 2));
  assert!(overlay.keys[..2].iter().any(|key| key.visible == 1));
}

#[test]
fn the_recorded_copy_then_paste_never_shows_both_letters_settled() {
  let records = vec![
    serde_json::json!({"type":"keyDown","keyCode":55,"timestampUs":7_276_454,"modifiers":["command"]}),
    serde_json::json!({"type":"keyDown","keyCode":8,"timestampUs":8_376_825,"modifiers":["command"]}),
    serde_json::json!({"type":"keyUp","keyCode":8,"timestampUs":8_492_895,"modifiers":["command"]}),
    serde_json::json!({"type":"keyDown","keyCode":9,"timestampUs":9_660_839,"modifiers":["command"]}),
    serde_json::json!({"type":"keyUp","keyCode":9,"timestampUs":9_730_693,"modifiers":["command"]}),
    serde_json::json!({"type":"keyUp","keyCode":55,"timestampUs":9_789_248,"modifiers":[]}),
  ];
  let compositor = KeyboardCompositor::from_shortcuts(reconstruct_v2(&records));
  for position_ms in 7_276..10_600 {
    let Some(overlay) = compositor.evaluate(position_ms, Default::default()) else {
      continue;
    };
    let settled = overlay.keys[..overlay.key_count as usize]
      .iter()
      .filter(|key| !is_modifier_key(key.key_code) && key.visible == 1 && key.alpha > 0.0)
      .count();
    assert!(settled <= 1, "two letters share the row at {position_ms}ms");
  }
}

#[test]
fn the_reported_recording_keeps_replacement_slots_and_moves_without_jumps() {
  let records = vec![
    serde_json::json!({"type":"keyDown","keyCode":60,"timestampUs":2_571_571,"modifiers":["shift"]}),
    serde_json::json!({"type":"keyUp","keyCode":60,"timestampUs":2_807_540,"modifiers":[]}),
    serde_json::json!({"type":"keyDown","keyCode":55,"timestampUs":3_556_366,"modifiers":["command"]}),
    serde_json::json!({"type":"keyDown","keyCode":0,"timestampUs":3_686_272,"modifiers":["command"]}),
    serde_json::json!({"type":"keyUp","keyCode":0,"timestampUs":3_777_615,"modifiers":["command"]}),
    serde_json::json!({"type":"keyUp","keyCode":55,"timestampUs":4_286_034,"modifiers":[]}),
    serde_json::json!({"type":"keyDown","keyCode":55,"timestampUs":5_272_233,"modifiers":["command"]}),
    serde_json::json!({"type":"keyDown","keyCode":0,"timestampUs":5_382_186,"modifiers":["command"]}),
    serde_json::json!({"type":"keyUp","keyCode":0,"timestampUs":5_473_690,"modifiers":["command"]}),
    serde_json::json!({"type":"keyDown","keyCode":8,"timestampUs":6_164_356,"modifiers":["command"]}),
    serde_json::json!({"type":"keyUp","keyCode":8,"timestampUs":6_225_880,"modifiers":["command"]}),
    serde_json::json!({"type":"keyUp","keyCode":55,"timestampUs":6_273_893,"modifiers":[]}),
    serde_json::json!({"type":"keyDown","keyCode":55,"timestampUs":7_276_454,"modifiers":["command"]}),
    serde_json::json!({"type":"keyDown","keyCode":8,"timestampUs":8_376_825,"modifiers":["command"]}),
    serde_json::json!({"type":"keyUp","keyCode":8,"timestampUs":8_492_895,"modifiers":["command"]}),
    serde_json::json!({"type":"keyDown","keyCode":9,"timestampUs":9_660_839,"modifiers":["command"]}),
    serde_json::json!({"type":"keyUp","keyCode":9,"timestampUs":9_730_693,"modifiers":["command"]}),
    serde_json::json!({"type":"keyUp","keyCode":55,"timestampUs":9_789_248,"modifiers":[]}),
  ];
  let compositor = KeyboardCompositor::from_shortcuts(reconstruct_v2(&records));

  let visuals = compositor.visuals_snapshot();
  let modifier_slots = visuals
    .iter()
    .filter(|visual| is_modifier_key(visual.key_code))
    .map(|visual| visual.slot_id)
    .collect::<std::collections::HashSet<_>>();
  let primary_slots = visuals
    .iter()
    .filter(|visual| !is_modifier_key(visual.key_code))
    .map(|visual| visual.slot_id)
    .collect::<std::collections::HashSet<_>>();
  assert_eq!(modifier_slots.len(), 1);
  assert_eq!(primary_slots.len(), 1);

  let switching = compositor.evaluate(3_800, Default::default()).unwrap();
  let shift = switching
    .keys
    .iter()
    .find(|key| key.key_code == 60)
    .unwrap();
  let command = switching
    .keys
    .iter()
    .find(|key| key.key_code == 55)
    .unwrap();
  assert_ne!(shift.layout_from_mask, shift.layout_to_mask);
  assert_ne!(command.layout_from_mask, command.layout_to_mask);
  assert!(command.layout_progress > 0.0 && command.layout_progress < 1.0);
  assert_eq!(shift.layout_from_mask, command.layout_from_mask);
  assert_eq!(shift.layout_to_mask, command.layout_to_mask);
  assert_eq!(shift.layout_progress, command.layout_progress);

  let recentered = compositor.evaluate(7_900, Default::default()).unwrap();
  let command = recentered
    .keys
    .iter()
    .find(|key| key.key_code == 55 && key.visible == 1)
    .unwrap();
  assert_eq!(command.layout_from_mask, command.layout_to_mask);

  let swapping = compositor.evaluate(9_700, Default::default()).unwrap();
  let command = swapping
    .keys
    .iter()
    .find(|key| key.key_code == 55 && key.visible == 1)
    .unwrap();
  let paste = swapping.keys.iter().find(|key| key.key_code == 9).unwrap();
  // With the 400 ms exit lifetime the previous copy may already be fully
  // gone by the time this later paste appears; the replacement still uses
  // the shared primary slot whenever both visuals overlap.
  if let Some(copy) = swapping.keys.iter().find(|key| key.key_code == 8) {
    assert_eq!(copy.slot, paste.slot);
  }
  assert_eq!(command.layout_from_mask, command.layout_to_mask);
}

fn is_modifier_key(key: u16) -> bool {
  matches!(key, 54 | 55 | 56 | 58 | 59 | 60 | 61 | 62 | 63)
}

#[test]
fn newest_shortcut_replaces_previous() {
  let compositor = KeyboardCompositor::from_shortcuts(vec![
    Shortcut {
      keys: vec![KeyPress {
        key_code: 0,
        modifier_mask: 1,
        down_us: 1_000_000,
        up_us: Some(1_100_000),
      }],
    },
    Shortcut {
      keys: vec![KeyPress {
        key_code: 8,
        modifier_mask: 1,
        down_us: 2_000_000,
        up_us: None,
      }],
    },
  ]);
  let overlay = compositor.evaluate(2_100, Default::default()).unwrap();
  assert!(overlay.keys[..overlay.key_count as usize]
    .iter()
    .any(|key| key.key_code == 8 && key.visible == 1));
}

#[test]
fn held_shortcut_waits_then_animates_after_the_fixed_release_time() {
  let down = serde_json::json!({"type":"keyDown","keyCode":55,"timestampUs":1_000_000,"modifiers":["command"]});
  let held = KeyboardCompositor::from_shortcuts(reconstruct_v2(std::slice::from_ref(&down)));
  assert!(held.evaluate(10_000, Default::default()).is_some());
  let released = KeyboardCompositor::from_shortcuts(reconstruct_v2(&[
    down,
    serde_json::json!({"type":"keyUp","keyCode":55,"timestampUs":1_200_000,"modifiers":[]}),
  ]));
  assert_eq!(
    released.evaluate(1_949, Default::default()).unwrap().keys[0].visible,
    1
  );
  assert_eq!(
    released.evaluate(1_950, Default::default()).unwrap().keys[0].visible,
    2
  );
  assert!(released.evaluate(2_349, Default::default()).is_some());
  assert!(released.evaluate(2_350, Default::default()).is_none());
}

#[test]
fn animations_reach_true_zero_and_none_exits_without_a_tail() {
  let compositor = KeyboardCompositor::from_shortcuts(vec![Shortcut {
    keys: vec![KeyPress {
      key_code: 0,
      modifier_mask: 0,
      down_us: 1_000_000,
      up_us: Some(1_200_000),
    }],
  }]);
  let pop = compositor.evaluate(1_000, Default::default()).unwrap();
  assert_eq!(pop.keys[0].scale, 0.0);
  let fade = compositor
    .evaluate(
      1_000,
      KeyboardEffectSettings {
        animation: KeyboardAnimation::Fade,
        ..Default::default()
      },
    )
    .unwrap();
  assert_eq!(fade.keys[0].alpha, 0.0);
  assert!(compositor
    .evaluate(
      1_950,
      KeyboardEffectSettings {
        animation: KeyboardAnimation::None,
        ..Default::default()
      },
    )
    .is_none());
}
