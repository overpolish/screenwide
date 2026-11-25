// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use std::collections::HashSet;
use std::path::{Path, PathBuf};

/// The files in `directory` named after one of `stems`: the file itself, a
/// numbered one beside it, its camera and its subtitles.
pub(super) fn named_after(directory: &Path, stems: &[String]) -> HashSet<PathBuf> {
  let Ok(entries) = std::fs::read_dir(directory) else {
    return HashSet::new();
  };
  entries
    .flatten()
    .map(|entry| entry.path())
    .filter(|path| {
      let Some(name) = path.file_name().and_then(|name| name.to_str()) else {
        return false;
      };
      stems.iter().any(|stem| {
        name
          .strip_prefix(stem.as_str())
          .is_some_and(|rest| rest.starts_with(['.', ' ']))
      })
    })
    .collect()
}
