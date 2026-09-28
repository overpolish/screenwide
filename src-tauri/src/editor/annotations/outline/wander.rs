// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! How a hand-drawn shape's stroke strays from its outline.
//!
//! The stroke wanders off the outline as it goes: a gentle bow all the way
//! round, a lead-in where the pen touches down off the line and eases onto
//! it, and a trail-off where it flicks away as the pen lifts. The bow repeats
//! exactly once a lap, so where the stroke passes its start twice the passes
//! differ only by the lead-in and the trail-off. How the stroke finishes is
//! one of four, by its seed:
//!
//! - it runs on past its start and crosses it, lead-in and trail-off leaning
//!   the same way;
//! - it runs on past its start and opens away from it, the two leaning apart;
//! - it touches down on the line, so the returning pass covers its start
//!   whole, and flicks off either way;
//! - it stops short of its start, leaving a gap between two clean ends.
//!
//! Every tip is either clear of the other pass or wholly under it, never half
//! on it, which is what reads as a lump rather than the end of a pen line.
//!
//! The record carries the bow's reach and the lead-in in `start_head.a`, the
//! bow's two waves' frequency and phase in `start_head.b` and `.c`, the
//! trail-off in `end_head.a`, and how far along the lead-in and the
//! trail-off each ease in `end_head.b`. Everything random is decided here,
//! once, so the shaders only evaluate [`wander`]'s and [`slope`]'s twins.

use std::f32::consts::TAU;

use crate::editor::annotations::geometry::{ArrowGeometry, ArrowTriangle};

/// Sets the stroke `geometry` walks a `perimeter` long outline with from
/// `seed`: where it begins, how long it is, and its wander. The wander grows
/// with the shape, so a thin pen round a large box still reads as drawn by
/// hand, and never shrinks below what the pen needs to show it.
pub(super) fn hand_drawn(
  geometry: &mut ArrowGeometry,
  seed: u32,
  perimeter: f32,
  half: [f32; 2],
  pen: f32,
) {
  let unit = |salt: u32| unit(seed, salt);
  let side = if unit(3) < 0.5 { -1.0 } else { 1.0 };
  let share = |low: f32, span: f32, salt: u32| perimeter * (low + span * unit(salt));
  let shortest = 2.0 * half[0].min(half[1]);
  let overshoot = share(0.04, 0.05, 1).max(pen * 3.0).min(perimeter * 0.2);
  // A hand starts near the top-left. Where the top side is long enough, the
  // start and whatever happens around it sit on it: two passes crossing over
  // a square corner tangle rather than read as a pen.
  let top = 2.0 * (half[0] - geometry.rounding);
  let begin = if top >= 3.0 * overshoot {
    top * (0.1 + 0.25 * unit(0))
  } else {
    perimeter * (0.93 + 0.1 * unit(0)).fract()
  };
  let reach = share(0.003, 0.003, 2).max(pen * 0.25).min(shortest * 0.04);
  let finish = unit(13);
  // (lead-in, its ease, trail-off, its ease, the stroke's length).
  let (lead, lead_ease, trail, trail_ease, stroke) = if finish < 0.8 {
    let trail = share(0.006, 0.006, 6).max(pen * 2.0).min(shortest * 0.2);
    // No bigger than the trail-off, so what is left of it at the end of the
    // overlap never closes on the end's tip.
    let lead = share(0.004, 0.004, 4)
      .max(pen * 1.8)
      .min(shortest * 0.15)
      .min(trail * 0.9);
    // The lead-in has mostly gone by the end of the overlap; the trail-off
    // has barely begun at its start, and not at all under a hidden one.
    let lead_ease = overshoot * (1.4 + 0.3 * unit(11));
    let trail_ease = overshoot * (1.0 + 0.3 * unit(12));
    let stroke = perimeter + overshoot;
    match finish {
      f if f < 0.3 => (side * lead, lead_ease, side * trail, trail_ease, stroke),
      f if f < 0.55 => (side * lead, lead_ease, -side * trail, trail_ease, stroke),
      _ => (0.0, lead_ease, side * trail, overshoot, stroke),
    }
  } else {
    // Short of its start by a few pens, so the two ends stand apart.
    let gap = share(0.02, 0.03, 1).max(pen * 3.0).min(perimeter * 0.1);
    let lead = share(0.002, 0.003, 4).max(pen * 0.8).min(shortest * 0.1);
    let trail = share(0.004, 0.006, 6).max(pen * 1.5).min(shortest * 0.15);
    let flip = if unit(5) < 0.5 { -1.0 } else { 1.0 };
    (
      side * lead,
      share(0.04, 0.03, 11),
      flip * trail,
      share(0.05, 0.05, 12),
      perimeter - gap,
    )
  };
  // Whole waves a lap: two or three long bows, and five to seven ripples.
  let waves = |count: f32| TAU * count / perimeter;
  geometry.c = [begin, stroke];
  geometry.start_head = ArrowTriangle {
    a: [reach, lead],
    b: [waves(2.0 + (unit(7) * 2.0).floor()), TAU * unit(8)],
    c: [waves(5.0 + (unit(9) * 3.0).floor()), TAU * unit(10)],
  };
  geometry.end_head = ArrowTriangle {
    a: [trail, 0.0],
    b: [lead_ease, trail_ease],
    c: [0.0, 0.0],
  };
  geometry.head = 1;
}

