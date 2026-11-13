// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The sounds taken out of the library on this computer. A project carries
//! its own copy of each sound it plays and hands the library those it lacks
//! when it is opened, so a sound taken out would come back with the next
//! project that plays it; one listed here is not handed back. It is let
//! back in when it is added again.

use std::collections::BTreeSet;
use std::path::Path;

const FILE: &str = "removed.json";

fn read(folder: &Path) -> BTreeSet<String> {
  std::fs::read(folder.join(FILE))
    .ok()
    .and_then(|bytes| serde_json::from_slice(&bytes).ok())
    .unwrap_or_default()
}

fn write(folder: &Path, names: &BTreeSet<String>) {
  let written = serde_json::to_vec(names)
    .map_err(|error| error.to_string())
    .and_then(|bytes| std::fs::write(folder.join(FILE), bytes).map_err(|error| error.to_string()));
  if let Err(error) = written {
    eprintln!("Could not note the sounds taken out of the library: {error}");
  }
}

/// Whether the sound `name` was taken out of the library in `folder`.
pub(super) fn was_removed(folder: &Path, name: &str) -> bool {
  read(folder).contains(name)
}

/// Notes that the sound `name` was taken out of the library in `folder`.
pub(super) fn note(folder: &Path, name: &str) {
  let mut names = read(folder);
  if names.insert(name.to_owned()) {
    write(folder, &names);
  }
}

/// Lets the sound `name` back into the library in `folder`, added again.
pub(super) fn forget(folder: &Path, name: &str) {
  let mut names = read(folder);
  if names.remove(name) {
    write(folder, &names);
  }
}
