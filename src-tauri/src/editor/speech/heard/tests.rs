// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use std::path::{Path, PathBuf};

use super::super::noise::{self, NoiseReduction};
use super::super::voice::{self, VocalCleanup};
use super::{files, heard_files, Processing};

/// A project folder of its own for `name`, empty.
fn project(name: &str) -> PathBuf {
  let folder = std::env::temp_dir().join(format!("screenwide-heard-{name}-{}", std::process::id()));
  let _ = std::fs::remove_dir_all(&folder);
  std::fs::create_dir_all(&folder).unwrap();
  folder
}

fn make(path: PathBuf) -> PathBuf {
  std::fs::write(&path, b"flac").unwrap();
  path
}

/// Keeps the noise switch at `noise_on` and the voice switch at `voice_on`.
fn switched(folder: &Path, noise_on: bool, voice_on: bool) {
  let choice = |on| {
    if on {
      NoiseReduction::On
    } else {
      NoiseReduction::Off
    }
  };
  noise::keep(folder, 1, choice(noise_on)).unwrap();
  let cleanup = if voice_on {
    VocalCleanup::On
  } else {
    VocalCleanup::Off
  };
  voice::keep(folder, 1, cleanup).unwrap();
}

#[test]
fn each_pair_of_switches_plays_its_own_file() {
  let folder = project("pairs");
  // Kept first, as turning a switch on keeps the make before the file.
  switched(&folder, false, false);
  let noise_file = make(noise::cleaned_path(&folder, 1));
  let voice_file = make(voice::cleaned_path(&folder, 1, false));
  let both_file = make(voice::cleaned_path(&folder, 1, true));
  assert_eq!(heard_files(&folder, &[0, 1]), []);
  switched(&folder, true, false);
  assert_eq!(heard_files(&folder, &[0, 1]), [(1, noise_file)]);
  switched(&folder, false, true);
  assert_eq!(heard_files(&folder, &[0, 1]), [(1, voice_file)]);
  switched(&folder, true, true);
  assert_eq!(heard_files(&folder, &[0, 1]), [(1, both_file)]);
  assert_eq!(files(&folder, &[1]).len(), 3);
  let _ = std::fs::remove_dir_all(&folder);
}

#[test]
fn a_switch_whose_file_is_not_made_is_heard_as_off() {
  let folder = project("missing");
  switched(&folder, true, true);
  let noise_file = make(noise::cleaned_path(&folder, 1));
  // The cleaned-up voice is still being made from the noise-reduced track.
  assert_eq!(heard_files(&folder, &[1]), [(1, noise_file.clone())]);
  let voice_only = Processing {
    noise: false,
    voice: true,
  };
  make(voice::cleaned_path(&folder, 1, false));
  // A voice file made from the track as recorded does not stand in for one
  // made from the noise-reduced track.
  assert!(files(&folder, &[1])
    .iter()
    .any(|(_, processing, _)| *processing == voice_only));
  assert_eq!(heard_files(&folder, &[1]), [(1, noise_file)]);
  let _ = std::fs::remove_dir_all(&folder);
}

#[test]
fn files_from_an_older_cleanup_are_cleared_and_the_switch_starts_off() {
  let folder = project("older");
  make(voice::cleaned_path(&folder, 1, false));
  make(voice::cleaned_path(&folder, 1, true));
  std::fs::write(
    folder.join("voice-1.json"),
    br#"{"version":0,"choice":"on"}"#,
  )
  .unwrap();
  assert_eq!(voice::choice(&folder, 1), VocalCleanup::Off);
  voice::keep(&folder, 1, VocalCleanup::Off).unwrap();
  assert!(!voice::cleaned_path(&folder, 1, false).exists());
  assert!(!voice::cleaned_path(&folder, 1, true).exists());
  let _ = std::fs::remove_dir_all(&folder);
}
