// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use std::collections::BTreeMap;

use super::super::model::PinKeyframe;
use super::super::target::PinTarget;
use super::super::tracker::Status;
use super::covers::placed;
use super::{Raw, Sides, LOST_GRACE_MS};

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

/// Whether the annotation is in sight at each moment, where `[dx, dy,
/// scale]` put it on a recording `source` pixels in size. A redaction is
/// while any of its box is on the frame and clear of the cover it is trimmed
/// to, since the part of what it hides still in sight must stay hidden; any
/// other kind while its content was followed or lost on screen, with its
/// anchor on the frame.
pub(super) fn in_sight(
  raw: &[Raw],
  [dx, dy, scale]: [&[f32]; 3],
  target: &PinTarget,
  source: (u32, u32),
) -> Vec<bool> {
  let (width, height) = (f64::from(source.0), f64::from(source.1));
  (0..raw.len())
    .map(|index| {
      if target.redaction {
        let [x0, y0, x1, y1] = placed(target, dx[index], dy[index], scale[index]);
        let [left, top, right, bottom] = raw[index].view.map(f64::from);
        let (x0, y0, x1, y1) = (x0.max(left), y0.max(top), x1.min(right), y1.min(bottom));
        x1 > x0.max(0.0) && y1 > y0.max(0.0) && x0 < x1.min(width) && y0 < y1.min(height)
      } else {
        let (x, y) = (f64::from(dx[index]), f64::from(dy[index]));
        let (ax, ay) = (target.anchor[0] + x, target.anchor[1] + y);
        raw[index].status != Status::Hidden
          && !raw[index].under
          && ax >= 0.0
          && ay >= 0.0
          && ax < width
          && ay < height
      }
    })
    .collect()
}

/// Carries a stretch off the frame or under a cover on from its ends: its
/// first half at the speed the content left at, its second back from where
/// and how fast it came in. Bridging straight across would hold it near the
/// edge it left by, and a redaction still partly in sight must go on
/// covering what is left of its content there.
pub(super) fn carry_hidden(raw: &[Raw], dx: &mut [f32], dy: &mut [f32]) {
  let gone = |index: usize| raw[index].status == Status::Hidden || raw[index].under;
  for (from, to) in runs(raw.len(), gone) {
    let before = from.checked_sub(1);
    let after = (to + 1 < raw.len()).then_some(to + 1);
    // How far a frame moves the content away from `at`, measured from its
    // neighbour on the far side: onward in time from the end it left by,
    // back in time from the end it came in by.
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
        (Some(leaving), Some(_)) if first_half => Some(leaving),
        (Some(leaving), None) => Some(leaving),
        (_, Some(arriving)) => Some(arriving),
        _ => None,
      };
      let Some((at, [vx, vy])) = end else {
        continue;
      };
      let frames = index.abs_diff(at) as f32;
      dx[index] = dx[at] + vx * frames;
      dy[index] = dy[at] + vy * frames;
    }
  }
}

