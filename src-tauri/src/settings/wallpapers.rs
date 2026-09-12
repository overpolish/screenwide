// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The pictures the operating system already ships as desktop backgrounds.
//!
//! The app used to bundle eight wallpapers of its own making. Every machine
//! already carries a folder of them, chosen by the people who designed the
//! system the app runs on, so the picker offers those instead and the bundle
//! stays the size of the app.

use std::path::{Path, PathBuf};

use serde::Serialize;
use tauri::{AppHandle, Manager};

#[cfg(target_os = "macos")]
mod cached_macos;
#[cfg(target_os = "macos")]
mod dynamic_macos;
mod paths;

/// One picture the system offers as a desktop background.
///
/// The thumbnail is the system's own small copy where there is one: a desktop
/// picture is tens of megabytes, and a swatch is a few dozen pixels across.
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SystemWallpaper {
  pub name: String,
  pub path: String,
  pub thumbnail_path: Option<String>,
}

/// Where the stills copied out of the wallpaper movies are kept, under the
/// app's own cache folder: one PNG per wallpaper, written once.
const STILLS: &str = "wallpaper-stills";

/// The pictures the system offers, in name order.
///
/// `stills` is where a wallpaper that is only a movie may keep the frame
/// copied out of it. Without it those wallpapers are simply not offered.
///
/// A folder that is not there, or cannot be read, answers with nothing rather
/// than with an error: a picker without system wallpapers is still a picker.
pub(crate) fn system_wallpapers(stills: Option<&Path>) -> Vec<SystemWallpaper> {
  let mut found = collect(stills);
  found.sort_by(|left, right| left.name.cmp(&right.name));
  found.dedup_by(|left, right| left.name == right.name);
  found
}

/// The pictures the system offers, read off the main thread.
///
/// Reading the folders is cheap, but a wallpaper the system ships as a movie
/// needs a frame copied out of a file that can be a couple of hundred
/// megabytes, so the whole listing goes to a blocking task.
#[tauri::command]
pub async fn list_system_wallpapers(app: AppHandle) -> Vec<SystemWallpaper> {
  let stills: Option<PathBuf> = app
    .path()
    .app_cache_dir()
    .ok()
    .map(|directory| directory.join(STILLS));
  tauri::async_runtime::spawn_blocking(move || system_wallpapers(stills.as_deref()))
    .await
    .unwrap_or_default()
}

#[cfg(target_os = "macos")]
mod platform {
  use super::paths::{is_image, stem, IMAGE_EXTENSIONS};
  use super::SystemWallpaper;
  use std::path::{Path, PathBuf};

  pub(super) const DIRECTORY: &str = "/System/Library/Desktop Pictures";
  /// The folder holds one swatch-sized copy per picture, named by the same
  /// stem as the picture itself, extension and all kept as the system wrote
  /// it. On this machine every entry is HEIC.
  const THUMBNAILS: &str = ".thumbnails";
  /// The flat tones are a colour picker's job, not a wallpaper's.
  const SKIPPED: &str = "Solid Colors";
  /// A dynamic desktop is a bundle rather than a file. Where one carries a
  /// still picture the picker offers that; where it carries nothing, as the
  /// recent systems' bundles do, there is nothing to offer.
  const BUNDLE_EXTENSION: &str = "madesktop";

  /// The first still picture inside a dynamic desktop bundle, in name order
  /// so the same bundle always answers with the same picture.
  fn still_in_bundle(bundle: &Path) -> Option<PathBuf> {
    let Ok(entries) = std::fs::read_dir(bundle) else {
      return None;
    };
    let mut stills: Vec<PathBuf> = entries
      .flatten()
      .map(|entry| entry.path())
      .filter(|path| path.is_file() && is_image(path))
      .collect();
    stills.sort();
    stills.into_iter().next()
  }

  fn thumbnail(thumbnails: &Path, name: &str) -> Option<String> {
    IMAGE_EXTENSIONS.iter().find_map(|extension| {
      let candidate = thumbnails.join(format!("{name}.{extension}"));
      candidate
        .is_file()
        .then(|| candidate.to_string_lossy().into_owned())
    })
  }

