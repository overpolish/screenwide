// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The pictures taken out of the library on this computer. A project
//! carries its own copy of each picture it shows and hands the store those
//! it lacks when it is opened, since images draw from the store alone; one
//! listed here is still handed back, so the project draws it, but stays out
//! of the library. It is let back in when it is added again.

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
    eprintln!("Could not note the pictures taken out of the library: {error}");
  }
}

/// The pictures taken out of the library in `folder`.
pub(super) fn names(folder: &Path) -> BTreeSet<String> {
  read(folder)
}

/// Notes that the picture `name` was taken out of the library in `folder`.
pub(super) fn note(folder: &Path, name: &str) {
  let mut names = read(folder);
  if names.insert(name.to_owned()) {
    write(folder, &names);
  }
}

/// Lets the picture `name` back into the library in `folder`, added again.
pub(super) fn forget(folder: &Path, name: &str) {
  let mut names = read(folder);
  if names.remove(name) {
    write(folder, &names);
  }
}
