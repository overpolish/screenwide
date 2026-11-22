// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The small file kept beside a picture that describes it: how it is named,
//! read back, and written so that writers landing together keep their own.

use std::path::Path;
use std::sync::atomic::{AtomicU64, Ordering};

use serde::{Deserialize, Serialize};

use super::Picture;

/// The version of the file kept beside a picture.
const FORMAT_VERSION: u16 = 1;

#[derive(Deserialize, Serialize)]
struct Kept {
  version: u16,
  picture: Picture,
}

pub(in super::super) fn kept_path(folder: &Path, name: &str) -> std::path::PathBuf {
  folder.join(format!("{name}.json"))
}

pub(in super::super) fn read_kept(folder: &Path, name: &str) -> Option<Picture> {
  let bytes = std::fs::read(kept_path(folder, name)).ok()?;
  let kept: Kept = serde_json::from_slice(&bytes).ok()?;
  (kept.version == FORMAT_VERSION && kept.picture.name == name).then_some(kept.picture)
}

/// Keeps `picture`'s description, in place of one kept already when
/// `replace` says so, or else only where none is.
pub(super) fn write_kept(folder: &Path, picture: &Picture, replace: bool) -> Result<(), String> {
  let kept = Kept {
    version: FORMAT_VERSION,
    picture: picture.clone(),
  };
  let bytes = serde_json::to_vec(&kept).map_err(|error| error.to_string())?;
  let path = kept_path(folder, &picture.name);
  // Each write has a partial file of its own: listing describes a picture
  // the moment its file lands, which can be while it is being noted or
  // renamed, and two writers sharing one would rename it from under each
  // other.
  static WRITES: AtomicU64 = AtomicU64::new(0);
  let write = WRITES.fetch_add(1, Ordering::Relaxed);
  let partial = folder.join(format!("{}.json.{write}.tmp", picture.name));
  let written = std::fs::write(&partial, bytes).and_then(|()| {
    if replace {
      return std::fs::rename(&partial, &path);
    }
    // A link fails where a description is kept already, which stands.
    let linked = match std::fs::hard_link(&partial, &path) {
      Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => Ok(()),
      linked => linked,
    };
    let _ = std::fs::remove_file(&partial);
    linked
  });
  written.map_err(|error| {
    let _ = std::fs::remove_file(&partial);
    format!("The picture could not be described: {error}")
  })
}
