// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;
use crate::exports::timeline_edit::TimelineRange;

#[test]
fn v1_timeline_expands_legacy_modifier_mask() {
  let compositor = KeyboardCompositor::from_shortcuts_with_legacy(
    vec![Shortcut {
      keys: vec![KeyPress {
        key_code: 0,
        modifier_mask: 1 | 8,
        down_us: 1_000_000,
        up_us: Some(1_100_000),
      }],
    }],
    true,
  );
  assert_eq!(compositor.timeline_items()[0].label, "Command Shift A");
}

#[test]
fn timeline_items_use_reconstruction_order_and_key_labels() {
  let compositor = KeyboardCompositor::from_shortcuts(vec![
    Shortcut {
      keys: vec![
        KeyPress {
          key_code: 55,
          modifier_mask: 1,
          down_us: 1_250_000,
          up_us: Some(1_500_000),
        },
        KeyPress {
          key_code: 0,
          modifier_mask: 1,
          down_us: 1_300_000,
          up_us: Some(1_450_000),
        },
      ],
    },
    Shortcut {
      keys: vec![KeyPress {
        key_code: 53,
        modifier_mask: 0,
        down_us: 2_000_000,
        up_us: None,
      }],
    },
  ]);
  assert_eq!(compositor.timeline_items()[0].label, "Command A");
  assert_eq!(compositor.timeline_items()[0].id, 0);
  assert_eq!(compositor.timeline_items()[1].label, "Esc");
}

#[test]
fn reconstructed_v2_timeline_keeps_chord_bounds() {
  let records = vec![
    serde_json::json!({"type":"keyDown","keyCode":55,"timestampUs":1_000_000,"modifiers":["command"]}),
    serde_json::json!({"type":"keyDown","keyCode":0,"timestampUs":1_100_000,"modifiers":["command"]}),
    serde_json::json!({"type":"keyUp","keyCode":0,"timestampUs":1_200_000,"modifiers":["command"]}),
    serde_json::json!({"type":"keyUp","keyCode":55,"timestampUs":1_300_000,"modifiers":[]}),
  ];
  let compositor = KeyboardCompositor::from_shortcuts(reconstruct_v2(&records));
  assert_eq!(compositor.timeline_items()[0].start_ms, 1_000);
  assert_eq!(compositor.timeline_items()[0].end_ms, 2_450);
}

#[test]
fn timeline_lane_exit_tail_uses_edited_output_time() {
  let records = vec![
    serde_json::json!({"type":"keyDown","keyCode":55,"timestampUs":1_000_000,"modifiers":["command"]}),
    serde_json::json!({"type":"keyDown","keyCode":0,"timestampUs":1_100_000,"modifiers":["command"]}),
    serde_json::json!({"type":"keyUp","keyCode":0,"timestampUs":1_200_000,"modifiers":["command"]}),
    serde_json::json!({"type":"keyUp","keyCode":55,"timestampUs":1_300_000,"modifiers":[]}),
  ];
  let compositor = KeyboardCompositor::from_shortcuts(reconstruct_v2(&records));
  let range = |playback_rate| TimelineRange {
    output_start_us: 0,
    source_end_us: 5_000_000,
    source_start_us: 0,
    playback_rate,
  };

  assert_eq!(
    compositor.timeline_items_with_timeline(Some(&[range(2.0)]))[0].end_ms,
    2_850
  );
  assert_eq!(
    compositor.timeline_items_with_timeline(Some(&[range(0.5)]))[0].end_ms,
    2_250
  );
}

#[test]
fn timeline_lane_exit_tail_crosses_a_speed_boundary_in_output_time() {
  let records = vec![
    serde_json::json!({"type":"keyDown","keyCode":55,"timestampUs":1_000_000,"modifiers":["command"]}),
    serde_json::json!({"type":"keyDown","keyCode":0,"timestampUs":1_100_000,"modifiers":["command"]}),
    serde_json::json!({"type":"keyUp","keyCode":0,"timestampUs":1_200_000,"modifiers":["command"]}),
    serde_json::json!({"type":"keyUp","keyCode":55,"timestampUs":1_300_000,"modifiers":[]}),
  ];
  let compositor = KeyboardCompositor::from_shortcuts(reconstruct_v2(&records));
  let ranges = [
    TimelineRange {
      output_start_us: 0,
      source_end_us: 2_200_000,
      source_start_us: 0,
      playback_rate: 2.0,
    },
    TimelineRange {
      output_start_us: 1_100_000,
      source_end_us: 5_000_000,
      source_start_us: 2_200_000,
      playback_rate: 0.5,
    },
  ];

  assert_eq!(
    compositor.timeline_items_with_timeline(Some(&ranges))[0].end_ms,
    2_363
  );
}

