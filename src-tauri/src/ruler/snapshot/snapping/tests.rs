// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;

#[test]
fn captured_plus_outside_drag_does_not_widen_the_detected_keycap() {
  // Exact candidates and drag from the native reproduction trace.
  let keycap = Rect::from_xywh(646.5, 509.0, 20.0, 20.0);
  let boxes = [
    Rect::from_xywh(653.0, 514.5, 7.0, 9.5),
    Rect::from_xywh(632.0, 514.5, 9.0, 9.0),
    Rect::from_xywh(606.5, 509.0, 20.0, 20.0),
    keycap,
  ];
  let drag = Rect::from_xywh(
    642.089763542723,
    503.9941483145801,
    674.2788807886004 - 642.089763542723,
    531.7989541478114 - 503.9941483145801,
  );
  assert_eq!(snap_bounds(&boxes, drag), keycap);
}

#[test]
fn outside_components_and_edge_touching_components_are_excluded_on_every_side() {
  let content = Rect::from_xywh(35.0, 35.0, 20.0, 20.0);
  let drag = Rect::from_xywh(30.0, 30.0, 30.0, 30.0);
  for gap in [0.0, 2.0] {
    let boxes = [
      content,
      Rect::from_xywh(25.0 - gap, 40.0, 5.0, 5.0),
      Rect::from_xywh(60.0 + gap, 40.0, 5.0, 5.0),
      Rect::from_xywh(40.0, 25.0 - gap, 5.0, 5.0),
      Rect::from_xywh(40.0, 60.0 + gap, 5.0, 5.0),
    ];
    assert_eq!(snap_bounds(&boxes, drag), content);
  }
}

#[test]
fn slack_still_recovers_a_partially_clipped_component() {
  let boxes = [Rect::from_xywh(28.0, 35.0, 25.0, 20.0)];
  let drag = Rect::from_xywh(30.0, 30.0, 30.0, 30.0);
  assert_eq!(
    snap_bounds(&boxes, drag),
    Rect::from_xywh(30.0, 35.0, 23.0, 20.0)
  );
}
