// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! When each caption leaves. Without No gaps a caption stays up through a
//! pause the speaker usually makes between phrases, and goes once they stop;
//! with No gaps it stays until the next, unless the speaker stops for long
//! enough that a caption hanging over the silence would read as stale.

use crate::captions::{Caption, CaptionSettings};

/// A caption shows for at least this long, where the next leaves it room.
const MIN_SHOWN_MS: f64 = 700.0;
/// The longest silence No gaps bridges; a caption hung over a longer one, a
/// stretch of the video with nothing said, reads as left behind.
const NO_GAPS_LIMIT_MS: f64 = 4_000.0;

/// How long the speaker's pause has to run to be the speaker stopping, the
/// twin of `dead_air_ms` in `screenwide_speech`'s `silences.rs`, which Remove
/// silences cuts by: a pause shorter than this is a breath between phrases.
mod dead_air {
  /// A gap shorter than this sits inside a phrase and says nothing about how
  /// long the speaker pauses between them.
  pub(super) const SHORTEST_PAUSE_MS: f64 = 150.0;
  /// A pause is the speaker stopping once it runs this many times their
  /// usual one...
  const USUAL_PAUSE_FACTOR: f64 = 2.0;
  /// ...within these bounds.
  const BOUNDS_MS: (f64, f64) = (700.0, 1_500.0);
  /// Without enough pauses to learn from, the speaker's usual pause is taken
  /// to give this.
  const DEFAULT_MS: f64 = 1_000.0;
  /// How many pauses it takes to learn the speaker's usual one.
  const PAUSES_TO_LEARN: usize = 3;

  pub(super) fn ms(mut pauses: Vec<f64>) -> f64 {
    if pauses.len() < PAUSES_TO_LEARN {
      return DEFAULT_MS;
    }
    pauses.sort_by(f64::total_cmp);
    let usual = pauses[pauses.len() / 2];
    (usual * USUAL_PAUSE_FACTOR).clamp(BOUNDS_MS.0, BOUNDS_MS.1)
  }
}

/// How long a pause runs before it is the speaker stopping, learnt from the
/// pauses between every word the captions show, on the edited video.
fn stopping_ms(captions: &[Caption]) -> f64 {
  let words = captions
    .iter()
    .flat_map(|caption| caption.lines.iter().flatten());
  let pauses = words
    .clone()
    .zip(words.skip(1))
    .map(|(before, after)| after.start_ms - before.end_ms)
    .filter(|&pause| pause >= dead_air::SHORTEST_PAUSE_MS)
    .collect();
  dead_air::ms(pauses)
}

/// Settles when each caption leaves: never before it has shown for a
/// moment, through a pause too short to be the speaker stopping, with No
/// gaps through any silence short of [`NO_GAPS_LIMIT_MS`], and never after
/// the next caption starts or the video ends.
pub(crate) fn timed(captions: &mut [Caption], settings: &CaptionSettings, length_ms: f64) {
  let bridged_ms = if settings.no_gaps {
    NO_GAPS_LIMIT_MS
  } else {
    stopping_ms(captions)
  };
  for index in 0..captions.len() {
    let next = captions.get(index + 1).map(|next| next.start_ms);
    let caption = &mut captions[index];
    let spoken_end = caption.end_ms;
    caption.end_ms = caption.end_ms.max(caption.start_ms + MIN_SHOWN_MS);
    if let Some(next) = next {
      // Measured from the last word, not the stretched end: a short caption
      // held up for reading is no reason to bridge a longer silence.
      if next - spoken_end < bridged_ms {
        caption.end_ms = next;
      }
      caption.end_ms = caption.end_ms.min(next);
    }
    caption.end_ms = caption.end_ms.min(length_ms).max(caption.start_ms);
  }
}
