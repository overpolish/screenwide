// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Which file each track is heard from. Studio sound, Reduce noise and Vocal
//! cleanup each make files of their own from the microphone, and the
//! switches pick one: the recording itself, the track cleaned of noise, the
//! cleaned-up file made from either, or, while Studio sound is on, its
//! rebuilt voice in place of them all. Auto volume makes the system audio
//! make way for the voice (`duck`), heard while the microphone is on too. A
//! switch whose file is not made yet is heard as off.

use std::path::{Path, PathBuf};

use super::duck;
use super::noise::{self, NoiseReduction};
use super::studio;
use super::voice::{self, VocalCleanup};

#[cfg(test)]
mod tests;

/// What has been done to a track as it is heard.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) struct Processing {
  pub noise: bool,
  pub voice: bool,
  /// Made to make way for the voice; only ever of the system audio.
  pub duck: bool,
  /// Rebuilt by Studio sound; never together with noise or voice.
  pub studio: bool,
}

impl Processing {
  const ALL: [Self; 5] = [
    Self {
      noise: true,
      ..Self::NONE
    },
    Self {
      voice: true,
      ..Self::NONE
    },
    Self {
      noise: true,
      voice: true,
      ..Self::NONE
    },
    Self {
      duck: true,
      ..Self::NONE
    },
    Self {
      studio: true,
      ..Self::NONE
    },
  ];
  const NONE: Self = Self {
    noise: false,
    voice: false,
    duck: false,
    studio: false,
  };
}

/// The `stream`th track's file for `processing`, if it is made and current.
fn made(project_folder: &Path, stream: usize, processing: Processing) -> Option<PathBuf> {
  if processing.duck {
    return duck::is_made(project_folder, stream).then(|| duck::path(project_folder, stream));
  }
  if processing.studio {
    return studio::is_made(project_folder, stream).then(|| studio::path(project_folder, stream));
  }
  let noise_made = || noise::is_made(project_folder, stream);
  let made = match processing {
    Processing {
      noise: false,
      voice: false,
      ..
    } => false,
    Processing {
      noise: true,
      voice: false,
      ..
    } => noise_made(),
    Processing {
      noise, voice: true, ..
    } => voice::is_made(project_folder, stream, noise) && (!noise || noise_made()),
  };
  made.then(|| match processing {
    Processing { voice: false, .. } => noise::cleaned_path(project_folder, stream),
    Processing { noise, .. } => voice::cleaned_path(project_folder, stream, noise),
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

/// How the `stream`th track is heard, among the `streams` that are on: as
/// its switches say, as far as their files are made.
fn processing(project_folder: &Path, stream: usize, streams: &[usize]) -> Processing {
  let duck = duck::is_heard(project_folder, stream, streams);
  if studio::is_heard(project_folder, stream) {
    return Processing {
      studio: true,
      duck,
      ..Processing::NONE
    };
  }
  let noise = noise::choice(project_folder, stream) == NoiseReduction::On
    && noise::is_made(project_folder, stream);
  let voice = voice::choice(project_folder, stream) == VocalCleanup::On
    && voice::is_made(project_folder, stream, noise);
  Processing {
    noise,
    voice,
    duck,
    studio: false,
  }
}

/// How each of the `streams`, the tracks that are on, is heard, leaving out
/// those heard as recorded.
pub(crate) fn heard(project_folder: &Path, streams: &[usize]) -> Vec<(usize, Processing)> {
  streams
    .iter()
    .map(|&stream| (stream, processing(project_folder, stream, streams)))
    .filter(|(_, processing)| *processing != Processing::default())
    .collect()
}

/// The file each of the `streams`, the tracks that are on, is heard from in
/// place of the recording, for those that have one.
pub(crate) fn heard_files(project_folder: &Path, streams: &[usize]) -> Vec<(usize, PathBuf)> {
  heard(project_folder, streams)
    .into_iter()
    .filter_map(|(stream, processing)| {
      made(project_folder, stream, processing).map(|path| (stream, path))
    })
    .collect()
}

/// The file the microphone, the `stream`th track, is heard from now in place
/// of the recording, if it has one.
pub(crate) fn heard_file(project_folder: &Path, stream: usize) -> Option<PathBuf> {
  made(
    project_folder,
    stream,
    processing(project_folder, stream, &[stream]),
  )
}
