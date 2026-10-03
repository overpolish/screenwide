// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The pictures on the desktop right now, offered as backgrounds.
//!
//! Only the wallpapers in use are offered. The rest of a system's catalogue
//! is not reliably on disk: macOS downloads most of its pictures only when
//! someone picks one, so a scan of its folders offers tiles nothing can draw.

use std::path::PathBuf;

use serde::Serialize;

#[cfg(target_os = "macos")]
mod macos;
#[cfg(target_os = "windows")]
mod windows;

/// What every desktop tile is called. A second display with a different
/// picture is "Desktop 2", so the names stay unique.
const NAME: &str = "Desktop";

/// One picture on the desktop, offered as a background.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct SystemWallpaper {
  pub name: String,
  pub path: String,
}

#[cfg(target_os = "macos")]
fn active_pictures() -> Vec<PathBuf> {
  macos::active_pictures()
}

#[cfg(target_os = "windows")]
fn active_pictures() -> Vec<PathBuf> {
  windows::active_pictures()
}

#[cfg(not(any(target_os = "macos", target_os = "windows")))]
fn active_pictures() -> Vec<PathBuf> {
  Vec::new()
}

/// The pictures as tiles: one per distinct file, in the order the system
/// lists them, so displays sharing a wallpaper share a tile.
fn tiles(pictures: Vec<PathBuf>) -> Vec<SystemWallpaper> {
  let mut seen: Vec<PathBuf> = Vec::new();
  for picture in pictures {
    if !seen.contains(&picture) {
      seen.push(picture);
    }
  }
  seen
    .into_iter()
    .enumerate()
    .map(|(index, picture)| SystemWallpaper {
      name: if index == 0 {
        NAME.to_owned()
      } else {
        format!("{NAME} {}", index + 1)
      },
      path: picture.to_string_lossy().into_owned(),
    })
    .collect()
}

/// The desktop's pictures, read off the main thread: Windows answers through
/// COM, and neither system promises its answer is quick.
#[tauri::command]
pub async fn list_system_wallpapers() -> Vec<SystemWallpaper> {
  tauri::async_runtime::spawn_blocking(|| tiles(active_pictures()))
    .await
    .unwrap_or_default()
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn displays_sharing_a_picture_share_a_tile() {
    let found = tiles(vec![
      PathBuf::from("/a.png"),
      PathBuf::from("/b.png"),
      PathBuf::from("/a.png"),
    ]);
    assert_eq!(
      found,
      vec![
        SystemWallpaper {
          name: "Desktop".to_owned(),
          path: "/a.png".to_owned(),
        },
        SystemWallpaper {
          name: "Desktop 2".to_owned(),
          path: "/b.png".to_owned(),
        },
      ]
    );
  }

  /// Whatever the machine has on its desktop has to come back as files the
  /// renderer can open.
  #[test]
  fn the_desktop_pictures_are_files() {
    for picture in active_pictures() {
      assert!(picture.is_file(), "{} is not a file", picture.display());
    }
  }
}
