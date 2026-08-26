// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;

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
fn timeline_ends_a_replaced_shortcut_when_its_visual_exits() {
  let records = vec![
    serde_json::json!({"type":"keyDown","keyCode":0,"timestampUs":1_000_000,"modifiers":[]}),
    serde_json::json!({"type":"keyUp","keyCode":0,"timestampUs":1_100_000,"modifiers":[]}),
    serde_json::json!({"type":"keyDown","keyCode":1,"timestampUs":1_200_000,"modifiers":[]}),
  ];
  let compositor = KeyboardCompositor::from_shortcuts(reconstruct_v2(&records));
  assert_eq!(compositor.timeline_items()[0].end_ms, 1_600);
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
