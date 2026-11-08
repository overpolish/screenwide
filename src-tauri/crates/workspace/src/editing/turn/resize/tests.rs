// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;

fn rect(x: f64, y: f64, width: f64, height: f64) -> PreviewSurfaceRect {
  PreviewSurfaceRect {
    x,
    y,
    width,
    height,
  }
}

/// A screen turned well round, taking most of a pane.
fn start() -> TurnedStart {
  TurnedStart {
    rotation: SceneRotation {
      x: -25.8,
      y: 43.0,
      z: 10.0,
    },
    frame: rect(219.0, 140.0, 797.0, 593.0),
    pane: rect(91.0, 19.0, 960.0, 714.0),
  }
}

/// Where the layer turned as `start` has it draws its own point `(x, y)`,
/// as shares of its half extents from its middle, its box at `scale` with
/// its top-left corner at `corner` in shares of the pane.
fn drawn(start: TurnedStart, scale: f64, corner: (f64, f64), (x, y): (f64, f64)) -> (f64, f64) {
  let TurnedStart {
    rotation,
    frame,
    pane,
  } = start;
  let (width, height) = (frame.width * scale, frame.height * scale);
  let middle = (
    pane.x + corner.0 * pane.width + width / 2.0,
    pane.y + corner.1 * pane.height + height / 2.0,
  );
  let turn = turned_homography(rotation, middle, pane.width.max(pane.height));
  apply(
    turn,
    (middle.0 + x * width / 2.0, middle.1 + y * height / 2.0),
  )
  .unwrap()
}

fn close(a: (f64, f64), b: (f64, f64)) -> bool {
  (a.0 - b.0).hypot(a.1 - b.1) < 1e-6
}

/// The start's own top-left corner, as shares of the pane.
fn corner(start: TurnedStart) -> (f64, f64) {
  (
    (start.frame.x - start.pane.x) / start.pane.width,
    (start.frame.y - start.pane.y) / start.pane.height,
  )
}

#[test]
fn a_turned_corner_drag_holds_the_opposite_corner_where_it_is_drawn() {
  let start = start();
  let held = drawn(start, 1.0, corner(start), (1.0, 1.0));
  for hand in [(150.0, 60.0), (260.0, 220.0), (40.0, -30.0)] {
    let (scale, top_left) = turned_resize(start, 1 | 4, false, hand, (0.1, 8.0)).unwrap();
    assert!(
      close(drawn(start, scale, top_left, (1.0, 1.0)), held),
      "{hand:?}"
    );
  }
}

/// The top-left corner, as shares of the pane, that puts the layer's own
/// point `held` where the start draws it once the layer is at `scale`.
fn holding(start: TurnedStart, scale: f64, held: (f64, f64)) -> (f64, f64) {
  let still = drawn(start, 1.0, corner(start), held);
  let mut top_left = corner(start);
  for _ in 0..200 {
    let now = drawn(start, scale, top_left, held);
    top_left.0 -= (now.0 - still.0) / start.pane.width * 0.5;
    top_left.1 -= (now.1 - still.1) / start.pane.height * 0.5;
  }
  top_left
}

#[test]
fn a_turned_grip_lands_under_a_hand_it_can_reach() {
  let start = start();
  // The hand placed where a grip is drawn once the layer has grown by half,
  // its opposite side held, takes the grip exactly there.
  for (edges, grip, held) in [
    (1 | 4, (-1.0, -1.0), (1.0, 1.0)),
    (4, (0.0, -1.0), (0.0, 1.0)),
    (2, (1.0, 0.0), (-1.0, 0.0)),
  ] {
    let hand = drawn(start, 1.5, holding(start, 1.5, held), grip);
    let (scale, top_left) = turned_resize(start, edges, false, hand, (0.1, 8.0)).unwrap();
    assert!((scale - 1.5).abs() < 1e-6, "{edges}: {scale}");
    assert!(close(drawn(start, scale, top_left, grip), hand), "{edges}");
  }
}

#[test]
fn a_centred_turned_drag_holds_the_middle_where_it_is_drawn() {
  let start = start();
  let middle = drawn(start, 1.0, corner(start), (0.0, 0.0));
  let (scale, top_left) = turned_resize(start, 1 | 4, true, (120.0, 90.0), (0.1, 8.0)).unwrap();
  assert!(close(drawn(start, scale, top_left, (0.0, 0.0)), middle));
}

#[test]
fn a_turned_drag_stops_at_the_smallest_and_largest_scale() {
  let start = start();
  // Past the held corner, and far out beyond the pane.
  let held = drawn(start, 1.0, corner(start), (1.0, 1.0));
  let (shrunk, _) = turned_resize(
    start,
    1 | 4,
    false,
    (held.0 + 50.0, held.1 + 50.0),
    (0.4, 2.0),
  )
  .unwrap();
  let (grown, _) = turned_resize(start, 1 | 4, false, (-5_000.0, -5_000.0), (0.4, 2.0)).unwrap();
  assert!((shrunk - 0.4).abs() < 1e-6, "{shrunk}");
  assert!((grown - 2.0).abs() < 1e-6, "{grown}");
}
