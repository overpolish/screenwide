// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The measures Auto volume keeps beside the project: one for each file a
//! track is heard from, with what the file was when it was measured, so a
//! file made again is measured again.

use std::path::{Path, PathBuf};
use std::time::UNIX_EPOCH;

use serde::{Deserialize, Serialize};

use super::Leveling;

/// Bumped when a measure of an older make should be taken again.
const MEASURE_VERSION: u16 = 2;

/// One file's measure, and what the file was when it was taken.
#[derive(Clone, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
struct Measure {
  /// The file's name in the project, or nothing for the recording, which
  /// never changes once made.
  file: Option<String>,
  length: u64,
  modified_ms: u64,
  /// Nothing for a file with no voice in it.
  leveling: Option<Leveling>,
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct KeptMeasures {
  version: u16,
  measures: Vec<Measure>,
}

fn path(project_folder: &Path, stream: usize) -> PathBuf {
  project_folder.join(format!("levels-{stream}.json"))
}

fn kept(project_folder: &Path, stream: usize) -> Vec<Measure> {
  std::fs::read(path(project_folder, stream))
    .ok()
    .and_then(|bytes| serde_json::from_slice::<KeptMeasures>(&bytes).ok())
    .filter(|kept| kept.version == MEASURE_VERSION)
    .map(|kept| kept.measures)
    .unwrap_or_default()
}

/// The file `heard` as its measure names it: the recording when nothing.
fn identity(heard: Option<&Path>) -> Measure {
  let Some(path) = heard else {
    return Measure {
      file: None,
      length: 0,
      modified_ms: 0,
      leveling: None,
    };
  };
  let metadata = std::fs::metadata(path).ok();
  Measure {
    file: path
      .file_name()
      .map(|name| name.to_string_lossy().into_owned()),
    length: metadata.as_ref().map_or(0, std::fs::Metadata::len),
    modified_ms: metadata
      .and_then(|metadata| metadata.modified().ok())
      .and_then(|time| time.duration_since(UNIX_EPOCH).ok())
      .map_or(0, |since| since.as_millis() as u64),
    leveling: None,
  }
}

/// The leveling of the `stream`th track heard from `heard`, or from the
/// recording when nothing, if it has been measured as it is now. The inner
/// nothing is a file with no voice to level.
pub(super) fn measured(
  project_folder: &Path,
  stream: usize,
  heard: Option<&Path>,
) -> Option<Option<Leveling>> {
  let wanted = identity(heard);
  kept(project_folder, stream)
    .into_iter()
    .find(|measure| {
      (&measure.file, measure.length, measure.modified_ms)
        == (&wanted.file, wanted.length, wanted.modified_ms)
    })
    .map(|measure| measure.leveling)
}

/// Keeps `leveling` as the measure of the `stream`th track heard from
/// `heard`, or from the recording when nothing, in place of any before it.
pub(super) fn keep(
  project_folder: &Path,
  stream: usize,
  heard: Option<&Path>,
  leveling: Option<Leveling>,
) -> Result<(), String> {
  let taken = Measure {
    leveling,
    ..identity(heard)
  };
  let mut measures = kept(project_folder, stream);
  measures.retain(|measure| measure.file != taken.file);
  measures.push(taken);
  let kept = KeptMeasures {
    version: MEASURE_VERSION,
    measures,
  };
  let bytes = serde_json::to_vec(&kept).map_err(|error| error.to_string())?;
  std::fs::write(path(project_folder, stream), bytes)
    .map_err(|error| format!("Could not keep the microphone's volume: {error}"))
}
