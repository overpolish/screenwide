// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! What the library knows of a sound as music, found once as it is measured
//! (`analyse`): its beat, where it holds a steady one. Songs that touch are
//! joined by it: where their tempos are close, the earlier comes to the
//! later's and the two blend on the beat.

mod analyse;

pub(super) use analyse::analyse;

use serde::{Deserialize, Serialize};

/// How far apart, as a share of the slower, two tempos may be for the
/// earlier song to come to the later's: further, the stretch is heard.
const MATCHING_TEMPO: f64 = 0.12;

#[derive(Clone, Copy, Debug, Default, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Music {
  pub beat: Option<Beat>,
}

/// A steady beat: one every `period_ms`, the first `first_ms` into the
/// sound.
#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Beat {
  pub period_ms: f64,
  pub first_ms: f64,
}

impl Beat {
  /// The beat of the sound nearest `at_ms` into it.
  pub fn nearest(&self, at_ms: f64) -> f64 {
    let index = ((at_ms - self.first_ms) / self.period_ms).round();
    self.first_ms + index * self.period_ms
  }

  /// Whether `other`'s tempo is close enough to this one's for a song at
  /// this tempo to be brought to it.
  pub fn matches(&self, other: &Self) -> bool {
    let (slow, fast) = if self.period_ms > other.period_ms {
      (self.period_ms, other.period_ms)
    } else {
      (other.period_ms, self.period_ms)
    };
    slow / fast - 1.0 <= MATCHING_TEMPO
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn tempos_match_within_twelve_percent() {
    let beat = |bpm: f64| Beat {
      period_ms: 60_000.0 / bpm,
      first_ms: 0.0,
    };
    assert!(beat(90.5).matches(&beat(100.6)));
    assert!(beat(100.6).matches(&beat(90.5)));
    assert!(!beat(108.0).matches(&beat(144.0)));
  }
}
