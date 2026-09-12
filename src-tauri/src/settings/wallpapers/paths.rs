// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! What a wallpaper's filename says about it.
//!
//! Every platform's listing asks the same few questions of a path, so they
//! are asked in one place rather than once per backend.

use std::path::{Path, PathBuf};

/// The extensions a wallpaper folder is read for. HEIC leads because that is
/// what the system stores its own pictures as.
pub(super) const IMAGE_EXTENSIONS: [&str; 4] = ["heic", "jpg", "jpeg", "png"];

/// A movie is what the systems since Sonoma ship their wallpapers as.
const MOVIE_EXTENSION: &str = "mov";

pub(super) fn extension(path: &Path) -> Option<String> {
  Some(path.extension()?.to_str()?.to_ascii_lowercase())
}

pub(super) fn is_image(path: &Path) -> bool {
  extension(path).is_some_and(|found| IMAGE_EXTENSIONS.contains(&found.as_str()))
}

pub(super) fn is_movie(path: &Path) -> bool {
  extension(path).as_deref() == Some(MOVIE_EXTENSION)
}

pub(super) fn stem(path: &Path) -> Option<String> {
  Some(path.file_stem()?.to_str()?.to_owned())
}

/// The files in `folder`, in name order, so a folder always answers the same
/// way. Folders inside it are not descended into: none of the system's are
/// nested, and a wallpaper is one asset rather than a tree.
pub(super) fn files(folder: &Path) -> Vec<PathBuf> {
  let Ok(entries) = std::fs::read_dir(folder) else {
    return Vec::new();
  };
  let mut found: Vec<PathBuf> = entries
    .flatten()
    .map(|entry| entry.path())
    .filter(|path| path.is_file())
    .collect();
  found.sort();
  found
}
