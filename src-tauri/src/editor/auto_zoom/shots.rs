// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Shots: which beats are zoomed into, which share one framing, and how the
//! camera gets from one to the next. Every choice is scored over the whole
//! recording at once, so a zoom is only made where seeing the work up close
//! is worth the moves it costs, and two beats close together share a framing
//! or pan from one to the other rather than zooming out and back in.

mod candidate;
mod resolve;

use super::beats::Beat;
use super::signals::CursorSample;
use crate::editor::scenes::SceneFraming;
use candidate::{candidate, lateness, link, Candidate, MIN_OVERVIEW_MS};

/// How many beats one shot may take in.
const MAX_BEATS_PER_SHOT: usize = 12;

#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct Shot {
  pub start_ms: u64,
  pub end_ms: u64,
  pub framing: SceneFraming,
}

/// The shot of the beats from `last - len` to `last`.
#[derive(Clone, Copy)]
struct ShotState {
  last: usize,
  len: usize,
}

impl ShotState {
  fn first(self) -> usize {
    self.last - self.len
  }
}

/// What comes before a shot.
#[derive(Clone, Copy)]
enum From {
  /// No zoom: the whole screen shows every beat before it.
  Start,
  /// The shot that ends on the beat just before it.
  Direct(ShotState),
  /// This shot, then the whole screen for the beats between them.
  Across(ShotState),
}

#[derive(Clone, Copy)]
struct Entry {
  score: f64,
  from: From,
}

/// The best plan of the beats before one, which leaves the whole screen
/// showing by the time that beat starts, and the last shot in it.
#[derive(Clone, Copy)]
struct Free {
  score: f64,
  shot: Option<ShotState>,
}

fn best(options: impl Iterator<Item = (f64, From)>) -> Option<(f64, From)> {
  options.max_by(|a, b| a.0.total_cmp(&b.0))
}

/// The shots that best show `beats`, in order and never overlapping. Each
/// shot is reached straight from the shot before it, or across beats shown
/// on the whole screen for long enough not to pump; the best way to each is
/// kept, and the best plan of every beat is followed back.
pub(super) fn plan(beats: &[Beat], cursor: &[CursorSample], duration_ms: u64) -> Vec<Shot> {
  let count = beats.len();
  let candidates: Vec<Vec<Option<Candidate>>> = (0..count)
    .map(|last| {
      (0..MAX_BEATS_PER_SHOT.min(last + 1))
        .map(|len| candidate(&beats[last - len..=last], cursor, duration_ms))
        .collect()
    })
    .collect();
  let mut shots: Vec<Vec<Option<Entry>>> = Vec::with_capacity(count);
  let mut free = vec![Free {
    score: 0.0,
    shot: None,
  }];
  for last in 0..count {
    let ending = candidates[last]
      .iter()
      .enumerate()
      .map(|(len, candidate)| {
        let candidate = candidate.as_ref()?;
        let first = last - len;
        let Some(before) = first.checked_sub(1) else {
          return Some(Entry {
            score: candidate.score,
            from: From::Start,
          });
        };
        let (score, from) = best(
          reached(beats, &candidates, &shots, &free, candidate, first)
            .into_iter()
            .chain(direct(
              &candidates[before],
              &shots[before],
              candidate,
              before,
            )),
        )?;
        Some(Entry {
          score: score + candidate.score,
          from,
        })
      })
      .collect::<Vec<_>>();
    // The whole screen shows by the next beat after the shots that have left
    // before it starts.
    let mut next = free[last];
    for (len, entry) in ending.iter().enumerate() {
      let (Some(entry), Some(candidate)) = (entry, candidates[last][len].as_ref()) else {
        continue;
      };
      let clear = beats
        .get(last + 1)
        .is_none_or(|beat| candidate.end_earliest <= beat.start_ms);
      if clear && entry.score > next.score {
        next = Free {
          score: entry.score,
          shot: Some(ShotState { last, len }),
        };
      }
    }
    shots.push(ending);
    free.push(next);
  }
  let mut chosen = Vec::new();
  let mut state = free[count].shot;
  while let Some(shot) = state {
    let Some(entry) = shots[shot.last][shot.len] else {
      break;
    };
    chosen.push((shot, entry.from));
    state = match entry.from {
      From::Start => None,
      From::Direct(previous) | From::Across(previous) => Some(previous),
    };
  }
  chosen.reverse();
  resolve::resolve(&chosen, &candidates, beats)
}

/// The ways to reach `candidate`, the shot starting on beat `first`, with
/// the beats before it on the whole screen: from no zoom at all, or from a
/// shot that left in time for the whole screen to show long enough.
fn reached(
  beats: &[Beat],
  candidates: &[Vec<Option<Candidate>>],
  shots: &[Vec<Option<Entry>>],
  free: &[Free],
  candidate: &Candidate,
  first: usize,
) -> Vec<(f64, From)> {
  let shown_until = beats[first - 1].end_ms;
  let arrive = candidate.start_ideal.max(shown_until);
  if arrive > candidate.start_late {
    return Vec::new();
  }
  let mut options = vec![(-lateness(candidate, arrive), From::Start)];
  // Shown from a beat at least this long before, the whole screen shows long
  // enough whichever shot left before that beat.
  let long_ago =
    beats[..first].partition_point(|beat| beat.start_ms + MIN_OVERVIEW_MS <= shown_until);
  if let Some(far) = long_ago.checked_sub(1) {
    if let Some(shot) = free[far].shot {
      options.push((
        free[far].score - lateness(candidate, arrive),
        From::Across(shot),
      ));
    }
  }
  // Closer than that, each shot that left before the beats it skips must
  // leave the whole screen up long enough before this one arrives.
  for (skipped, skipped_beat) in beats.iter().enumerate().take(first).skip(long_ago.max(1)) {
    let before = skipped - 1;
    for (len, previous) in candidates[before].iter().enumerate() {
      let (Some(previous), Some(entry)) = (previous, shots[before][len]) else {
        continue;
      };
      let start = arrive.max(previous.end_earliest + MIN_OVERVIEW_MS);
      if previous.end_earliest <= skipped_beat.start_ms && start <= candidate.start_late {
        options.push((
          entry.score - lateness(candidate, start),
          From::Across(ShotState { last: before, len }),
        ));
      }
    }
  }
  options
}

/// The ways to reach `candidate` straight from a shot ending on beat
/// `before`, the one just before it.
fn direct<'a>(
  previous: &'a [Option<Candidate>],
  entries: &'a [Option<Entry>],
  candidate: &'a Candidate,
  before: usize,
) -> impl Iterator<Item = (f64, From)> + 'a {
  previous
    .iter()
    .zip(entries)
    .enumerate()
    .filter_map(move |(len, (previous, entry))| {
      let link = link(previous.as_ref()?, candidate)?;
      Some((
        entry.as_ref()?.score + link,
        From::Direct(ShotState { last: before, len }),
      ))
    })
}
