// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Which file each microphone track is heard from. Reduce noise and Vocal
//! cleanup each make files of their own from the track, and together the
//! two switches pick one: the recording itself, the track cleaned of noise,
//! or the cleaned-up file made from either. A switch whose file is not made
//! yet is heard as off.

use std::path::{Path, PathBuf};

use super::noise::{self, NoiseReduction};
use super::voice::{self, VocalCleanup};

#[cfg(test)]
mod tests;

/// What has been done to a track as it is heard.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) struct Processing {
  pub noise: bool,
  pub voice: bool,
}

impl Processing {
  const ALL: [Self; 3] = [
    Self {
      noise: true,
      voice: false,
    },
    Self {
      noise: false,
      voice: true,
    },
    Self {
      noise: true,
      voice: true,
    },
  ];
}

/// The `stream`th track's file for `processing`, if it is made and current.
fn made(project_folder: &Path, stream: usize, processing: Processing) -> Option<PathBuf> {
  let noise_made = || noise::is_made(project_folder, stream);
  let made = match processing {
    Processing {
      noise: false,
      voice: false,
    } => false,
    Processing {
      noise: true,
      voice: false,
    } => noise_made(),
    Processing { noise, voice: true } => {
      voice::is_made(project_folder, stream, noise) && (!noise || noise_made())
    }
  };
  made.then(|| match processing {
    Processing { voice: false, .. } => noise::cleaned_path(project_folder, stream),
    Processing { noise, voice: true } => voice::cleaned_path(project_folder, stream, noise),
  })
}

/// Every file made of the `streams` of the project in `project_folder`,
/// whether or not it is heard.
pub(crate) fn files(project_folder: &Path, streams: &[usize]) -> Vec<(usize, Processing, PathBuf)> {
  streams
    .iter()
    .flat_map(|&stream| {
      Processing::ALL.into_iter().filter_map(move |processing| {
        made(project_folder, stream, processing).map(|path| (stream, processing, path))
      })
    })
    .collect()
}

/// How the `stream`th track is heard: as its switches say, as far as their
/// files are made.
fn processing(project_folder: &Path, stream: usize) -> Processing {
  let noise = noise::choice(project_folder, stream) == NoiseReduction::On
    && noise::is_made(project_folder, stream);
  let voice = voice::choice(project_folder, stream) == VocalCleanup::On
    && voice::is_made(project_folder, stream, noise);
  Processing { noise, voice }
}

/// How each of the `streams` is heard, leaving out those heard as recorded.
pub(crate) fn heard(project_folder: &Path, streams: &[usize]) -> Vec<(usize, Processing)> {
  streams
    .iter()
    .map(|&stream| (stream, processing(project_folder, stream)))
    .filter(|(_, processing)| *processing != Processing::default())
    .collect()
}

/// The file each of the `streams` is heard from in place of the recording,
/// for those that have one.
pub(crate) fn heard_files(project_folder: &Path, streams: &[usize]) -> Vec<(usize, PathBuf)> {
  heard(project_folder, streams)
    .into_iter()
    .filter_map(|(stream, processing)| {
      made(project_folder, stream, processing).map(|path| (stream, path))
    })
    .collect()
}

/// The file the `stream`th track is heard from now in place of the recording,
/// if it has one.
pub(crate) fn heard_file(project_folder: &Path, stream: usize) -> Option<PathBuf> {
  made(project_folder, stream, processing(project_folder, stream))
}