#[test]
fn timeline_lanes_show_the_true_visible_range() {
  let records = vec![
    serde_json::json!({"type":"keyDown","keyCode":0,"timestampUs":1_000_000,"modifiers":[]}),
    serde_json::json!({"type":"keyUp","keyCode":0,"timestampUs":1_100_000,"modifiers":[]}),
    serde_json::json!({"type":"keyDown","keyCode":1,"timestampUs":1_200_000,"modifiers":[]}),
  ];
  let compositor = KeyboardCompositor::from_shortcuts(reconstruct_v2(&records));
  // The replaced badge fades until 1.6s, so its lane overlaps the successor:
  // the timeline shows when artwork is actually on screen.
  assert_eq!(compositor.timeline_items()[0].end_ms, 1_600);
  assert_eq!(compositor.timeline_items()[1].start_ms, 1_200);
}

#[test]
fn timeline_lanes_overlap_across_a_modifier_repress() {
  // A lone Control press whose lingering badge is replaced by a chord that
  // starts before the hold-and-fade lifetime has finished; the lane keeps
  // the fade tail so the true exit time stays visible.
  let records = vec![
    serde_json::json!({"type":"keyDown","keyCode":59,"timestampUs":2_002_000,"modifiers":["control"]}),
    serde_json::json!({"type":"keyUp","keyCode":59,"timestampUs":2_570_000,"modifiers":["control"]}),
    serde_json::json!({"type":"keyDown","keyCode":59,"timestampUs":3_074_000,"modifiers":["control"]}),
    serde_json::json!({"type":"keyDown","keyCode":56,"timestampUs":3_114_000,"modifiers":["shift"]}),
    serde_json::json!({"type":"keyDown","keyCode":0,"timestampUs":4_466_000,"modifiers":["control","shift"]}),
    serde_json::json!({"type":"keyUp","keyCode":0,"timestampUs":4_554_000,"modifiers":["control","shift"]}),
    serde_json::json!({"type":"keyUp","keyCode":59,"timestampUs":4_650_000,"modifiers":["control"]}),
    serde_json::json!({"type":"keyUp","keyCode":56,"timestampUs":4_650_000,"modifiers":["shift"]}),
  ];
  let compositor = KeyboardCompositor::from_shortcuts(reconstruct_v2(&records));
  let items = compositor.timeline_items();
  assert_eq!(items.len(), 2);
  assert_eq!(items[0].label, "Control");
  assert_eq!(items[1].label, "Control Shift A");
  assert!(items[0].end_ms > items[1].start_ms);
  assert_eq!(items[0].end_ms, 3_474);
}

#[test]
fn deleted_ids_preserve_surviving_identity_and_suppress_evaluation() {
  let compositor = KeyboardCompositor::from_shortcuts(vec![
    Shortcut {
      keys: vec![KeyPress {
        key_code: 0,
        modifier_mask: 0,
        down_us: 1_000_000,
        up_us: None,
      }],
    },
    Shortcut {
      keys: vec![KeyPress {
        key_code: 1,
        modifier_mask: 0,
        down_us: 2_000_000,
        up_us: None,
      }],
    },
  ]);
  compositor.set_deleted_shortcuts(&[0], &[]);
  let items = compositor.timeline_items();
  assert_eq!(items.len(), 1);
  assert_eq!(items[0].id, 1);
  assert!(compositor.evaluate(2_100, Default::default()).is_some());
  compositor.set_deleted_shortcuts(&[0, 1], &[]);
  assert!(compositor.evaluate(2_100, Default::default()).is_none());
}

#[test]
fn deleted_ranges_suppress_only_the_selected_timeline_fragment() {
  let compositor = KeyboardCompositor::from_shortcuts(vec![Shortcut {
    keys: vec![KeyPress {
      key_code: 0,
      modifier_mask: 0,
      down_us: 1_000_000,
      up_us: None,
    }],
  }]);
  compositor.set_deleted_shortcuts(
    &[],
    &[DeletedKeyboardShortcutRange {
      end_ms: 1_500,
      shortcut_id: 0,
      start_ms: 0,
    }],
  );
  assert!(compositor.evaluate(1_100, Default::default()).is_none());
  assert!(compositor.evaluate(1_600, Default::default()).is_some());
}

