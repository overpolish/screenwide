// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The reveal's timing, its arc-length inversion, and its promise that a mark
//! standing still is drawn exactly the way it was before it could animate.

use super::geometry::*;
use super::*;

/// A curve whose bend is nowhere near its middle, so length and parameter
/// disagree: the control sits almost on top of the start.
const A: [f32; 2] = [0.0, 0.0];
const B: [f32; 2] = [20.0, 0.0];
const C: [f32; 2] = [400.0, 300.0];

fn arc_length_to(t: f32) -> f32 {
  let mut total = 0.0;
  let mut previous = A;
  let steps = 4096;
  for step in 1..=steps {
    let point = curve_point(A, B, C, t * step as f32 / steps as f32);
    total += (point[0] - previous[0]).hypot(point[1] - previous[1]);
    previous = point;
  }
  total
}

/// A mark's stroke, whose heads are four of it long apiece. Ten leaves the
/// single head these tests measure against forty pixels of the path.
const STROKE: f32 = 10.0;

/// A full-sized mark whose window runs from the tail to `fraction`.
fn opened(fraction: f32) -> AnnotationReveal {
  AnnotationReveal {
    low: 0.0,
    high: fraction,
    scale: 1.0,
    opacity: 1.0,
    previous: [0.0, fraction, 1.0, 1.0],
  }
}

/// The prepared reveal of a bare stroke on the test curve, whose window is
/// the whole path's.
fn bare(window: AnnotationReveal) -> AnnotationRevealGeometry {
  reveal_geometry(A, B, C, STROKE, 0.0, window)
}

/// The prepared reveal of a single-headed mark on the test curve.
fn headed(window: AnnotationReveal) -> AnnotationRevealGeometry {
  reveal_geometry(A, B, C, STROKE, 1.0, window)
}

#[test]
fn half_the_arc_is_not_half_the_parameter() {
  let half = bare(opened(0.5));
  assert!(
    (half.high - 0.5).abs() > 0.02,
    "a bent curve's arc midpoint is not its parameter midpoint: {}",
    half.high
  );
  let total = arc_length_to(1.0);
  let measured = arc_length_to(half.high);
  assert!(
    (measured - total * 0.5).abs() < total * 0.002,
    "half the arc landed at {measured} of {total}"
  );
}

#[test]
fn the_window_ends_invert_to_their_own_arc_fractions() {
  let total = arc_length_to(1.0);
  for fraction in [0.1, 0.25, 0.75, 0.9] {
    let measured = arc_length_to(bare(opened(fraction)).high);
    assert!(
      (measured - total * fraction).abs() < total * 0.002,
      "{fraction} of the arc landed at {measured} of {total}"
    );
  }
}

#[test]
fn a_static_mark_collapses_to_the_whole_path_at_full_size() {
  assert_eq!(AnnotationReveal::default(), AnnotationReveal::WHOLE);
  assert_eq!(
    headed(AnnotationReveal::WHOLE),
    AnnotationRevealGeometry {
      low: 0.0,
      high: 1.0,
      start_tip: 0.0,
      end_tip: 1.0,
      scale: 1.0,
    }
  );
  // A clip holds the whole path between its two phases, so the middle of an
  // animated clip prepares exactly as a mark that never animates does.
  assert!(reveal_window(1_500.0, 3_000.0, 16.0).is_whole());
}

/// The window is the shaft's, and the head rides on its end, ahead of it: the
/// head is there from the first frame, moving with the stroke, and its length
/// never counts against the travel. So half the window is half the path less
/// the head, the head's tip a head further on, and the whole window lands the
/// tip on the end point.
#[test]
fn the_head_rides_ahead_of_the_shaft_s_end() {
  let total = arc_length_to(1.0);
  let half = headed(opened(0.5));
  assert!(
    (arc_length_to(half.high) - (total - 40.0) * 0.5).abs() < total * 0.002,
    "the shaft ends at {} of {total}",
    arc_length_to(half.high)
  );
  assert!(
    (arc_length_to(half.end_tip) - arc_length_to(half.high) - 40.0).abs() < total * 0.002,
    "the head's tip is {} past the shaft",
    arc_length_to(half.end_tip) - arc_length_to(half.high)
  );
  // Fully drawn but still arriving, so the reveal still owns the geometry.
  let drawn = headed(AnnotationReveal {
    opacity: 0.5,
    ..opened(1.0)
  });
  assert_eq!(drawn.end_tip, 1.0);
  assert!(
    (arc_length_to(drawn.high) - (total - 40.0)).abs() < total * 0.002,
    "{}",
    arc_length_to(drawn.high)
  );
}

