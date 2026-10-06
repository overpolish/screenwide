// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Stored pictures carried by a project. A project keeps a copy of each
//! picture it shows under the store's own name for it, so on another
//! computer the copy is handed back to the store and the name finds it.

use std::path::{Path, PathBuf};

use super::{cache, content_name, directory, is_name, keep, KEPT_EXTENSIONS};

/// The file the stored picture `name` is kept in, if there is one.
pub(crate) fn stored_file(name: &str) -> Option<PathBuf> {
  let folder = directory()?;
  KEPT_EXTENSIONS
    .iter()
    .map(|extension| folder.join(format!("{name}.{extension}")))
    .find(|path| path.is_file())
}

/// Hands the store `file`, a picture a project carries as `<name>.<ext>`,
/// where the store has none of that name. Any other file is left alone.
pub(crate) fn adopt(file: &Path) -> Result<(), String> {
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
  Ok(())
}

/// A name for `file` drawn from its contents, keeping its extension, so the
/// same picture chosen twice is kept once.
pub(crate) fn content_file_name(file: &Path) -> Result<String, String> {
  let bytes = std::fs::read(file).map_err(|error| error.to_string())?;
  let name = content_name(&bytes);
  Ok(
    match file.extension().and_then(|extension| extension.to_str()) {
      Some(extension) => format!("{name}.{}", extension.to_ascii_lowercase()),
      None => name,
    },
  )
}
