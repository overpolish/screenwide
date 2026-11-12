// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Studio ducking's switch, as the project keeps it for each microphone: on,
//! the system audio and the songs come down while the voice speaks, their
//! singing out of its way entirely; off, they play as they are.

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::choice_file;

/// The make of the choice file.
const FORMAT_VERSION: u16 = 1;

/// Whether the system audio and the songs make way for the voice.
#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize, TS)]
#[serde(rename_all = "kebab-case")]
#[ts(export)]
pub enum StudioDucking {
  #[default]
  Off,
  On,
}

fn choice_path(project_folder: &Path, microphone: usize) -> PathBuf {
  project_folder.join(format!("studio-ducking-{microphone}.json"))
}

/// What the project says for the `microphone`th track. A recording starts
/// with Studio ducking on, until someone turns it off; it is heard only once
/// its model is downloaded.
pub fn choice(project_folder: &Path, microphone: usize) -> StudioDucking {
  choice_file::read(&choice_path(project_folder, microphone), FORMAT_VERSION)
    .unwrap_or(StudioDucking::On)
}

pub fn keep(project_folder: &Path, microphone: usize, choice: StudioDucking) -> Result<(), String> {
  choice_file::write(
    &choice_path(project_folder, microphone),
    FORMAT_VERSION,
    choice,
  )
  .map_err(|error| format!("Could not keep the Studio ducking choice: {error}"))
}
