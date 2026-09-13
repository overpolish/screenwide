// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;

#[test]
fn entrance_animation_duration_is_invariant_across_segment_speeds() {
  let compositor = v1();
  let half = compositor
    .evaluate_with_timeline(1_150, Default::default(), Some(&speed_plan(0.5)))
    .unwrap();
  let normal = compositor.evaluate(1_300, Default::default()).unwrap();
  let double = compositor
    .evaluate_with_timeline(1_600, Default::default(), Some(&speed_plan(2.0)))
    .unwrap();
  assert!((normal.keys[0].progress - 0.5).abs() < 0.001);
  assert!((half.keys[0].progress - normal.keys[0].progress).abs() < 0.001);
  assert!((double.keys[0].progress - normal.keys[0].progress).abs() < 0.001);
}

#[test]
fn exit_animation_duration_is_invariant_across_segment_speeds() {
  let compositor = KeyboardCompositor::from_shortcuts(vec![Shortcut {
    keys: vec![KeyPress {
      key_code: 0,
      modifier_mask: 0,
      down_us: 1_000_000,
      up_us: Some(1_200_000),
    }],
  }]);
  // The recorded hold ends at 1.95s; each sample is 0.2s later in output time.
  let half = compositor
    .evaluate_with_timeline(2_050, Default::default(), Some(&speed_plan(0.5)))
    .unwrap();
  let normal = compositor.evaluate(2_150, Default::default()).unwrap();
  let double = compositor
    .evaluate_with_timeline(2_350, Default::default(), Some(&speed_plan(2.0)))
    .unwrap();
  assert!((normal.keys[0].progress - 0.5).abs() < 0.001);
  assert!((half.keys[0].progress - normal.keys[0].progress).abs() < 0.001);
  assert!((double.keys[0].progress - normal.keys[0].progress).abs() < 0.001);
}

#[test]
fn layout_animation_duration_is_invariant_across_segment_speeds() {
  let compositor = KeyboardCompositor::from_shortcuts(vec![Shortcut {
    keys: vec![
      KeyPress {
        key_code: 55,
        modifier_mask: 1,
        down_us: 1_000_000,
        up_us: None,
      },
      KeyPress {
        key_code: 0,
        modifier_mask: 1,
        down_us: 2_000_000,
        up_us: None,
      },
    ],
  }]);
  let half = compositor
    .evaluate_with_timeline(2_150, Default::default(), Some(&speed_plan(0.5)))
    .unwrap();
  let normal = compositor.evaluate(2_300, Default::default()).unwrap();
  let double = compositor
    .evaluate_with_timeline(2_600, Default::default(), Some(&speed_plan(2.0)))
    .unwrap();
  let progress = |overlay: &KeyboardOverlay| {
    overlay
      .keys
      .iter()
      .find(|key| key.key_code == 55)
      .unwrap()
      .layout_progress
  };
  assert!((progress(&normal) - 0.5).abs() < 0.001);
  assert!((progress(&half) - progress(&normal)).abs() < 0.001);
  assert!((progress(&double) - progress(&normal)).abs() < 0.001);
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

#[test]
fn an_exit_composes_with_an_unfinished_entrance_without_snapping() {
  // At 2x the 600ms entrance is only ~70% done when the hold ends at 1.85s
  // source. The exit must take over from that point continuously instead of
  // snapping the key to fully-entered before animating out.
  let compositor = KeyboardCompositor::from_shortcuts(vec![Shortcut {
    keys: vec![KeyPress {
      key_code: 0,
      modifier_mask: 0,
      down_us: 1_000_000,
      up_us: Some(1_100_000),
    }],
  }]);
  let plan = speed_plan(2.0);
  let before = compositor
    .evaluate_with_timeline(1_840, Default::default(), Some(&plan))
    .unwrap();
  let after = compositor
    .evaluate_with_timeline(1_900, Default::default(), Some(&plan))
    .unwrap();
  assert!(before.keys[0].progress < 0.75, "the entrance is unfinished");
  assert!(
    (after.keys[0].progress - before.keys[0].progress).abs() < 0.1,
    "the handover is continuous"
  );
  assert!(after.keys[0].progress < 0.8, "no snap to fully entered");
}

#[test]
fn a_rolled_key_reserves_its_slot_before_its_own_entrance() {
  // Shift lands 81ms after Control - one rolled gesture - so it occupies its
  // slot from Control's press at zero size, keeping Control's position
  // stable; A joins 417ms later, outside the roll, and is absent until then.
  let records = vec![
    serde_json::json!({"type":"keyDown","keyCode":59,"modifiers":["control"],"timestampUs":1_787_822u64}),
    serde_json::json!({"type":"keyDown","keyCode":56,"modifiers":["shift"],"timestampUs":1_869_210u64}),
    serde_json::json!({"type":"keyDown","keyCode":0,"modifiers":["control","shift"],"timestampUs":2_285_987u64}),
    serde_json::json!({"type":"keyUp","keyCode":0,"modifiers":["control","shift"],"timestampUs":2_398_132u64}),
    serde_json::json!({"type":"keyUp","keyCode":56,"modifiers":["shift"],"timestampUs":2_545_309u64}),
    serde_json::json!({"type":"keyUp","keyCode":59,"modifiers":["control"],"timestampUs":2_607_073u64}),
  ];
  let compositor = KeyboardCompositor::from_shortcuts(reconstruct_v2(&records));

  let early = compositor.evaluate(1_800, Default::default()).unwrap();
  assert_eq!(early.key_count, 2, "the rolled shift is already present");
  let shift = early.keys[..2]
    .iter()
    .find(|key| key.key_code == 56)
    .expect("the reserved shift is in the overlay");
  assert_eq!(shift.alpha, 0.0);
  assert_eq!(shift.scale, 0.0);
  let control = early.keys[..2]
    .iter()
    .find(|key| key.key_code == 59)
    .expect("control is in the overlay");
  assert!(control.alpha > 0.0);

  let before_late_join = compositor.evaluate(2_200, Default::default()).unwrap();
  assert_eq!(
    before_late_join.key_count, 2,
    "a late join is not reserved ahead of its press"
  );

  let entered = compositor.evaluate(1_900, Default::default()).unwrap();
  let shift = entered.keys[..2]
    .iter()
    .find(|key| key.key_code == 56)
    .expect("shift stays in the overlay");
  assert!(
    shift.alpha > 0.0 || shift.scale > 0.0,
    "shift pops in place"
  );
}
