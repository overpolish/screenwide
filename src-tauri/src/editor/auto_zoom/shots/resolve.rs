// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::super::beats::Beat;
use super::candidate::{butts, handover, Candidate, MIN_OVERVIEW_MS};
use super::{From, Shot, ShotState};
use crate::editor::scenes::SceneFraming;

/// The shots `chosen` makes, each timed against its neighbours: a handover
/// shared with a shot it butts, or the whole screen shown for long enough
/// over the beats between it and the last one, and clear of the beats after
/// the last shot of all.
pub(super) fn resolve(
  chosen: &[(ShotState, From)],
  candidates: &[Vec<Option<Candidate>>],
  beats: &[Beat],
) -> Vec<Shot> {
  let candidate_of = |state: ShotState| candidates[state.last][state.len];
  let plan: Vec<(ShotState, Candidate, From)> = chosen
    .iter()
    .filter_map(|&(state, from)| Some((state, candidate_of(state)?, from)))
    .collect();
  let mut starts: Vec<u64> = plan.iter().map(|(_, shot, _)| shot.start_ideal).collect();
  let mut ends: Vec<u64> = plan.iter().map(|(_, shot, _)| shot.end_ideal).collect();
  for (index, &(state, shot, from)) in plan.iter().enumerate() {
    let shown_until = state
      .first()
      .checked_sub(1)
      .map_or(0, |before| beats[before].end_ms);
    match from {
      From::Start => {
        starts[index] = shot.start_ideal.max(shown_until).min(shot.start_late);
      }
      From::Direct(_) => {
        let previous = plan[index - 1].1;
        if butts(&previous, &shot) {
          let at = handover(&previous, &shot);
          ends[index - 1] = at;
          starts[index] = at;
        }
      }
      From::Across(before) => {
        let previous = plan[index - 1].1;
        let mut left = previous.end_ideal.min(beats[before.last + 1].start_ms);
        let mut arrive = shot.start_ideal.max(shown_until);
        if arrive < left + MIN_OVERVIEW_MS {
          left = previous
            .end_earliest
            .max(arrive.saturating_sub(MIN_OVERVIEW_MS))
            .min(left);
          arrive = arrive.max(left + MIN_OVERVIEW_MS);
        }
        ends[index - 1] = left;
        starts[index] = arrive.min(shot.start_late);
      }
    }
  }
  if let Some(&(state, shot, _)) = plan.last() {
    if let Some(after) = beats.get(state.last + 1) {
      let last = plan.len() - 1;
      ends[last] = shot.end_ideal.min(after.start_ms).max(shot.end_earliest);
    }
  }
  plan
    .iter()
    .zip(starts.into_iter().zip(ends))
    .filter(|(_, (start_ms, end_ms))| end_ms > start_ms)
    .map(|((_, shot, _), (start_ms, end_ms))| Shot {
      start_ms,
      end_ms,
      framing: SceneFraming {
        focus_x: shot.focus.0,
        focus_y: shot.focus.1,
        zoom: shot.zoom,
      },
    })
    .collect()
}
