// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! An image's sway: the slight, slow turn and drift a recording gives a
//! picture so it feels alive without pointing at anything.
//!
//! It is worked out for each frame from the seed the document keeps and how
//! far into its clip the frame is, and moves only the drawn picture. The
//! image's own middle and turn, which its frame, grips and snapping read,
//! stay where it rests. Every term is a sine, eased out of rest over the
//! start of the clip by a curve whose first two derivatives also start and
//! end at zero, so the picture's place, speed and acceleration all change
//! smoothly and a random phase never makes it jump.

use crate::editor::annotations::AnnotationPoint;

/// The most the picture turns either way: a fifth of a degree, so the turn
/// reads as life rather than as a wobble.
pub(crate) const MAX_TURN: f64 = 0.2 * std::f64::consts::PI / 180.0;

/// The furthest its middle drifts, as a share of its longer side, so a large
/// picture moves as little to the eye as a small one.
pub(crate) const MAX_DRIFT: f64 = 0.004;

/// How long it takes to ease out of rest at the start of its clip.
const SETTLE_MS: f64 = 600.0;

/// How strong each wave's lighter partner is beside its main swing.
const PARTNER: f64 = 0.25;

/// Where the picture is drawn from where it rests: turned `turn` radians
/// further clockwise and moved by `shift`, in source pixels.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct Sway {
  pub(crate) turn: f64,
  pub(crate) shift: AnnotationPoint,
}

impl Sway {
  const REST: Self = Self {
    turn: 0.0,
    shift: AnnotationPoint { x: 0.0, y: 0.0 },
  };
}

/// The sway of an image `size` source pixels along its longer side,
/// `clock_ms` into its clip: at rest where it does not sway or is drawn
/// outside a clip.
pub(crate) fn sway_at(seed: Option<u32>, clock_ms: Option<f64>, size: f64) -> Sway {
  let (Some(seed), Some(clock_ms)) = (seed, clock_ms) else {
    return Sway::REST;
  };
  if !clock_ms.is_finite() || clock_ms <= 0.0 || !size.is_finite() || size <= 0.0 {
    return Sway::REST;
  }
  let seconds = clock_ms / 1_000.0;
  let ease = settle(clock_ms / SETTLE_MS);
  // Each axis may drift this far, so together they stay inside the reach.
  let drift = MAX_DRIFT * size / std::f64::consts::SQRT_2;
  Sway {
    turn: ease * wave(seed, 0, (2.6, 3.8), (0.6, 1.2), seconds) * MAX_TURN / 1.5,
    shift: AnnotationPoint {
      x: ease * wave(seed, 1, (4.5, 7.0), (0.7, 1.0), seconds) * drift / (1.0 + PARTNER),
      y: ease * wave(seed, 2, (4.5, 7.0), (0.7, 1.0), seconds) * drift / (1.0 + PARTNER),
    },
  }
}

/// One wave of the sway, `seconds` in: a main swing with a period drawn
/// from `periods` and a strength drawn from `strengths`, and a lighter
/// partner at a frequency that is no whole multiple of it, so the motion
/// never settles into a beat. Never more than `strengths.1 * (1 + PARTNER)`
/// either way.
fn wave(seed: u32, index: u32, periods: (f64, f64), strengths: (f64, f64), seconds: f64) -> f64 {
  let draw = |slot: u32| unit(seed, index * 8 + slot);
  let between = |slot: u32, (low, high): (f64, f64)| low + (high - low) * draw(slot);
  let tau = std::f64::consts::TAU;
  let frequency = 1.0 / between(0, periods);
  let partner = frequency * between(1, (1.55, 1.85));
  let strength = between(2, strengths);
  let phase = tau * draw(3);
  let partner_phase = tau * draw(4);
  strength
    * ((tau * frequency * seconds + phase).sin()
      + PARTNER * (tau * partner * seconds + partner_phase).sin())
}

/// A share in [0, 1) drawn from `seed` for the draw numbered `slot`.
fn unit(seed: u32, slot: u32) -> f64 {
  // The `lowbias32` integer hash, over the seed and the slot.
  let mut value = seed ^ slot.wrapping_add(1).wrapping_mul(0x9E37_79B9);
  value ^= value >> 16;
  value = value.wrapping_mul(0x7FEB_352D);
  value ^= value >> 15;
  value = value.wrapping_mul(0x846C_A68B);
  value ^= value >> 16;
  f64::from(value) / 4_294_967_296.0
}

/// Quintic smootherstep: 0 below 0, 1 above 1, with no jump in its slope or
/// its curvature at either end.
fn settle(progress: f64) -> f64 {
  let t = progress.clamp(0.0, 1.0);
  t * t * t * (t * (t * 6.0 - 15.0) + 10.0)
}

#[cfg(test)]
mod tests {
  use super::*;

  const SIZE: f64 = 400.0;

  fn reach(sway: Sway) -> f64 {
    sway.shift.x.hypot(sway.shift.y)
  }

  #[test]
  fn it_never_turns_or_drifts_past_its_reach() {
    for seed in [0, 1, 7, 0xDEAD_BEEF, u32::MAX] {
      for step in 0..20_000 {
        let sway = sway_at(Some(seed), Some(f64::from(step) * 7.0), SIZE);
        assert!(
          sway.turn.abs() <= MAX_TURN + 1e-12,
          "{seed} {step}: {}",
          sway.turn
        );
        assert!(
          reach(sway) <= MAX_DRIFT * SIZE + 1e-9,
          "{seed} {step}: {sway:?}"
        );
      }
    }
  }

  #[test]
  fn it_rests_at_the_start_of_its_clip_and_outside_one() {
    assert_eq!(sway_at(Some(3), Some(0.0), SIZE), Sway::REST);
    assert_eq!(sway_at(Some(3), None, SIZE), Sway::REST);
    assert_eq!(sway_at(None, Some(2_000.0), SIZE), Sway::REST);
  }

  #[test]
  fn it_moves_without_jumps() {
    // A frame at 240 fps apart never moves the picture more than a sliver
    // of its reach, easing in included, from the first frame on.
    for seed in [0, 42, u32::MAX] {
      let mut last = sway_at(Some(seed), Some(0.0), SIZE);
      for step in 1..4_000 {
        let next = sway_at(Some(seed), Some(f64::from(step) * 1_000.0 / 240.0), SIZE);
        assert!(
          (next.turn - last.turn).abs() < MAX_TURN * 0.04,
          "{seed} {step}"
        );
        let moved = (next.shift.x - last.shift.x).hypot(next.shift.y - last.shift.y);
        assert!(moved < MAX_DRIFT * SIZE * 0.04, "{seed} {step}");
        last = next;
      }
    }
  }

  #[test]
  fn it_keeps_moving_once_settled() {
    let at = |ms: f64| sway_at(Some(9), Some(ms), SIZE);
    let reached = (0..400)
      .map(|step| at(1_000.0 + f64::from(step) * 25.0))
      .fold((0.0_f64, 0.0_f64), |(turn, drift), sway| {
        (turn.max(sway.turn.abs()), drift.max(reach(sway)))
      });
    assert!(reached.0 > MAX_TURN * 0.25, "{reached:?}");
    assert!(reached.1 > MAX_DRIFT * SIZE * 0.25, "{reached:?}");
  }

  #[test]
  fn different_seeds_sway_differently() {
    let a = sway_at(Some(1), Some(2_500.0), SIZE);
    let b = sway_at(Some(2), Some(2_500.0), SIZE);
    assert_ne!(a, b);
    assert_eq!(a, sway_at(Some(1), Some(2_500.0), SIZE));
  }
}