  pub(super) fn collect(stills: Option<&Path>) -> Vec<SystemWallpaper> {
    let directory = Path::new(DIRECTORY);
    let thumbnails = directory.join(THUMBNAILS);
    let mut found =
      super::dynamic_macos::collect(&directory.join(super::dynamic_macos::DIRECTORY), stills);
    let Ok(entries) = std::fs::read_dir(directory) else {
      found.extend(super::cached_macos::collect(Path::new(
        super::cached_macos::DIRECTORY,
      )));
      return found;
    };
    let flat = entries
      .flatten()
      .filter_map(|entry| {
        let path = entry.path();
        if path.file_name().is_some_and(|name| name == SKIPPED) {
          return None;
        }
        let picture = if super::paths::extension(&path).as_deref() == Some(BUNDLE_EXTENSION) {
          still_in_bundle(&path)?
        } else if path.is_file() && is_image(&path) {
          path.clone()
        } else {
          return None;
        };
        let name = stem(&path)?;
        Some(SystemWallpaper {
          path: picture.to_string_lossy().into_owned(),
          thumbnail_path: thumbnail(&thumbnails, &name),
          name,
        })
      })
      .collect::<Vec<_>>();
    found.extend(flat);
    // The pictures the system has drawn for itself come last, so a name they
    // share with a wallpaper that ships as a file belongs to the file.
    found.extend(super::cached_macos::collect(Path::new(
      super::cached_macos::DIRECTORY,
    )));
    found
  }
}

#[cfg(target_os = "windows")]
mod platform {
  use super::paths::{extension, stem};
  use super::SystemWallpaper;
  use std::path::{Path, PathBuf};

  /// The two folders Windows keeps its own pictures in: the wallpapers sit in
  /// themed subfolders, the lock screen pictures sit flat.
  const WALLPAPER: &str = r"Web\Wallpaper";
  const SCREEN: &str = r"Web\Screen";

  fn windows_directory() -> PathBuf {
    std::env::var_os("WINDIR").map_or_else(|| PathBuf::from(r"C:\Windows"), PathBuf::from)
  }

  /// Windows writes its own pictures as JPEG and PNG, so HEIC is left out
  /// rather than offered as a tile the shader cannot read.
  fn is_picture(path: &Path) -> bool {
    matches!(extension(path).as_deref(), Some("jpg" | "jpeg" | "png"))
  }

  fn visit(directory: &Path, recurse: bool, found: &mut Vec<SystemWallpaper>) {
    let Ok(entries) = std::fs::read_dir(directory) else {
      return;
    };
    for entry in entries.flatten() {
      let path = entry.path();
      if path.is_dir() {
        if recurse {
          visit(&path, true, found);
        }
        continue;
      }
      if !is_picture(&path) {
        continue;
      }
      let Some(name) = stem(&path) else {
        continue;
      };
      found.push(SystemWallpaper {
        name,
        path: path.to_string_lossy().into_owned(),
        thumbnail_path: None,
      });
    }
  }

  pub(super) fn collect(_stills: Option<&Path>) -> Vec<SystemWallpaper> {
    let root = windows_directory();
    let mut found = Vec::new();
    visit(&root.join(WALLPAPER), true, &mut found);
    visit(&root.join(SCREEN), false, &mut found);
    found
  }
}

#[cfg(target_os = "macos")]
fn collect(stills: Option<&Path>) -> Vec<SystemWallpaper> {
  platform::collect(stills)
}

#[cfg(target_os = "windows")]
fn collect(stills: Option<&Path>) -> Vec<SystemWallpaper> {
  platform::collect(stills)
}

/// Nothing is promised about where another system keeps its wallpapers, so
/// the picker simply offers none there.
#[cfg(not(any(target_os = "macos", target_os = "windows")))]
fn collect(_stills: Option<&Path>) -> Vec<SystemWallpaper> {
  Vec::new()
}

#[cfg(test)]
mod tests {
  use super::*;

  /// A throwaway folder for the frames copied out of the wallpaper movies,
  /// so a test never writes into the running app's cache.
  fn stills() -> PathBuf {
    let directory = std::env::temp_dir().join("screenwide-wallpaper-stills-test");
    let _ = std::fs::create_dir_all(&directory);
    directory
  }

  #[test]
  fn enumerating_does_not_panic() {
    let _ = system_wallpapers(None);
  }

  #[test]
  fn names_are_sorted_and_unique() {
    let found = system_wallpapers(Some(&stills()));
    let mut names: Vec<&str> = found.iter().map(|one| one.name.as_str()).collect();
    let sorted = names.clone();
    names.dedup();
    assert_eq!(names.len(), found.len());
    let mut ordered = sorted.clone();
    ordered.sort_unstable();
    assert_eq!(sorted, ordered);
  }

  #[cfg(target_os = "macos")]
  #[test]
  fn the_system_folder_yields_wallpapers() {
    let found = system_wallpapers(Some(&stills()));
    assert!(
      !found.is_empty(),
      "no wallpapers found in {}",
      platform::DIRECTORY
    );
    for wallpaper in &found {
      assert!(Path::new(&wallpaper.path).is_file());
      assert!(!wallpaper.name.is_empty());
    }
  }
}