/// A number in `[0, 1)` from the seed, different for every `salt`.
fn unit(seed: u32, salt: u32) -> f32 {
  let mut x = seed ^ salt.wrapping_mul(0x9e37_79b9);
  x ^= x >> 16;
  x = x.wrapping_mul(0x7feb_352d);
  x ^= x >> 15;
  x = x.wrapping_mul(0x846c_a68b);
  x ^= x >> 16;
  (x >> 8) as f32 / (1u32 << 24) as f32
}

/// How much of the lead-in is left `from` the stroke's start over `over`,
/// and how fast that changes per unit of `from`: all of it at the start, none
/// past `over`, in an S that leaves and arrives level, so the pen touches
/// down alongside the line and merges into it.
fn settle(from: f32, over: f32) -> [f32; 2] {
  let over = over.max(1e-6);
  let x = (from / over).clamp(0.0, 1.0);
  [1.0 - x * x * (3.0 - 2.0 * x), -6.0 * x * (1.0 - x) / over]
}

/// How much of the trail-off is left `from` the stroke's end over `over`,
/// and how fast that changes per unit of `from`: all of it at the end, none
/// past `over`, steepest at the tip, so the pen flicks off as it lifts.
fn flick(from: f32, over: f32) -> [f32; 2] {
  let over = over.max(1e-6);
  let left = 1.0 - (from / over).clamp(0.0, 1.0);
  [left * left, -2.0 * left / over]
}

/// The lead-in's ease `along` the stroke, and the trail-off's, each as its
/// share left and how fast that share changes along the stroke.
fn eases(geometry: &ArrowGeometry, along: f32) -> ([f32; 2], [f32; 2]) {
  let [lead_ease, trail_ease] = geometry.end_head.b;
  let [trail_left, trail_change] = flick(geometry.c[1] - along, trail_ease);
  (settle(along, lead_ease), [trail_left, -trail_change])
}

/// How far the stroke sits outside the outline `along` into it: nowhere for
/// a clean stroke, whose wander is all zero.
pub(super) fn wander(geometry: &ArrowGeometry, along: f32) -> f32 {
  let [reach, lead] = geometry.start_head.a;
  let [first, first_phase] = geometry.start_head.b;
  let [second, second_phase] = geometry.start_head.c;
  let trail = geometry.end_head.a[0];
  let ([lead_left, _], [trail_left, _]) = eases(geometry, along);
  reach
    * (0.85 * (first * along + first_phase).sin() + 0.15 * (second * along + second_phase).sin())
    + lead * lead_left
    + trail * trail_left
}

/// How fast [`wander`] changes `along` the stroke: what tilts the pen's line
/// against the outline, and so how much thinner a distance measured square to
/// the outline makes it look.
pub(super) fn slope(geometry: &ArrowGeometry, along: f32) -> f32 {
  let [reach, lead] = geometry.start_head.a;
  let [first, first_phase] = geometry.start_head.b;
  let [second, second_phase] = geometry.start_head.c;
  let trail = geometry.end_head.a[0];
  let ([_, lead_change], [_, trail_change]) = eases(geometry, along);
  reach
    * (0.85 * first * (first * along + first_phase).cos()
      + 0.15 * second * (second * along + second_phase).cos())
    + lead * lead_change
    + trail * trail_change
}
