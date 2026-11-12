// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The beat-matched blend. The earlier song comes to the later's tempo over
//! a few of its beats, pitch kept, so the two songs' beats lock; the later
//! starts up to half a beat earlier or later in its file, so a beat of it
//! falls on the swap. Over the beats around the swap they hand over band by
//! band: the later's highs come in first, the mids cross in the middle, the
//! bass swaps over a beat on the swap, and the earlier's highs go last. The
//! later song plays as it is throughout.

use super::{Fade, Laid, Piece, SWAP_MS};
use crate::pieces::Tone;
use crate::store::music::Beat;

/// The earlier song's beats its ramp lasts and the beats each side of the
/// swap, tried longest first, until the blend fits the clips and files.
const SIZES: [(u32, u32); 4] = [(8, 4), (4, 4), (4, 2), (2, 2)];
/// Tempos closer than this, as a share, need no ramp.
const SAME_TEMPO: f64 = 0.002;

#[expect(
  clippy::cast_possible_truncation,
  clippy::cast_sign_loss,
  reason = "a time in ms, checked not negative"
)]
fn ms(time: f64) -> u64 {
  time.max(0.0).round() as u64
}

#[expect(clippy::cast_precision_loss, reason = "times in ms fit an f64")]
fn float(ms: u64) -> f64 {
  ms as f64
}

/// Where in its file `piece`, played as it is, is at `at_ms` of the
/// timeline, which may be outside the piece.
fn file_at(piece: &Piece, at_ms: f64) -> f64 {
  float(piece.file_start_ms) + at_ms - float(piece.start_ms)
}

/// When, on the timeline, each part of a blend happens.
struct Plan {
  /// Where the earlier song's ramp starts, on one of its beats, and where
  /// in its file that beat is.
  ramp_at: f64,
  ramp_file: f64,
  /// The ramp's speeds, each held for one of the earlier song's beats,
  /// rising towards `speed`; none where the tempos already match.
  ramp: Vec<f64>,
  /// The earlier song's beat as long.
  our_beat: f64,
  /// Where the blend starts and the bass swaps, both on beats.
  blend_at: f64,
  swap: f64,
  /// Where the earlier song's file is at `blend_at`, and its speed from
  /// then on.
  blend_file: f64,
  speed: f64,
  /// The later song's beat as long, and where its file starts at the join
  /// for a beat of it to fall on the swap.
  beat: f64,
  later_file_start: f64,
}

impl Plan {
  fn end(&self) -> f64 {
    2.0 * self.swap - self.blend_at
  }
}

/// The blend of `ramp_beats` and `side_beats` around the join, if it fits.
fn plan(
  earlier: &Laid<'_>,
  later: &Laid<'_>,
  join: f64,
  (ours, theirs): (Beat, Beat),
  (ramp_beats, side_beats): (u32, u32),
) -> Option<Plan> {
  let (last, first) = (earlier.pieces.last()?, later.pieces.first()?);
  let half_swap = float(SWAP_MS / 2);
  let tempo_ratio = ours.period_ms / theirs.period_ms;
  let needs_ramp = (tempo_ratio - 1.0).abs() > SAME_TEMPO;
  let speed = if needs_ramp { tempo_ratio } else { 1.0 };
  // `atempo` cannot change speed as it plays without losing sound, so the
  // ramp is a speed for each beat, each its own piece.
  let ramp_beats = if needs_ramp { ramp_beats } else { 0 };
  let ramp: Vec<f64> = (1..=ramp_beats)
    .map(|step| 1.0 + (speed - 1.0) * f64::from(step) / f64::from(ramp_beats + 1))
    .collect();
  let ramp_ms: f64 = ramp.iter().map(|step| ours.period_ms / step).sum();
  let side = f64::from(side_beats) * theirs.period_ms;
  let wanted = join - side - ramp_ms;
  let ramp_file = ours.nearest(file_at(last, wanted));
  let ramp_at = wanted + ramp_file - file_at(last, wanted);
  let blend_file = ramp_file + f64::from(ramp_beats) * ours.period_ms;
  let blend_at = ramp_at + ramp_ms;
  let swap = blend_at + side;
  let theirs_now = file_at(first, swap);
  let later_file_start = float(first.file_start_ms) + theirs.nearest(theirs_now) - theirs_now;
  let plan = Plan {
    ramp_at,
    ramp_file,
    ramp,
    our_beat: ours.period_ms,
    blend_at,
    swap,
    blend_file,
    speed,
    beat: theirs.period_ms,
    later_file_start,
  };
  let earlier_file_end = blend_file + (plan.end() - blend_at) * speed + half_swap;
  let later_from = later_file_start + blend_at - join;
  let later_file_end = later_file_start + float(first.end_ms) - join;
  let fits = ramp_at - half_swap >= float(last.start_ms)
    && earlier_file_end <= float(earlier.file.duration_ms)
    && later_from >= 0.0
    && plan.end() + half_swap < float(first.end_ms)
    && later_file_end <= float(later.file.duration_ms);
  fits.then_some(plan)
}

