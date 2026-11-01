// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::{pick_through, shown_at, Lens, LensPick, LensView};

/// A square lens 200 across at (100, 100), enlarging twice.
fn square() -> Lens {
  Lens::from_box([0.0, 0.0], [200.0, 200.0], 0.0, 2.0)
}

#[test]
fn a_press_inside_a_lens_lands_nearer_its_middle_by_its_zoom() {
  let view = square().view();
  assert_eq!(view.beneath([100.0, 100.0]), [100.0, 100.0]);
  assert_eq!(view.beneath([180.0, 60.0]), [140.0, 80.0]);
  // Seeing is the inverse of pressing.
  assert_eq!(view.seen([140.0, 80.0]), [180.0, 60.0]);
}

#[test]
fn lenses_stacked_read_a_press_through_each_in_turn() {
  let low = square();
  let high = Lens::from_box([100.0, 100.0], [300.0, 300.0], 0.0, 4.0);
  // A press at (220, 220) reads through the high lens to (205, 205), then
  // through the low one to (152.5, 152.5).
  let view = LensView::IDENTITY.through(&high).through(&low);
  let beneath = view.beneath([220.0, 220.0]);
  assert!((beneath[0] - 152.5).abs() < 1e-9, "{beneath:?}");
  assert_eq!(view.seen(beneath), [220.0, 220.0]);
}

#[test]
fn a_round_lens_covers_its_circle_and_not_its_box_corners() {
  let round = Lens::from_box([0.0, 0.0], [200.0, 200.0], 50.0, 2.0);
  assert!(round.covers([100.0, 195.0]));
  assert!(!round.covers([195.0, 195.0]));
}

#[test]
fn what_lies_under_a_lens_shows_magnified_or_is_hidden_under_its_rim() {
  let lens = square();
  // Outside the lens, a point shows where it is.
  let (at, view) = shown_at([300.0, 300.0], [lens]).unwrap();
  assert_eq!(at, [300.0, 300.0]);
  assert!(view.is_identity());
  // Near the middle, it shows twice as far out.
  let (at, view) = shown_at([120.0, 90.0], [lens]).unwrap();
  assert_eq!(at, [140.0, 80.0]);
  assert_eq!(view.beneath(at), [120.0, 90.0]);
  // Under the rim, past half way out, the lens shows something else there.
  assert_eq!(shown_at([190.0, 100.0], [lens]), None);
}

/// Three annotations, bottom to top: an arrow under a lens, the lens, and
/// a counter over the lens. Each is hit within 5 of its own point.
fn picked(point: [f64; 2]) -> Option<LensPick> {
  let marks = [[140.0, 100.0], [f64::NAN; 2], [20.0, 20.0]];
  pick_through(
    3,
    |_| point,
    |index| (index == 1).then(square),
    |_, _| true,
    |index, at, insides| {
      !insides && (at[0] - marks[index][0]).hypot(at[1] - marks[index][1]) <= 5.0
    },
  )
}

#[test]
fn a_press_on_an_annotation_seen_through_a_lens_takes_it() {
  // The arrow at (140, 100) shows at (180, 100).
  let pick = picked([180.0, 100.0]).unwrap();
  assert_eq!(pick.index, 0);
  assert_eq!(pick.view.beneath([180.0, 100.0]), [140.0, 100.0]);
}

#[test]
fn a_press_on_its_true_place_under_the_lens_takes_the_lens() {
  // The arrow's own place is covered by the lens, which shows (120, 100)
  // there: nothing under the lens is hit, so the lens is.
  let pick = picked([140.0, 100.0]).unwrap();
  assert_eq!(pick.index, 1);
  assert!(pick.view.is_identity());
}

#[test]
fn an_annotation_over_the_lens_wins_over_what_it_shows() {
  let pick = picked([20.0, 20.0]).unwrap();
  assert_eq!(pick.index, 2);
  assert!(pick.view.is_identity());
}

#[test]
fn a_press_off_every_lens_and_mark_takes_nothing() {
  assert_eq!(picked([400.0, 400.0]), None);
}

#[test]
fn a_lens_shows_only_what_lies_under_it() {
  // The arrow is on another picture than the lens: the lens hides it.
  let pick = pick_through(
    2,
    |_| [180.0, 100.0],
    |index| (index == 1).then(square),
    |_, _| false,
    |index, at, _| index == 0 && (at[0] - 140.0).hypot(at[1] - 100.0) <= 5.0,
  )
  .unwrap();
  assert_eq!(pick.index, 1);
}

#[test]
fn an_inside_seen_through_a_lens_leaves_the_press_to_the_lens() {
  // Under the lens, a shape's inside covers everything, and an arrow is drawn
  // at (140, 100) over its middle: the arrow is taken where it shows, and
  // anywhere else in the lens the lens is, so it can still be moved.
  let pick = |point| {
    pick_through(
      3,
      |_| point,
      |index| (index == 2).then(square),
      |_, _| true,
      |index, at: [f64; 2], insides| match index {
        0 => insides,
        1 => !insides && (at[0] - 140.0).hypot(at[1] - 100.0) <= 5.0,
        _ => false,
      },
    )
    .unwrap()
    .index
  };
  assert_eq!(pick([180.0, 100.0]), 1);
  assert_eq!(pick([60.0, 60.0]), 2);
  // Off the lens, the inside is taken as ever.
  assert_eq!(pick([400.0, 400.0]), 0);
}
