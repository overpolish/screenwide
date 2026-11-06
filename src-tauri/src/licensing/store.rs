// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: Elastic-2.0

use std::path::Path;

use screenwide_licensing::LicenseState;

/// The saved state, or none when there is no file or it cannot be read.
pub(super) fn read(path: &Path) -> Option<LicenseState> {
  let contents = std::fs::read(path).ok()?;
  serde_json::from_slice(&contents).ok()
}

/// Writes beside the file and then moves it into place, so a write cut short
/// leaves the last whole state rather than half of one: losing the file would
/// lose the keys added.
pub(super) fn write(path: &Path, state: &LicenseState) -> Result<(), String> {
  if let Some(parent) = path.parent() {
    std::fs::create_dir_all(parent).map_err(|error| error.to_string())?;
  }
  let contents = serde_json::to_vec_pretty(state).map_err(|error| error.to_string())?;
  let partial = path.with_extension("json.partial");
  std::fs::write(&partial, contents).map_err(|error| error.to_string())?;
  std::fs::rename(&partial, path).map_err(|error| error.to_string())
}