/// Blends the songs around the join, the largest blend that fits. False
/// where none does.
pub(super) fn blend(
  earlier: &mut Laid<'_>,
  later: &mut Laid<'_>,
  join: u64,
  beats: (Beat, Beat),
  drops_singing: bool,
) -> bool {
  let Some(plan) = SIZES
    .into_iter()
    .find_map(|sizes| plan(earlier, later, float(join), beats, sizes))
  else {
    return false;
  };
  let half_swap = SWAP_MS / 2;
  let (swap, end, side) = (ms(plan.swap), ms(plan.end()), ms(plan.swap - plan.blend_at));
  let (blend_at, beat) = (ms(plan.blend_at), ms(plan.beat));
  let stretched = !plan.ramp.is_empty();

  // The earlier song: as it is up to its ramp, ramping to the later's
  // tempo, then band by band at it.
  let Some(last) = earlier.pieces.last_mut() else {
    return false;
  };
  let template = last.clone();
  last.end_ms = ms(plan.ramp_at) + half_swap;
  last.fade_out = Fade::linear(SWAP_MS);
  let mut step_at = plan.ramp_at;
  for (step, step_speed) in plan.ramp.iter().enumerate() {
    #[expect(clippy::cast_precision_loss, reason = "a few beats")]
    let step_file = plan.ramp_file + step as f64 * plan.our_beat;
    let step_end = step_at + plan.our_beat / step_speed;
    earlier.added.push(Piece {
      file_start_ms: ms(step_file - float(half_swap) * step_speed),
      start_ms: ms(step_at) - half_swap,
      end_ms: ms(step_end) + half_swap,
      fade_in: Fade::linear(SWAP_MS),
      fade_out: Fade::linear(SWAP_MS),
      speed: *step_speed,
      ..template.clone()
    });
    step_at = step_end;
  }
  let going = |tone, end_ms: u64, fade_ms: u64| Piece {
    file_start_ms: ms(plan.blend_file - float(half_swap) * plan.speed),
    start_ms: blend_at - half_swap,
    end_ms,
    fade_in: Fade::linear(SWAP_MS),
    fade_out: Fade::equal_power(fade_ms),
    tone,
    speed: if stretched { plan.speed } else { 1.0 },
    instrumental: template.instrumental || drops_singing,
    ..template.clone()
  };
  earlier.added.extend([
    going(Tone::Bass, swap + beat / 2, beat),
    going(Tone::Mids, swap + side / 2, side * 3 / 4),
    going(Tone::Highs, end, side),
  ]);

  // The later song: band by band into the blend, then as it is.
  let Some(first) = later.pieces.first_mut() else {
    return true;
  };
  let file_start = plan.later_file_start;
  let theirs_at = |at: u64| ms(file_start + float(at) - float(join));
  let template = first.clone();
  let coming = |tone, start_ms: u64, fade_ms: u64| Piece {
    file_start_ms: theirs_at(start_ms),
    start_ms,
    end_ms: end + half_swap,
    fade_in: Fade::equal_power(fade_ms),
    fade_out: Fade::linear(SWAP_MS),
    tone,
    ..template.clone()
  };
  later.added.extend([
    coming(Tone::Highs, blend_at, side),
    coming(Tone::Mids, swap - side / 2, side * 3 / 4),
    coming(Tone::Bass, swap - beat / 2, beat),
  ]);
  first.start_ms = end - half_swap;
  first.file_start_ms = theirs_at(first.start_ms);
  first.fade_in = Fade::linear(SWAP_MS);
  true
}