#[test]
fn position_ranges_move_only_the_matching_shortcut_fragment() {
  let compositor = KeyboardCompositor::from_shortcuts(vec![Shortcut {
    keys: vec![KeyPress {
      key_code: 0,
      modifier_mask: 0,
      down_us: 1_000_000,
      up_us: None,
    }],
  }]);
  compositor.set_shortcut_positions(&[KeyboardShortcutPositionRange {
    center_x: 0.2,
    center_y: 0.3,
    end_ms: 2_500,
    shortcut_id: 0,
    start_ms: 1_000,
    size_percent: Some(135.0),
  }]);
  let moved = compositor.evaluate(1_700, Default::default()).unwrap();
  assert_eq!((moved.center_x, moved.center_y), (0.2, 0.3));
  assert_eq!(moved.requested_scale, 1.35);
  assert_eq!(moved.keys[0].scale, 1.35);
  let inherited = compositor
    .evaluate(
      2_600,
      KeyboardEffectSettings {
        position_x_percent: Some(40.0),
        position_y_percent: Some(60.0),
        size_percent: 90.0,
        ..Default::default()
      },
    )
    .unwrap();
  assert_eq!((inherited.center_x, inherited.center_y), (0.4, 0.6));
  assert_eq!(inherited.requested_scale, 0.9);
}

fn control_then_control_shift_a() -> Vec<Shortcut> {
  reconstruct_v2(&[
    serde_json::json!({"type":"keyDown","keyCode":59,"timestampUs":2_002_000,"modifiers":["control"]}),
    serde_json::json!({"type":"keyUp","keyCode":59,"timestampUs":2_570_000,"modifiers":["control"]}),
    serde_json::json!({"type":"keyDown","keyCode":59,"timestampUs":3_074_000,"modifiers":["control"]}),
    serde_json::json!({"type":"keyDown","keyCode":56,"timestampUs":3_114_000,"modifiers":["shift"]}),
    serde_json::json!({"type":"keyDown","keyCode":0,"timestampUs":4_466_000,"modifiers":["control","shift"]}),
    serde_json::json!({"type":"keyUp","keyCode":0,"timestampUs":4_554_000,"modifiers":["control","shift"]}),
    serde_json::json!({"type":"keyUp","keyCode":59,"timestampUs":4_650_000,"modifiers":["control"]}),
    serde_json::json!({"type":"keyUp","keyCode":56,"timestampUs":4_650_000,"modifiers":["shift"]}),
  ])
}

#[test]
fn a_replaced_badge_carries_its_own_centre_through_the_cross_fade() {
  let compositor = KeyboardCompositor::from_shortcuts(control_then_control_shift_a());
  compositor.set_shortcut_positions(&[KeyboardShortcutPositionRange {
    center_x: 0.2,
    center_y: 0.3,
    end_ms: 10_000,
    shortcut_id: 0,
    start_ms: 0,
    size_percent: None,
  }]);
  // The moved Control badge is a separate continuity group, and its exit is
  // FINISHED by the time the fresh badge is pressed: the fade is pulled
  // forward to run 2.674s-3.074s, so the two badges never coexist. While it
  // is the only badge the overlay follows its manual position.
  let fading = compositor.evaluate(2_900, Default::default()).unwrap();
  assert_eq!(fading.key_count, 1);
  assert_eq!(fading.keys[0].key_code, 59);
  assert_eq!(fading.keys[0].visible, 2);
  assert!((fading.center_x - 0.2).abs() < 1e-6);
  assert!((fading.center_y - 0.3).abs() < 1e-6);
  // At the press the stale badge is gone and only the fresh one remains, at
  // the default position.
  let entered = compositor.evaluate(3_200, Default::default()).unwrap();
  assert_eq!(entered.key_count, 2);
  assert!(entered.center_x < 0.0);
  assert!((entered.keys[0].center_x - KEY_CENTER_INHERIT).abs() < 1e-6);
  // The fresh badge's keys never morph out of the replaced badge.
  let visuals = compositor.visuals_snapshot();
  assert!(visuals
    .iter()
    .filter(|visual| visual.source_shortcut == 1)
    .all(|visual| !visual.replacement_enter));
}