/// A bare tail keeps its round end inside the mark until it has a few
/// strokes of shaft behind the head, and has it all back by four of them:
/// the stroke stops half a width inside the window at first, so the round
/// end's edge is on the tail rather than a knob past it. Both ways.
#[test]
fn a_short_bare_tail_keeps_its_round_end_inside_the_mark() {
  let total = arc_length_to(1.0);
  let setting_off = |shaft: f32| {
    arc_length_to(
      headed(AnnotationReveal {
        low: 0.0,
        high: shaft / (total - 40.0),
        scale: 1.0,
        opacity: 1.0,
        previous: [0.0, 1.0, 1.0, 1.0],
      })
      .low,
    )
  };
  assert!(
    (setting_off(0.0) - STROKE * 0.5).abs() < 0.1,
    "{}",
    setting_off(0.0)
  );
  assert!(
    (setting_off(10.0) - STROKE * 0.375).abs() < 0.1,
    "{}",
    setting_off(10.0)
  );
  assert!(setting_off(40.0).abs() < 0.1, "{}", setting_off(40.0));
  assert!(setting_off(200.0).abs() < 0.1, "{}", setting_off(200.0));
  let leaving = |shaft: f32| {
    arc_length_to(
      headed(AnnotationReveal {
        low: 1.0 - shaft / (total - 40.0),
        high: 1.0,
        scale: 1.0,
        opacity: 1.0,
        previous: [0.0, 1.0, 1.0, 1.0],
      })
      .low,
    ) - (total - 40.0 - shaft)
  };
  assert!(
    (leaving(0.0) - STROKE * 0.5).abs() < 0.1,
    "{}",
    leaving(0.0)
  );
  assert!(leaving(40.0).abs() < 0.1, "{}", leaving(40.0));
  // A tail wearing its own head has no bare end to hold in.
  let both = reveal_geometry(A, B, C, STROKE, 2.0, opened(0.1));
  assert!(
    (arc_length_to(both.low) - 40.0).abs() < 0.1,
    "{}",
    arc_length_to(both.low)
  );
}

/// A head grows out of its base on the shaft's end, and back into it: at half
/// size its tip is half a head past the shaft, and the shaft's end has not
/// moved for it.
#[test]
fn a_head_grows_from_its_base_on_the_shaft() {
  let total = arc_length_to(1.0);
  let half_size = headed(AnnotationReveal {
    scale: 0.5,
    ..opened(0.5)
  });
  let full_size = headed(opened(0.5));
  assert_eq!(half_size.high, full_size.high);
  assert_eq!(half_size.scale, 0.5);
  assert!(
    (arc_length_to(half_size.end_tip) - arc_length_to(half_size.high) - 20.0).abs() < total * 0.002,
    "{}",
    arc_length_to(half_size.end_tip) - arc_length_to(half_size.high)
  );
  // Both heads, and the tail's rides the shaft's start the same way.
  let both = reveal_geometry(
    A,
    B,
    C,
    STROKE,
    2.0,
    AnnotationReveal {
      low: 0.5,
      high: 1.0,
      scale: 1.0,
      opacity: 1.0,
      previous: [0.0, 1.0, 1.0, 1.0],
    },
  );
  assert!(
    (arc_length_to(both.low) - arc_length_to(both.start_tip) - 40.0).abs() < total * 0.002,
    "{}",
    arc_length_to(both.low) - arc_length_to(both.start_tip)
  );
  assert_eq!(both.end_tip, 1.0);
}

#[test]
fn the_mark_draws_in_over_a_second_and_out_over_three_quarters_of_one() {
  let at = |elapsed| reveal_window(elapsed, 3_000.0, 0.0);
  assert_eq!(at(0.0).high, 0.0);
  assert_eq!(at(1_000.0).high, 1.0);
  assert_eq!(at(1_000.0).low, 0.0);
  assert_eq!(at(2_250.0).low, 0.0);
  assert!(at(2_625.0).low > 0.0 && at(2_625.0).low < 1.0);
  assert_eq!(at(3_000.0).low, 1.0);
  assert_eq!(at(3_000.0).high, 1.0);
}

