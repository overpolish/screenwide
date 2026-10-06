// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Where the pointer is against a shot's frame. The pointer is where the
//! attention is, so how long it stays out of a frame says how long that frame
//! has stopped showing the work.

use super::super::signals::Signals;
use super::super::Area;

/// How far outside its frame the pointer may be and still count as seen.
const COVER_MARGIN: f64 = 0.02;

/// The seconds the pointer spends outside `frame` from `from` to `to`.
pub(super) fn outside_seconds(signals: &Signals, frame: Area, from: u64, to: u64) -> f64 {
  outside(signals, frame, from, to)
    .iter()
    .map(|&(begin, end)| end - begin)
    .sum::<u64>() as f64
    / 1_000.0
}

/// The stretches from `from` to `to` the pointer spends outside `frame` with
/// no key pressed. A key brings the attention back for a moment: someone
/// typing often parks the pointer out of the way of the text.
pub(super) fn away(signals: &Signals, frame: Area, from: u64, to: u64) -> Vec<(u64, u64)> {
  let keys = &signals.keys;
  let mut stretches = Vec::new();
  for (begin, end) in outside(signals, frame, from, to) {
    let mut start = begin;
    let first = keys.partition_point(|&key| key <= begin);
    for &key in keys[first..].iter().take_while(|&&key| key < end) {
      stretches.push((start, key));
      start = key;
    }
    stretches.push((start, end));
  }
  stretches
}

/// The stretches from `from` to `to` the pointer spends outside `frame`.
fn outside(signals: &Signals, frame: Area, from: u64, to: u64) -> Vec<(u64, u64)> {
  let cursor = &signals.cursor;
  let first = cursor
    .partition_point(|sample| sample.ms <= from)
    .saturating_sub(1);
  let mut stretches = Vec::new();
  let mut open: Option<u64> = None;
  for (index, sample) in cursor.iter().enumerate().skip(first) {
    if sample.ms >= to {
      break;
    }
    let begin = sample.ms.max(from);
    let end = cursor.get(index + 1).map_or(to, |next| next.ms.min(to));
    if end <= begin {
      continue;
    }
    if frame.contains(sample.x, sample.y, COVER_MARGIN) {
      if let Some(start) = open.take() {
        stretches.push((start, begin));
      }
    } else {
      open.get_or_insert(begin);
    }
  }
  if let Some(start) = open {
    stretches.push((start, to));
  }
  stretches
}
