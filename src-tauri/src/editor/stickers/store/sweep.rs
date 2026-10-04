// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Clearing out the stored pictures nothing needs any more.
//!
//! The only edits that outlive the app are the unsaved recordings kept for
//! recovery, each beside its recording as a `.timeline-edit-` file. Their
//! text is searched for picture names rather than read as an edit, so a
//! picture is kept by any edit that could still name it, whichever version
//! of the format wrote it.

use std::path::Path;

use super::{is_name, IMAGE_PREFIX, KEPT_EXTENSIONS, NAME_DIGITS};

/// Every stored picture named in the edits kept in `recordings`.
fn named_in(recordings: &Path) -> Vec<String> {
  let Ok(entries) = std::fs::read_dir(recordings) else {
    return Vec::new();
  };
  let mut names = Vec::new();
  for entry in entries.flatten() {
    let path = entry.path();
    let is_edit = path
      .file_name()
      .and_then(|name| name.to_str())
      .is_some_and(|name| name.contains(".timeline-edit-") && name.ends_with(".json"));
    let Some(text) = is_edit
      .then(|| std::fs::read_to_string(&path).ok())
      .flatten()
    else {
      continue;
    };
    for (at, _) in text.match_indices(IMAGE_PREFIX) {
      let start = at + IMAGE_PREFIX.len();
      if let Some(name) = text
        .get(start..start + NAME_DIGITS)
        .filter(|name| is_name(name))
      {
        names.push(name.to_owned());
      }
    }
  }
  names
}

/// Removes from `folder` every picture no edit in `recordings` names, and
/// anything a write cut short left behind.
pub(super) fn sweep(folder: &Path, recordings: Option<&Path>) {
  let Ok(entries) = std::fs::read_dir(folder) else {
    return;
  };
  let kept = recordings.map(named_in).unwrap_or_default();
  for entry in entries.flatten() {
    let path = entry.path();
    let Some(file) = path.file_name().and_then(|name| name.to_str()) else {
      continue;
    };
    // A kept picture is its name and one of the kept formats, nothing more:
    // a write cut short leaves `.tmp` behind it, which goes.
    let named = file.split_once('.').is_some_and(|(name, extension)| {
      KEPT_EXTENSIONS.contains(&extension) && kept.iter().any(|kept| kept == name)
    });
    if !named && path.is_file() {
      let _ = std::fs::remove_file(&path);
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use std::path::PathBuf;

  fn folder(name: &str) -> PathBuf {
    let folder = std::env::temp_dir()
      .join("screenwide-tests")
      .join("sticker-store")
      .join(name);
    let _ = std::fs::remove_dir_all(&folder);
    std::fs::create_dir_all(&folder).unwrap();
    folder
  }

  #[test]
  fn a_launch_keeps_only_the_pictures_an_unsaved_edit_names() {
    let recordings = folder("recordings");
    let store = folder("store");
    let kept = "0123456789abcdef0123456789abcdef";
    let dropped = "fedcba9876543210fedcba9876543210";
    std::fs::write(
      recordings.join("recording-1.timeline-edit-a.json"),
      format!(r#"{{"asset":"image:{kept}"}}"#),
    )
    .unwrap();
    // A name in any other file keeps nothing.
    std::fs::write(
      recordings.join("recording-1.cursor.jsonl"),
      format!("image:{dropped}"),
    )
    .unwrap();
    for file in [
      format!("{kept}.png"),
      format!("{kept}.gif"),
      format!("{dropped}.png"),
      format!("{dropped}.webp"),
      format!("{kept}.png.tmp"),
    ] {
      std::fs::write(store.join(file), b"x").unwrap();
    }
    sweep(&store, Some(&recordings));
    let mut left: Vec<_> = std::fs::read_dir(&store)
      .unwrap()
      .map(|entry| entry.unwrap().file_name().into_string().unwrap())
      .collect();
    left.sort();
    assert_eq!(left, vec![format!("{kept}.gif"), format!("{kept}.png")]);
  }
}
