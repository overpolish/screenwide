// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Stored pictures carried by a project. A project keeps a copy of each
//! picture it shows under the store's own name for it, and the record kept
//! beside it where there is one, which says who made a picture found on
//! Openverse, so on another computer the copy is handed back to the store,
//! the name finds it, and its credit comes with it.

use std::path::{Path, PathBuf};

use screenwide_credits::Credit;

use super::library::{kept_path, read_kept};
use super::{cache, content_name, directory, is_name, keep, KEPT_EXTENSIONS};

/// The file the stored picture `name` is kept in, if there is one.
pub fn stored_file(name: &str) -> Option<PathBuf> {
  let folder = directory()?;
  KEPT_EXTENSIONS
    .iter()
    .map(|extension| folder.join(format!("{name}.{extension}")))
    .find(|path| path.is_file())
}

/// Hands the store `file`, a picture a project carries as `<name>.<ext>`,
/// where the store has none of that name. Any other file is left alone.
pub fn adopt(file: &Path) -> Result<(), String> {
  let (Some(name), Some(extension)) = (
    file.file_stem().and_then(|stem| stem.to_str()),
    file.extension().and_then(|extension| extension.to_str()),
  ) else {
    return Ok(());
  };
  if !is_name(name) || !KEPT_EXTENSIONS.contains(&extension) || stored_file(name).is_some() {
    return Ok(());
  }
  let bytes = std::fs::read(file).map_err(|error| error.to_string())?;
  keep(name, extension, &bytes)?;
  cache::forget_failure(name);
  // Its credit, where the project keeps one, which the store has none of.
  if let (Some(folder), Some(from)) = (directory(), file.parent()) {
    let record = kept_path(from, name);
    if record.is_file() && !kept_path(folder, name).is_file() {
      let _ = std::fs::copy(record, kept_path(folder, name));
    }
  }
  Ok(())
}

/// The record the store keeps beside the picture `name`, for a project to
/// carry beside its copy, where there is one.
pub fn record_file(name: &str) -> Option<PathBuf> {
  let path = kept_path(directory()?, name);
  path.is_file().then_some(path)
}

/// Who made the picture `name`, from the record the store or else the
/// project folder `carried` keeps beside it, where it was found on
/// Openverse.
pub fn credit_in(carried: Option<&Path>, name: &str) -> Option<Credit> {
  if !is_name(name) {
    return None;
  }
  directory()
    .and_then(|folder| read_kept(folder, name))
    .or_else(|| read_kept(carried?, name))?
    .credit
}

/// A name for `file` drawn from its contents, keeping its extension, so the
/// same picture chosen twice is kept once.
pub fn content_file_name(file: &Path) -> Result<String, String> {
  let bytes = std::fs::read(file).map_err(|error| error.to_string())?;
  let name = content_name(&bytes);
  Ok(
    match file.extension().and_then(|extension| extension.to_str()) {
      Some(extension) => format!("{name}.{}", extension.to_ascii_lowercase()),
      None => name,
    },
  )
}
