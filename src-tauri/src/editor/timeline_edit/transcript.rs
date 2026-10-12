// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! What the Transcript panel did to a recording: the stretches it cut out,
//! and the words it corrected. The cuts themselves are the gaps between
//! segments, as any cut is; this only says which stretches the tool took, so
//! Restore all can bring back those and no cut of your own. Corrections are
//! text only: nothing plays or exports differently because of either.

use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// More than any recording has words: a cap on what a damaged file may ask
/// to be read.
const MAX_TRANSCRIPT_CUTS: usize = 100_000;
const MAX_TRANSCRIPT_CORRECTIONS: usize = 100_000;
/// A correction is one run of words rewritten by hand; ten thousand
/// characters is far more than any speaker fits in one.
const MAX_CORRECTION_TEXT: usize = 10_000;

/// A stretch of the recording the Transcript panel cut, in shares of the
/// whole recording, and the rate it played at before it was cut, which it
/// plays at again when it is restored.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct RecordingTranscriptCut {
  pub source_start: f64,
  pub source_end: f64,
  pub playback_rate: f64,
}

/// Words the Transcript panel corrected, in shares of the whole recording:
/// the text that replaces what was heard across the stretch. Nothing plays
/// or exports differently because of it.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct RecordingTranscriptCorrection {
  pub source_start: f64,
  pub source_end: f64,
  pub text: String,
}

fn ordered_non_overlapping<T>(
  items: &[T],
  start: impl Fn(&T) -> f64,
  end: impl Fn(&T) -> f64,
) -> bool {
  let mut previous_end = 0.0;
  for item in items {
    if !start(item).is_finite()
      || !end(item).is_finite()
      || start(item) < 0.0
      || end(item) <= start(item)
      || end(item) > 1.0
      || start(item) < previous_end
    {
      return false;
    }
    previous_end = end(item);
  }
  true
}

pub(super) fn validate(
  cuts: &[RecordingTranscriptCut],
  corrections: &[RecordingTranscriptCorrection],
) -> Result<(), String> {
  if cuts.len() > MAX_TRANSCRIPT_CUTS
    || !ordered_non_overlapping(cuts, |cut| cut.source_start, |cut| cut.source_end)
    || cuts
      .iter()
      .any(|cut| !(0.25..=4.0).contains(&cut.playback_rate))
  {
    return Err("The timeline records an invalid transcript cut".to_owned());
  }
  if corrections.len() > MAX_TRANSCRIPT_CORRECTIONS
    || !ordered_non_overlapping(
      corrections,
      |correction| correction.source_start,
      |correction| correction.source_end,
    )
    || corrections
      .iter()
      .any(|correction| correction.text.len() > MAX_CORRECTION_TEXT)
  {
    return Err("The timeline records an invalid transcript correction".to_owned());
  }
  Ok(())
}
