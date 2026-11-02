// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Beats: runs of clicks, drags and typing close together in time and place,
//! each one stretch of work a single framing can show. Planning shots merges
//! beats further; a beat itself is never split.

use super::signals::{CursorSample, Signals};
use super::Area;

/// The longest pause inside one beat.
const BEAT_GAP_MS: u64 = 1_200;
/// The longest pause between keys, or between the last click and a key, that
/// still holds a beat on: typing into the field just clicked.
const TYPING_GAP_MS: u64 = 2_000;
/// The widest a beat may grow, as a share of the picture. Past it, the work
/// is spread over too much of the screen for a zoom to help.
pub(super) const MAX_BEAT_SPAN: f64 = 0.56;
const TYPING_WEIGHT_PER_KEY: f64 = 0.1;
const MAX_TYPING_WEIGHT: f64 = 1.5;
/// The share of the pointer's time on each side left out when a beat takes
/// in where the pointer worked, so one flick away does not widen it.
const CURSOR_TRIM: f64 = 0.1;

#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct Beat {
  pub start_ms: u64,
  pub end_ms: u64,
  pub area: Area,
  pub weight: f64,
  /// Another app came to the front here. No shot takes it in, so the whole
  /// screen shows the switch and no zoom carries across it.
  pub is_switch: bool,
  /// The press that brought an app forward, with any typing after it. Where
  /// that press lands, a title bar or wherever the window was caught, says
  /// little about the work to come, so it is zoomed into only on its own.
  pub brought_forward: bool,
  /// Text was typed in it. Where typing goes is all the beat says, and the
  /// text runs on from there, so a shot of it is framed on it.
  pub typed: bool,
}

impl Beat {
  fn switch(ms: u64) -> Self {
    Self {
      start_ms: ms,
      end_ms: ms,
      area: Area {
        left: 0.0,
        top: 0.0,
        right: 1.0,
        bottom: 1.0,
      },
      weight: 0.0,
      is_switch: true,
      brought_forward: false,
      typed: false,
    }
  }
}

pub(super) fn group(signals: &Signals) -> Vec<Beat> {
  let mut beats: Vec<Beat> = Vec::new();
  let mut switches = signals.switches.iter().copied().peekable();
  for activity in &signals.activities {
    // A switch at the moment of a press comes first: the press that caused
    // it was made in the app it brought forward.
    while let Some(switch) = switches.next_if(|&switch| switch <= activity.start_ms) {
      beats.push(Beat::switch(switch));
    }
    if let Some(beat) = beats
      .last_mut()
      .filter(|beat| !beat.is_switch && !beat.brought_forward)
    {
      let area = beat.area.union(activity.area);
      if activity.start_ms <= beat.end_ms + BEAT_GAP_MS && area.span() <= MAX_BEAT_SPAN {
        beat.end_ms = beat.end_ms.max(activity.end_ms);
        beat.area = area;
        beat.weight += activity.weight;
        continue;
      }
    }
    beats.push(Beat {
      start_ms: activity.start_ms,
      end_ms: activity.end_ms,
      area: activity.area,
      weight: activity.weight,
      is_switch: false,
      brought_forward: activity.brought_forward,
      typed: false,
    });
  }
  beats.extend(switches.map(Beat::switch));
  let next_starts: Vec<u64> = beats
    .iter()
    .skip(1)
    .map(|beat| beat.start_ms)
    .chain(std::iter::once(u64::MAX))
    .collect();
  for (beat, next_start) in beats.iter_mut().zip(next_starts) {
    if !beat.is_switch {
      hold_for_typing(beat, signals, next_start);
      take_in_pointer(beat, &signals.cursor);
    }
  }
  beats
}

/// Keys pressed during `beat`, or soon after and before the next beat starts,
/// hold it on and make it matter more: typed text is small on screen.
fn hold_for_typing(beat: &mut Beat, signals: &Signals, next_start: u64) {
  let keys = &signals.keys;
  let first = keys.partition_point(|&key| key < beat.start_ms);
  let mut typed = 0_u32;
  for &key in &keys[first..] {
    if key >= next_start || key > beat.end_ms + TYPING_GAP_MS {
      break;
    }
    beat.end_ms = beat.end_ms.max(key);
    typed += 1;
  }
  beat.weight += (f64::from(typed) * TYPING_WEIGHT_PER_KEY).min(MAX_TYPING_WEIGHT);
  let typing = &signals.typing;
  beat.typed = typing.partition_point(|&key| key < beat.start_ms)
    < typing.partition_point(|&key| key <= beat.end_ms);
}

/// Widens `beat` to where the pointer spent most of its time over it, where
/// that still fits in a beat: the work between the clicks happens there too.
fn take_in_pointer(beat: &mut Beat, cursor: &[CursorSample]) {
  let first = cursor.partition_point(|sample| sample.ms < beat.start_ms);
  let last = cursor.partition_point(|sample| sample.ms <= beat.end_ms);
  let mut xs: Vec<f64> = Vec::with_capacity(last.saturating_sub(first));
  let mut ys: Vec<f64> = Vec::with_capacity(xs.capacity());
  for sample in &cursor[first..last] {
    if (0.0..=1.0).contains(&sample.x) && (0.0..=1.0).contains(&sample.y) {
      xs.push(sample.x);
      ys.push(sample.y);
    }
  }
  if xs.len() < 2 {
    return;
  }
  xs.sort_by(f64::total_cmp);
  ys.sort_by(f64::total_cmp);
  let at =
    |values: &[f64], share: f64| values[((values.len() - 1) as f64 * share).round() as usize];
  let worked = Area {
    left: at(&xs, CURSOR_TRIM),
    top: at(&ys, CURSOR_TRIM),
    right: at(&xs, 1.0 - CURSOR_TRIM),
    bottom: at(&ys, 1.0 - CURSOR_TRIM),
  };
  let area = beat.area.union(worked);
  if area.span() <= MAX_BEAT_SPAN {
    beat.area = area;
  }
}
