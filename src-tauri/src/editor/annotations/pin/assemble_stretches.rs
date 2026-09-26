// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::super::tracker::Status;
use super::Raw;

/// How far past its doubtful stretch a redaction is grown, as a share of its
/// box's longer side.
const GROWTH_MARGIN: f32 = 0.05;

/// Runs of consecutive indices where `keep` holds.
pub(super) fn runs(count: usize, keep: impl Fn(usize) -> bool) -> Vec<(usize, usize)> {
  let mut out = Vec::new();
  let mut start = None;
  for index in 0..=count {
    match (start, index < count && keep(index)) {
      (None, true) => start = Some(index),
      (Some(from), false) => {
        out.push((from, index - 1));
        start = None;
      }
      _ => {}
    }
  }
  out
}

/// Carries a stretch off the frame on from its ends: its first half at the
/// speed the content left at, its second back from where and how fast it
/// came in. Bridging straight across would hold it near the edge it left by,
/// and a redaction still partly on the frame must go on covering what is
/// left of its content there.
pub(super) fn carry_hidden(raw: &[Raw], dx: &mut [f32], dy: &mut [f32]) {
  for (from, to) in runs(raw.len(), |index| raw[index].status == Status::Hidden) {
    let before = from.checked_sub(1);
    let after = (to + 1 < raw.len()).then_some(to + 1);
    let speed = |at: usize, towards: Option<usize>| {
      towards.map_or([0.0; 2], |other| {
        let frames = at.abs_diff(other) as f32;
        [(dx[at] - dx[other]) / frames, (dy[at] - dy[other]) / frames]
      })
    };
    let leaving = before.map(|at| (at, speed(at, at.checked_sub(1))));
    let arriving = after.map(|at| (at, speed(at, (at + 1 < raw.len()).then_some(at + 1))));
    for index in from..=to {
      let first_half = index - from <= to - index;
      let end = match (leaving, arriving) {
        (Some(leaving), Some(_)) if first_half => Some((leaving, true)),
        (Some(leaving), None) => Some((leaving, true)),
        (_, Some(arriving)) => Some((arriving, false)),
        _ => None,
      };
      let Some(((at, [vx, vy]), forward)) = end else {
        continue;
      };
      let frames = index.abs_diff(at) as f32;
      let sign = if forward { 1.0 } else { -1.0 };
      dx[index] = dx[at] + sign * vx * frames;
      dy[index] = dy[at] + sign * vy * frames;
    }
  }
}

/// How far a redaction is grown at each sample: over each doubtful stretch
/// and the trusted frame on either side of it, out to every place its
/// content was put, and a little past.
pub(super) fn grown(
  weak: &[(usize, usize)],
  dx: &[f32],
  dy: &[f32],
  region: [f64; 4],
) -> Vec<[f32; 4]> {
  let mut growth = vec![[0.0_f32; 4]; dx.len()];
  let margin = GROWTH_MARGIN * (region[2] - region[0]).max(region[3] - region[1]) as f32;
  for &(from, to) in weak {
    let (first, last) = (from.saturating_sub(1), (to + 1).min(dx.len() - 1));
    let span = first..=last;
    let min_x = span
      .clone()
      .map(|index| dx[index])
      .fold(f32::INFINITY, f32::min);
    let max_x = span
      .clone()
      .map(|index| dx[index])
      .fold(f32::NEG_INFINITY, f32::max);
    let min_y = span
      .clone()
      .map(|index| dy[index])
      .fold(f32::INFINITY, f32::min);
    let max_y = span
      .map(|index| dy[index])
      .fold(f32::NEG_INFINITY, f32::max);
    for index in from..=to {
      growth[index] = [
        dx[index] - min_x + margin,
        dy[index] - min_y + margin,
        max_x - dx[index] + margin,
        max_y - dy[index] + margin,
      ];
    }
  }
  growth
}
