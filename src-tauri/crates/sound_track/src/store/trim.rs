// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Dead space cut from a sound effect as it is added. Sound effects shared
//! on sites such as Freesound often start a second or more after their file
//! does and end long before it, which would leave a clip silent at its
//! start and too long on the timeline. A song is kept as it is: a quiet
//! intro or outro is part of it.

use std::path::Path;

use crate::clip::SONG_MS;

/// How much is kept before what is heard starts, so its attack is never
/// clipped, and after it ends, for a tail too quiet to measure, in ms.
const KEPT_BEFORE_MS: u64 = 20;
const KEPT_AFTER_MS: u64 = 150;
/// The least dead space worth cutting from either end, in ms.
const LEAST_CUT_MS: u64 = 50;

/// The stretch of a sound `duration_ms` long, heard from and to
/// `heard_ms`, it is cut to as a sound effect: none for one that would be a
/// song, or where there is next to nothing to cut.
pub(super) fn kept(duration_ms: u64, heard_ms: Option<(u64, u64)>) -> Option<(u64, u64)> {
  let (from_ms, to_ms) = heard_ms?;
  let start_ms = from_ms.saturating_sub(KEPT_BEFORE_MS);
  let end_ms = (to_ms + KEPT_AFTER_MS).min(duration_ms);
  let is_worth_it = start_ms >= LEAST_CUT_MS || duration_ms - end_ms >= LEAST_CUT_MS;
  (end_ms - start_ms < SONG_MS && is_worth_it).then_some((start_ms, end_ms))
}

/// Writes the stretch `kept` of the sound in `from` to `to`, losslessly as
/// FLAC, its tags with it.
pub(super) fn cut(from: &Path, to: &Path, (start_ms, end_ms): (u64, u64)) -> Result<(), String> {
  let seconds = |ms: u64| format!("{}.{:03}", ms / 1_000, ms % 1_000);
  let status = screenwide_sidecar::ffmpeg_command()
    .args([
      "-hide_banner",
      "-nostdin",
      "-loglevel",
      "error",
      "-y",
      "-ss",
    ])
    .arg(seconds(start_ms))
    .arg("-i")
    .arg(from)
    .arg("-t")
    .arg(seconds(end_ms - start_ms))
    .args(["-map", "0:a:0", "-vn", "-c:a", "flac", "-f", "flac"])
    .arg(to)
    .status()
    .map_err(|error| format!("Could not start FFmpeg: {error}"))?;
  status
    .success()
    .then_some(())
    .ok_or_else(|| "The sound could not be trimmed".to_owned())
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn a_sound_effect_loses_its_dead_space_and_keeps_its_edges() {
    // Heard from 1.2 s to 2 s of 6 s.
    assert_eq!(kept(6_000, Some((1_200, 2_000))), Some((1_180, 2_150)));
  }

  #[test]
  fn next_to_nothing_to_cut_or_a_song_is_kept_whole() {
    assert_eq!(kept(2_000, Some((30, 1_900))), None);
    assert_eq!(kept(60_000, Some((5_000, 50_000))), None);
    assert_eq!(kept(2_000, None), None);
  }
}
