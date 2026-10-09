// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! A microphone switch's choice, kept beside the project with the make of
//! the files the switch plays, so it holds when the project opens again
//! and the export reads the same answer as the preview.

use std::path::Path;

use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct Kept<T> {
  version: u16,
  choice: T,
}

/// The choice kept at `path`, if it was kept for files of make `version`.
pub(super) fn read<T: DeserializeOwned>(path: &Path, version: u16) -> Option<T> {
  std::fs::read(path)
    .ok()
    .and_then(|bytes| serde_json::from_slice::<Kept<T>>(&bytes).ok())
    .filter(|kept| kept.version == version)
    .map(|kept| kept.choice)
}

pub(super) fn write<T: Serialize>(path: &Path, version: u16, choice: T) -> Result<(), String> {
  let bytes = serde_json::to_vec(&Kept { version, choice }).map_err(|error| error.to_string())?;
  std::fs::write(path, bytes).map_err(|error| error.to_string())
}
