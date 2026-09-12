// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The wallpapers the system draws rather than ships.
//!
//! Sequoia, Tahoe, the aerials and the older families are extensions now:
//! nothing under `/System/Library/Desktop Pictures` holds their pixels. What
//! does hold them is the cache the system writes when a person sets one:
//! `/private/var/db/Wallpapers/<UUID>/Wallpaper.png`, the picture at the
//! display's own size, beside a `Metadata.plist` naming the extension that
//! drew it. Only the wallpapers somebody has actually set are in there, which
//! is exactly the set worth offering: they are the ones that machine has.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

use super::SystemWallpaper;

/// Where the system keeps the pictures it has drawn.
pub(super) const DIRECTORY: &str = "/private/var/db/Wallpapers";

/// The two files a folder in there holds: the picture, and what drew it.
const PICTURE: &str = "Wallpaper.png";
const METADATA: &str = "Metadata.plist";

/// The name a plural family of wallpapers is offered under. `aerials` is the
/// one the system ships; a wallpaper is one picture, so the tile is singular.
const SINGULAR: [(&str, &str); 1] = [("aerials", "aerial")];

/// The extension that drew the picture, as the last part of a provider such
/// as `com.apple.wallpaper.choice.sequoia`.
///
/// The plist is binary, and only the first choice is read: a folder holds one
/// rendered picture, so the choice that named it is the first one.
fn provider(metadata: &Path) -> Option<String> {
  let file = plist::Value::from_file(metadata).ok()?;
  let choice = file
    .as_dictionary()?
    .get("Content")?
    .as_dictionary()?
    .get("Choices")?
    .as_array()?
    .first()?;
  let provider = choice.as_dictionary()?.get("Provider")?.as_string()?;
  let segment = provider.rsplit('.').next()?;
  (!segment.is_empty()).then(|| segment.to_ascii_lowercase())
}

/// A provider's last part as a wallpaper's name: `sequoia` is "Sequoia".
fn titled(segment: &str) -> Option<String> {
  let base = SINGULAR
    .iter()
    .find_map(|(plural, one)| (*plural == segment).then_some(*one))
    .unwrap_or(segment);
  let mut letters = base.chars();
  let first = letters.next()?;
  Some(first.to_uppercase().collect::<String>() + letters.as_str())
}

/// When the folder was last written, which is when its wallpaper was set.
/// A folder whose time cannot be read sorts first rather than being dropped.
fn modified(folder: &Path) -> SystemTime {
  std::fs::metadata(folder)
    .and_then(|data| data.modified())
    .unwrap_or(SystemTime::UNIX_EPOCH)
}

/// Every picture the system has drawn, oldest first.
///
/// Two folders can name the same extension, a dark Sequoia beside a light
/// one, and they are told apart by the order they were written in: the first
/// is "Sequoia" and the second "Sequoia 2", so a name always means the same
/// tile for as long as the folders are there.
pub(super) fn collect(directory: &Path) -> Vec<SystemWallpaper> {
  let Ok(entries) = std::fs::read_dir(directory) else {
    return Vec::new();
  };
  let mut folders: Vec<(SystemTime, PathBuf)> = entries
    .flatten()
    .map(|entry| entry.path())
    .filter(|path| path.join(PICTURE).is_file())
    .map(|path| (modified(&path), path))
    .collect();
  folders.sort();
  let mut seen: HashMap<String, usize> = HashMap::new();
  folders
    .iter()
    .filter_map(|(_, folder)| {
      let name = titled(&provider(&folder.join(METADATA))?)?;
      let count = seen.entry(name.clone()).or_insert(0);
      *count += 1;
      let name = if *count > 1 {
        format!("{name} {count}")
      } else {
        name
      };
      Some(SystemWallpaper {
        path: folder.join(PICTURE).to_string_lossy().into_owned(),
        thumbnail_path: None,
        name,
      })
    })
    .collect()
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn names_a_provider_by_its_last_part() {
    assert_eq!(titled("sequoia").as_deref(), Some("Sequoia"));
    assert_eq!(titled("tahoe").as_deref(), Some("Tahoe"));
    assert_eq!(titled("macintosh").as_deref(), Some("Macintosh"));
    assert_eq!(titled("aerials").as_deref(), Some("Aerial"));
    assert_eq!(titled(""), None);
  }

  /// Every folder the system has drawn a picture into has to come back as a
  /// tile that points at a file, whatever the machine happens to hold.
  #[test]
  fn the_drawn_pictures_are_offered() {
    let found = collect(Path::new(DIRECTORY));
    let Ok(entries) = std::fs::read_dir(DIRECTORY) else {
      return;
    };
    let drawn = entries
      .flatten()
      .filter(|entry| entry.path().join(PICTURE).is_file())
      .count();
    assert!(found.len() <= drawn);
    assert_eq!(found.is_empty(), drawn == 0);
    let mut names: Vec<&str> = found.iter().map(|one| one.name.as_str()).collect();
    let offered = names.len();
    names.sort_unstable();
    names.dedup();
    assert_eq!(names.len(), offered, "two tiles share a name");
    for wallpaper in &found {
      assert!(Path::new(&wallpaper.path).is_file());
      assert!(!wallpaper.name.is_empty());
      assert_eq!(wallpaper.thumbnail_path, None);
    }
  }
}