/// The mark arrives whole, fading in over the first three tenths of the opening
/// phase while it is already drawing, and its head shrinks away over the
/// last third of the closing one, starting while the stroke is still drawing
/// itself back in: it leaves with the stroke, not after it.
#[test]
fn the_mark_fades_in_whole_and_shrinks_away_late() {
  let at = |elapsed| reveal_window(elapsed, 3_000.0, 0.0);
  assert_eq!(at(0.0).opacity, 0.0);
  assert_eq!(at(0.0).scale, 1.0);
  assert!(
    (at(150.0).opacity - 0.5).abs() < 1e-6,
    "{}",
    at(150.0).opacity
  );
  assert_eq!(at(300.0).opacity, 1.0);
  // And moving while it fades - a stroke or more of travel by the time it is
  // half solid, a few by the time it is whole, and not a stem's worth.
  assert!(at(150.0).high > 0.04, "{}", at(150.0).high);
  assert!(
    at(300.0).high > 0.1 && at(300.0).high < 0.3,
    "{}",
    at(300.0).high
  );
  assert_eq!(at(2_737.5).opacity, 1.0);
  // The head starts to go with the stroke still coming in, and coming in at
  // speed: the tail is home at nine tenths of the phase, with the head half
  // gone, rather than creeping over its last pixels for the whole shrink.
  assert_eq!(at(2_737.5).scale, 1.0);
  assert!(
    at(2_737.5).low > 0.8 && at(2_737.5).low < 0.95,
    "{}",
    at(2_737.5).low
  );
  assert!(
    (at(2_868.75).scale - 0.5).abs() < 1e-6,
    "{}",
    at(2_868.75).scale
  );
  assert!(
    at(2_868.75).low > 0.99 && at(2_868.75).low < 1.0,
    "{}",
    at(2_868.75).low
  );
  assert_eq!(at(2_925.0).low, 1.0);
  assert!(
    at(2_925.0).scale > 0.0 && at(2_925.0).scale < 0.5,
    "{}",
    at(2_925.0).scale
  );
  assert_eq!(at(3_000.0).scale, 0.0);
}

#[test]
fn a_short_clip_still_finishes_both_phases() {
  // Each phase is capped at a third of the clip, so a 900ms clip draws itself
  // in over 300ms, holds for 300ms and leaves over the last 300ms.
  let at = |elapsed| reveal_window(elapsed, 900.0, 0.0);
  assert_eq!(at(300.0).high, 1.0);
  assert_eq!(at(300.0).low, 0.0);
  assert_eq!(at(600.0).low, 0.0);
  assert!(at(450.0).is_whole());
  assert_eq!(at(900.0).low, 1.0);
  // The ramps shorten with the phase they take their share of.
  assert_eq!(at(90.0).opacity, 1.0);
  assert_eq!(at(795.0).scale, 1.0);
}

/// 4p^3 below half, 1 - (2 - 2p)^3 / 2 above it: a quarter of the way
/// through a phase is 4 * 0.25^3 = 1/16 of the travel, the midpoint is half
/// of it, and three quarters through is 1 - 0.5^3 / 2 = 15/16. An ease-out
/// would have been past half the travel by the first of those. The closing
/// travel runs the curve whole; the opening sets off a little way in, so it
/// is moving from its first frame and still lands the same way.
#[test]
fn each_phase_eases_in_and_out() {
  let at = |elapsed| reveal_window(elapsed, 3_000.0, 0.0);
  for (fraction, eased) in [(0.25, 0.0625), (0.5, 0.5), (0.75, 0.9375)] {
    // The tail is home at nine tenths of the closing phase.
    let closing = at(2_250.0 + 675.0 * fraction).low;
    assert!(
      (closing - eased).abs() < 1e-6,
      "closing at {fraction}: {closing}"
    );
  }
  assert_eq!(at(0.0).high, 0.0);
  assert_eq!(at(1_000.0).high, 1.0);
  // Moving from the start: a tenth of the phase in, past a hundredth of the
  // way, where the whole curve would be at a two-hundred-and-fiftieth.
  assert!(at(100.0).high > 0.01, "{}", at(100.0).high);
  // And still an ease: the second half of the travel is the slower one.
  assert!(at(500.0).high > 0.6, "{}", at(500.0).high);
  assert!(
    at(1_000.0).high - at(900.0).high < at(100.0).high,
    "the landing is softer than the takeoff"
  );
}

/// The shutter-start window trails whichever end is moving, and a paused
/// frame exposes no interval at all.
#[test]
fn the_previous_window_trails_whichever_end_is_moving() {
  let opening = reveal_window(500.0, 3_000.0, 16.0);
  assert!(
    opening.previous[1] < opening.high,
    "the end leads while the clip opens"
  );
  assert_eq!(opening.previous[0], opening.low, "the tail is at rest");
  let closing = reveal_window(2_625.0, 3_000.0, 16.0);
  assert!(
    closing.previous[0] < closing.low,
    "the start leads while the clip closes"
  );
  let still = reveal_window(500.0, 3_000.0, 0.0);
  assert_eq!(
    [still.previous[0], still.previous[1]],
    [still.low, still.high]
  );
}

/// The editor reaches a clip back by the draw-in phase when it places a fresh
/// mark, and TypeScript cannot read this constant, so it keeps its own copy.
/// This is what stops the two from drifting.
#[test]
fn the_editor_places_a_clip_by_this_phase() {
  const SOURCE: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../src/features/editor/annotations.ts"
  ));
  let declared = SOURCE
    .lines()
    .find_map(|line| {
      line
        .trim()
        .strip_prefix("export const ANNOTATION_DRAW_IN_MS = ")
    })
    .and_then(|value| value.trim_end_matches(';').parse::<f32>().ok());
  assert_eq!(declared, Some(REVEAL_DRAW_IN_MS));
}