#[test]
fn the_overlay_follows_the_moved_successor_while_the_predecessor_keeps_the_default() {
  let compositor = KeyboardCompositor::from_shortcuts(control_then_control_shift_a());
  compositor.set_shortcut_positions(&[KeyboardShortcutPositionRange {
    center_x: 0.7,
    center_y: 0.6,
    end_ms: 10_000,
    shortcut_id: 1,
    start_ms: 0,
    size_percent: Some(150.0),
  }]);
  // The unmoved Control badge finishes its exit at the default position and
  // size before the moved successor is pressed.
  let fading = compositor.evaluate(2_900, Default::default()).unwrap();
  assert_eq!(fading.key_count, 1);
  assert_eq!(fading.keys[0].visible, 2);
  assert!(fading.center_x < 0.0);
  assert!((fading.keys[0].scale_ratio - 1.0).abs() < 1e-6);
  // After the press the overlay follows the successor's manual placement.
  let entered = compositor.evaluate(3_200, Default::default()).unwrap();
  assert_eq!(entered.key_count, 2);
  assert!((entered.center_x - 0.7).abs() < 1e-6);
  assert!((entered.center_y - 0.6).abs() < 1e-6);
  assert!((entered.requested_scale - 1.5).abs() < 1e-6);
  assert!((entered.keys[0].center_x - KEY_CENTER_INHERIT).abs() < 1e-6);
  assert!((entered.keys[0].scale_ratio - 1.0).abs() < 1e-6);
}

#[test]
fn co_located_shortcuts_keep_the_replacement_cross_fade() {
  let compositor = KeyboardCompositor::from_shortcuts(control_then_control_shift_a());
  // Without manual positions the chords continue one badge: every key shares
  // the overlay centre and the outgoing key overlaps its replacement.
  let transition = compositor.evaluate(3_200, Default::default()).unwrap();
  assert_eq!(transition.key_count, 3);
  for index in 0..transition.key_count as usize {
    assert!((transition.keys[index].center_x - KEY_CENTER_INHERIT).abs() < 1e-6);
    assert!((transition.keys[index].center_y - KEY_CENTER_INHERIT).abs() < 1e-6);
  }
  let visuals = compositor.visuals_snapshot();
  let groups = visuals
    .iter()
    .map(|visual| visual.group)
    .collect::<std::collections::HashSet<_>>();
  assert_eq!(groups.len(), 1);
}

#[test]
fn a_deleted_predecessor_never_shapes_the_next_badge() {
  let compositor = KeyboardCompositor::from_shortcuts(control_then_control_shift_a());
  compositor.set_deleted_shortcuts(&[0], &[]);
  // With the lone Control deleted, the chord that follows must be a brand
  // new badge: fresh continuity group and no replacement morphs.
  let visuals = compositor.visuals_snapshot();
  let deleted_group = visuals
    .iter()
    .find(|visual| visual.source_shortcut == 0)
    .expect("the deleted shortcut still builds visuals")
    .group;
  for visual in visuals
    .iter()
    .filter(|visual| visual.source_shortcut == 1)
  {
    assert_ne!(visual.group, deleted_group);
    assert!(!visual.replacement_enter);
  }
}

#[test]
fn many_fresh_badges_stay_within_wire_slot_bounds() {
  // Every shortcut in its own place: each becomes a fresh continuity group
  // with fresh slot ids. The wire's slot indices and layout masks are bit
  // positions, so evaluation must compact them to the frame's own slots.
  let mut records = Vec::new();
  let mut positions = Vec::new();
  for index in 0..40u64 {
    let at = 1_000_000 + index * 200_000;
    records.push(
      serde_json::json!({"type":"keyDown","keyCode":0,"timestampUs":at,"modifiers":[]}),
    );
    records.push(
      serde_json::json!({"type":"keyUp","keyCode":0,"timestampUs":at + 80_000,"modifiers":[]}),
    );
    positions.push(KeyboardShortcutPositionRange {
      center_x: 0.02 * index as f64 + 0.05,
      center_y: 0.5,
      end_ms: 100_000,
      shortcut_id: index,
      start_ms: 0,
      size_percent: None,
    });
  }
  let compositor = KeyboardCompositor::from_shortcuts(reconstruct_v2(&records));
  compositor.set_shortcut_positions(&positions);
  for position_ms in (1_000..10_000).step_by(500) {
    let Some(overlay) = compositor.evaluate(position_ms, Default::default()) else {
      continue;
    };
    for index in 0..overlay.key_count as usize {
      assert!(overlay.keys[index].slot < 32);
    }
  }
}
