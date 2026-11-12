// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Auto volume for the system audio: brought to sit a set distance under the
//! voice as it is heard, whatever the app or the game recorded played it
//! at, and its peaks held below -1 dB, always, as the songs and sound effects
//! laid under the recording are. Studio ducking takes it further down while
//! the voice speaks. The system audio is measured once, as recorded, and
//! the same lift or cut is played over whichever file it is heard from, so
//! its ducked file is not lifted back up.

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use super::{choice, heard, loudness, measured, AutoVolume, Leveling};
use super::{ABOVE_LOUDNESS_DB, SILENT_LUFS, TARGET_LUFS};
use crate::choice_file;
use crate::heard::Processing;

/// How far under the voice the system audio plays, in dB.
const SYSTEM_UNDER_VOICE_DB: f64 = 6.0;
/// The most a quiet system audio is lifted by, in dB.
const MOST_LIFT_DB: f64 = 20.0;
/// The make of the measure file.
const FORMAT_VERSION: u16 = 1;

/// The system audio as measured, and the microphone it plays under.
#[derive(Clone, Copy, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
struct Measure {
  microphone: usize,
  /// None where it is silent.
  loudness_lufs: Option<f64>,
}

fn path(project_folder: &Path, system: usize) -> PathBuf {
  project_folder.join(format!("system-level-{system}.json"))
}

fn kept(project_folder: &Path, system: usize) -> Option<Measure> {
  choice_file::read(&path(project_folder, system), FORMAT_VERSION)
}

/// How loud the `microphone`th track is heard, in LUFS: where Auto volume is
/// on, the loudness it brings voices to; otherwise as recorded, where its
/// heard file has been measured, and else that loudness, as a fair guess.
pub fn voice_lufs(project_folder: &Path, microphone: usize) -> f64 {
  if choice(project_folder, microphone) == AutoVolume::On {
    return TARGET_LUFS;
  }
  let heard = heard::heard_file(project_folder, microphone);
  measured(project_folder, microphone, heard.as_deref())
    .flatten()
    .map_or(TARGET_LUFS, |leveling| {
      leveling.threshold_db - ABOVE_LOUDNESS_DB
    })
}

/// Whether the `system`th track still has to be measured to play under the
/// `microphone`th.
pub fn is_system_unmeasured(project_folder: &Path, system: usize, microphone: usize) -> bool {
  kept(project_folder, system).is_none_or(|measure| measure.microphone != microphone)
}

/// Measures the `system`th track of `movie`, as recorded, to play under the
/// `microphone`th, unless it has been already, telling `progress` how far it
/// has got.
pub fn measure_system(
  project_folder: &Path,
  (system, microphone): (usize, usize),
  movie: &Path,
  duration_ms: u64,
  progress: &mut dyn FnMut(f32),
) -> Result<(), String> {
  if !is_system_unmeasured(project_folder, system, microphone) {
    return Ok(());
  }
  let loudness = loudness::integrated(movie, system, None, duration_ms, progress)?;
  choice_file::write(
    &path(project_folder, system),
    FORMAT_VERSION,
    Measure {
      microphone,
      loudness_lufs: (loudness > SILENT_LUFS).then_some(loudness),
    },
  )
  .map_err(|error| format!("Could not keep the system audio's level: {error}"))
}

/// How the `stream`th track is levelled under the voice, where it is system
/// audio that has been measured: a lift or cut, with peaks held, and no
/// compression worth the name, as a threshold at full scale touches nothing
/// below it.
pub(super) fn system_leveling(project_folder: &Path, stream: usize) -> Option<Leveling> {
  let measure = kept(project_folder, stream)?;
  let loudness = measure.loudness_lufs?;
  let target = voice_lufs(project_folder, measure.microphone) - SYSTEM_UNDER_VOICE_DB;
  Some(Leveling {
    threshold_db: 0.0,
    makeup_db: (target - loudness).min(MOST_LIFT_DB),
  })
}

/// The files the system audio is heard from, each levelled the same: as
/// recorded, and made to make way for the voice.
pub(super) const SYSTEM_FILES: [Processing; 2] = [
  Processing {
    noise: false,
    voice: false,
    duck: false,
    studio: false,
  },
  Processing {
    noise: false,
    voice: false,
    duck: true,
    studio: false,
  },
];

#[cfg(test)]
mod tests {
  use super::super::{heard_filters, levelings};
  use super::*;

  #[test]
  fn system_audio_sits_six_decibels_under_the_voice_from_any_file() {
    let folder =
      std::env::temp_dir().join(format!("screenwide-under-voice-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&folder);
    std::fs::create_dir_all(&folder).unwrap();
    // System audio, stream 0, measured at -30 LUFS under microphone 1,
    // which Auto volume brings to -16 LUFS: lifted 8 dB to sit at -22.
    choice_file::write(
      &path(&folder, 0),
      FORMAT_VERSION,
      Measure {
        microphone: 1,
        loudness_lufs: Some(-30.0),
      },
    )
    .unwrap();
    let levelled = levelings(&folder, 0);
    assert_eq!(levelled.len(), 2);
    assert!(levelled
      .iter()
      .all(|(_, leveling)| (leveling.makeup_db - 8.0).abs() < 1e-9));
    assert_eq!(heard_filters(&folder, &[0, 1]).len(), 1);
    // A deafening one is cut as far as it takes; a near-silent one lifted
    // 20 dB at most.
    for (loudness, makeup) in [(-2.0, -20.0), (-70.0, 20.0)] {
      choice_file::write(
        &path(&folder, 0),
        FORMAT_VERSION,
        Measure {
          microphone: 1,
          loudness_lufs: Some(loudness),
        },
      )
      .unwrap();
      assert_eq!(system_leveling(&folder, 0).unwrap().makeup_db, makeup);
    }
    let _ = std::fs::remove_dir_all(&folder);
  }
}