/// Carries a stretch the content was lost in straight across, in time, from
/// where it was last seen to where it was seen again; held at the one end it
/// has when it has only one. Nothing is known of where it went in between:
/// smoothing would carry it on at the speed it left at and back at the speed
/// it returned at, far past both ends, and a redaction grown over that would
/// cover much of the screen for nothing.
pub(super) fn bridge_lost(raw: &[Raw], values: &mut [&mut [f32]]) {
  let lost = |index: usize| raw[index].status == Status::Lost && !raw[index].under;
  for (from, to) in runs(raw.len(), lost) {
    let before = from.checked_sub(1);
    let after = (to + 1 < raw.len()).then_some(to + 1);
    for values in values.iter_mut() {
      for index in from..=to {
        values[index] = match (before, after) {
          (Some(before), Some(after)) => {
            let span = raw[after].ms.saturating_sub(raw[before].ms).max(1) as f32;
            let share = raw[index].ms.saturating_sub(raw[before].ms) as f32 / span;
            values[before] + share * (values[after] - values[before])
          }
          (Some(end), None) | (None, Some(end)) => values[end],
          (None, None) => values[index],
        };
      }
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

/// Between each pair of keyframes, where neither leg saw the content for
/// longer than `grace_ms`: from the first such stretch's start to the last
/// one's end. A correction made after the content came back says nothing
/// about how it sat before it went, so each side keeps to its own keyframe
/// there instead of being blended towards the other across still frames.
pub(super) fn unseen_between(
  moments: &BTreeMap<u64, Sides>,
  keyframes: &[PinKeyframe],
  grace_ms: u64,
) -> Vec<Option<(u64, u64)>> {
  keyframes
    .windows(2)
    .map(|pair| {
      let (before, after) = (pair[0].ms, pair[1].ms);
      let mut span: Option<(u64, u64)> = None;
      let mut start = None;
      let mut close = |from: u64, to: u64| {
        if to.saturating_sub(from) >= grace_ms {
          span = Some(span.map_or((from, to), |(first, _)| (first, to)));
        }
      };
      for (&ms, sides) in moments.range(before.saturating_add(1)..after) {
        let seen = [&sides.forward, &sides.backward]
          .into_iter()
          .flatten()
          .any(|(_, sample)| sample.status == Status::Tracked);
        match (seen, start) {
          (false, None) => start = Some(ms),
          (true, Some(from)) => {
            close(from, ms);
            start = None;
          }
          _ => {}
        }
      }
      if let Some(from) = start {
        close(from, after);
      }
      span
    })
    .collect()
}

/// Gives each keyframe the hand said is out of view a moment of its own in
/// `raw`, hidden, so the stretch shown before it ends there; and says which
/// moments fall in a stretch out of view, from such a keyframe to the next.
/// Nothing was followed there, so these are the only moments it has.
pub(super) fn out_of_view(raw: &mut Vec<Raw>, keyframes: &[PinKeyframe]) -> Vec<bool> {
  let mut spans = Vec::new();
  for (index, keyframe) in keyframes.iter().enumerate() {
    if !keyframe.out_of_view {
      continue;
    }
    let end = keyframes.get(index + 1).map_or(u64::MAX, |next| next.ms);
    spans.push((keyframe.ms, end));
    let at = raw.partition_point(|moment| moment.ms < keyframe.ms);
    let before = at.checked_sub(1).map(|before| raw[before]);
    let moment = Raw {
      ms: keyframe.ms,
      confidence: 0.0,
      status: Status::Hidden,
      ..before.unwrap_or(Raw::exact(keyframe, keyframe.ms))
    };
    match raw.get_mut(at) {
      Some(existing) if existing.ms == keyframe.ms => *existing = moment,
      _ => raw.insert(at, moment),
    }
  }
  raw
    .iter()
    .map(|moment| {
      spans
        .iter()
        .any(|&(start, end)| moment.ms >= start && moment.ms < end)
    })
    .collect()
}

/// Hides what is out of sight, in `visible`, and says which moments that
/// hid, for bringing the annotation back in view there.
///
/// An arrow, counter or text box whose content is out of sight on screen
/// for more than a moment has gone under something - a sticky header, a
/// window moved over it - and pointing at what covers it misleads. A shorter
/// dropout, a hover or a cursor passing, stays bridged rather than making the
/// annotation blink. A redaction stays, and grows. Content gone under a
/// still cover is out of sight at once, and so is what the hand said is out
/// of view, whatever the kind.
pub(super) fn hide_unseen(
  raw: &[Raw],
  visible: &mut [bool],
  unseen_by_hand: &[bool],
  redaction: bool,
  span: &dyn Fn((usize, usize)) -> [u64; 2],
) -> Vec<bool> {
  let mut covered = vec![false; raw.len()];
  if !redaction {
    for (from, to) in runs(raw.len(), |index| raw[index].status == Status::Lost) {
      let [start, end] = span((from, to));
      if end.saturating_sub(start) >= LOST_GRACE_MS {
        covered[from..=to].fill(true);
      }
    }
  }
  for (index, hidden) in covered.iter_mut().enumerate() {
    *hidden |= unseen_by_hand[index] || (raw[index].under && !visible[index]);
    if *hidden {
      visible[index] = false;
    }
  }
  covered
}
