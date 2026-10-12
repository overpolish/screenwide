// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Content lost where it went under a still cover - a sticky header, a
//! toolbar - and came back out from under it.
//!
//! A leg proves a cover's edge only once content has moved across it for a
//! few frames, and a header may stick only as the content reaches it, so a
//! stretch the content was lost in is judged with what was proven on both
//! sides of it. It went under a cover when a proven edge meets the box as it
//! was lost, heading into the cover, and as it was found again, heading out.
//! Then it is carried under the cover at the speed it went and came, and
//! trimmed to the edge on its way in and out, rather than bridged straight
//! across and grown. Evidence on one side alone is not enough: content lost
//! half under a cover may have stopped there, still partly in sight.

use super::super::target::PinTarget;
use super::super::tracker::Status;
use super::stretches::runs;
use super::Raw;

/// How far either side of a lost stretch a proven edge is taken from, in
/// milliseconds.
const NEAR_MS: u64 = 500;
/// How far apart two edges found either side of a stretch may be, in source
/// pixels, and be one cover's.
const AGREE: f32 = 8.0;
/// The least movement a frame, in source pixels, for content to be heading
/// into or out of a cover.
const MIN_SPEED: f32 = 0.5;
/// How far short of an edge the box may stop, as a share of its size along
/// the axis, and still meet it.
const REACH: f64 = 0.1;

/// `target`'s box moved by `[dx, dy]` and grown by `scale` about its anchor,
/// in source pixels: left, top, right, bottom.
pub(super) fn placed(target: &PinTarget, dx: f32, dy: f32, scale: f32) -> [f64; 4] {
  let (x, y, s) = (f64::from(dx), f64::from(dy), f64::from(scale));
  let [ax, ay] = target.anchor;
  let place = |value: f64, anchor: f64, shift: f64| anchor + shift + s * (value - anchor);
  [
    place(target.region[0], ax, x),
    place(target.region[1], ay, y),
    place(target.region[2], ax, x),
    place(target.region[3], ay, y),
  ]
}

/// The side of the frame a cover lies on, as an index into a view: left,
/// top, right, bottom. A cover on the left or top lies before its edge.
fn before(side: usize) -> bool {
  side < 2
}

/// Which axis a side's edge lies across: 0 for x, 1 for y.
fn axis(side: usize) -> usize {
  side % 2
}

/// Whether `bounds`, moving `speed` a frame along the side's axis, meets the
/// edge at `edge` on `side`: reaching it or past it, heading into the cover
/// when `into`, out of it otherwise.
fn meets(bounds: [f64; 4], side: usize, edge: f32, speed: f32, into: bool) -> bool {
  let (low, high) = (bounds[axis(side)], bounds[axis(side) + 2]);
  let reach = REACH * (high - low);
  let edge = f64::from(edge);
  let (reached, towards) = if before(side) {
    (low < edge + reach, speed < -MIN_SPEED)
  } else {
    (high > edge - reach, speed > MIN_SPEED)
  };
  reached && towards == into && speed.abs() > MIN_SPEED
}

/// The edge on `side` proven nearest in time to the stretch from `from` to
/// `to`, within [`NEAR_MS`] of it on either side; where both sides have one,
/// only if they agree.
fn edge_near(raw: &[Raw], from: usize, to: usize, side: usize) -> Option<f32> {
  let found = |index: &usize| raw[*index].proven[side].is_finite();
  let earlier = (0..from)
    .rev()
    .take_while(|&index| raw[from].ms - raw[index].ms <= NEAR_MS)
    .find(found);
  let later = (to + 1..raw.len())
    .take_while(|&index| raw[index].ms - raw[to].ms <= NEAR_MS)
    .find(found);
  match (earlier, later) {
    (Some(a), Some(b)) => {
      let (a, b) = (raw[a].proven[side], raw[b].proven[side]);
      ((a - b).abs() <= AGREE).then_some(0.5 * (a + b))
    }
    (Some(one), None) | (None, Some(one)) => Some(raw[one].proven[side]),
    (None, None) => None,
  }
}

/// Marks each stretch the content was lost in that it spent under a still
/// cover, where `[dx, dy, scale]` put `target`, and trims every moment it
/// was crossing the cover's edge to it.
pub(super) fn under_covers(raw: &mut [Raw], [dx, dy, scale]: [&[f32]; 3], target: &PinTarget) {
  let count = raw.len();
  let bounds = |index: usize| placed(target, dx[index], dy[index], scale[index]);
  let speed = |at: usize, from: usize, side: usize| {
    let values = if axis(side) == 0 { dx } else { dy };
    values[at] - values[from]
  };
  for (from, to) in runs(count, |index| raw[index].status == Status::Lost) {
    let (Some(went), came) = (from.checked_sub(1), to + 1) else {
      continue;
    };
    if came >= count {
      continue;
    }
    for side in 0..4 {
      let Some(edge) = edge_near(raw, from, to, side) else {
        continue;
      };
      let going = went
        .checked_sub(1)
        .map_or(0.0, |earlier| speed(went, earlier, side));
      let coming = if came + 1 < count {
        speed(came + 1, came, side)
      } else {
        0.0
      };
      if !meets(bounds(went), side, edge, going, true)
        || !meets(bounds(came), side, edge, coming, false)
      {
        continue;
      }
      let trim = |raw: &mut Raw| {
        raw.view[side] = if before(side) {
          raw.view[side].max(edge)
        } else {
          raw.view[side].min(edge)
        };
      };
      for moment in &mut raw[from..=to] {
        moment.under = true;
        trim(moment);
      }
      // On its way in and out, followed but already crossing the edge.
      let crossing = |index: usize| {
        let [low, high] = [bounds(index)[axis(side)], bounds(index)[axis(side) + 2]];
        let edge = f64::from(edge);
        if before(side) {
          low < edge
        } else {
          high > edge
        }
      };
      for index in (0..from).rev() {
        if raw[from].ms - raw[index].ms > NEAR_MS || !crossing(index) {
          break;
        }
        trim(&mut raw[index]);
      }
      for index in came..count {
        if raw[index].ms - raw[to].ms > NEAR_MS || !crossing(index) {
          break;
        }
        trim(&mut raw[index]);
      }
    }
  }
}
