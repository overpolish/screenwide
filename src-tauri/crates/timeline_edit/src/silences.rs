// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! A record of the pauses Remove silences cut out of a recording. The cuts
//! themselves are the gaps between segments, as any cut is; this only says
//! which stretches the tool took, so Restore all can bring back those and no
//! cut of your own. Nothing plays or exports differently because of it.

use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// More than any recording has pauses: a cap on what a damaged file may ask
/// to be read.
const MAX_SILENCE_CUTS: usize = 100_000;

/// A stretch of the recording Remove silences cut, in shares of the whole
/// recording, and the rate it played at before it was cut, which it plays at
/// again when it is restored.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct RecordingSilenceCut {
  pub source_start: f64,
  pub source_end: f64,
  pub playback_rate: f64,
}

pub(super) fn validate(cuts: &[RecordingSilenceCut]) -> Result<(), String> {
  if cuts.len() > MAX_SILENCE_CUTS
    || cuts.iter().any(|cut| {
      !cut.source_start.is_finite()
        || !cut.source_end.is_finite()
        || cut.source_start < 0.0
        || cut.source_end > 1.0
        || cut.source_end <= cut.source_start
        || !(0.25..=4.0).contains(&cut.playback_rate)
    })
  {
    return Err("The timeline records an invalid removed silence".to_owned());
  }
  Ok(())
}
