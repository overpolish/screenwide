// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Typing followed where its caret went. A line of text runs on past any
//! frame that showed where it began, so where the caret was recorded, a typed
//! beat becomes the work before the typing and a run of beats along the
//! text, each as much of it as one close framing shows; the shots made of
//! them pan from one to the next as the text goes.

use super::super::signals::Signals;
use super::super::Area;
use super::{key_weight, Beat};

/// The widest a run of typing may grow, as a share of the picture: the most
/// the closest auto zoom shows with room round it.
const RUN_SPAN: f64 = 0.35;
/// How long before the caret reaches the next run a run lets it go, so the
/// camera sets off ahead of the text rather than after it: a shot holds a
/// moment after its work, and the next arrives before its own
/// (`HANDOVER_HOLD_MS` and `TRANSITION_MS` in `shots/candidate.rs`).
const PAN_AHEAD_MS: u64 = 900;

/// The work before `beat`'s typing and the runs of its typing along the
/// caret, or `None` where no caret was recorded in it. `typing_weight` is
/// what its keys added to its weight; each run is weighed by its own keys.
pub(super) fn follow_caret(
  beat: &Beat,
  typing_weight: f64,
  signals: &Signals,
) -> Option<(Beat, Vec<Beat>)> {
  if !beat.typed {
    return None;
  }
  let carets = &signals.carets;
  let carets = &carets[carets.partition_point(|caret| caret.ms < beat.start_ms)
    ..carets.partition_point(|caret| caret.ms <= beat.end_ms)];
  let first = carets.first()?;
  let mut runs: Vec<(u64, Area)> = Vec::new();
  for caret in carets {
    match runs.last_mut() {
      Some((_, area)) if area.union(caret.area).span() <= RUN_SPAN => {
        *area = area.union(caret.area);
      }
      _ => runs.push((caret.ms, caret.area)),
    }
  }
  let lead = Beat {
    end_ms: first.ms.max(beat.start_ms),
    weight: (beat.weight - typing_weight).max(0.0),
    typed: false,
    ..*beat
  };
  let ends = runs
    .iter()
    .skip(1)
    .map(|&(start, _)| start.saturating_sub(PAN_AHEAD_MS))
    .chain(std::iter::once(beat.end_ms + 1));
  let keys = &signals.keys;
  let runs = runs
    .iter()
    .zip(ends)
    .map(|(&(start_ms, area), next)| {
      let from = keys.partition_point(|&key| key < start_ms);
      // The next run may begin sooner after this one than the caret is let
      // go early, which ends this one before it starts: it then has no keys.
      let to = keys.partition_point(|&key| key < next).max(from);
      Beat {
        start_ms,
        end_ms: keys[from..to]
          .last()
          .copied()
          .unwrap_or(start_ms)
          .max(start_ms),
        area,
        weight: key_weight(to - from),
        is_switch: false,
        brought_forward: false,
        typed: true,
      }
    })
    .collect();
  Some((lead, runs))
}
