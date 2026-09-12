// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The wallpapers the recent systems keep as folders rather than as files.
//!
//! Everything since Sonoma lives under `.wallpapers/<Name>/`, and what is in
//! there is not always a picture: `Sonoma Horizon` holds a HEIC and two
//! thumbnails, `Tahoe Day` holds one movie, `Sonoma` holds four, one per
//! appearance and orientation. One folder is one wallpaper, named by the
//! folder, and the asset is picked the same way every time so the same
//! machine always offers the same tile.

use std::path::{Path, PathBuf};

use super::paths::{files, is_image, is_movie, stem};
use super::SystemWallpaper;

/// The folder the modern wallpapers sit in, beside the older flat pictures.
pub(super) const DIRECTORY: &str = ".wallpapers";

/// Of the several movies a folder may hold, the landscape one is the one a
/// canvas wants: the others are the portrait and the dark variants.
const PREFERRED: &str = "Landscape";

/// The system's own small copy where the folder carries one. The retina copy
/// is preferred: a tile is drawn at a couple of device pixels per point.
fn thumbnail(folder: &Path, name: &str) -> Option<String> {
  ["@2x", ""].iter().find_map(|suffix| {
    let candidate = folder.join(format!("{name} Thumbnail{suffix}.png"));
    candidate
      .is_file()
      .then(|| candidate.to_string_lossy().into_owned())
  })
}

/// The movie the folder is shown as: the one named after the folder, else the
/// landscape one, else the first in name order.
fn chosen_movie(files: &[PathBuf], name: &str) -> Option<PathBuf> {
  let movies: Vec<&PathBuf> = files.iter().filter(|path| is_movie(path)).collect();
  let named = movies
    .iter()
    .find(|path| stem(path).as_deref() == Some(name));
  let landscape = || {
    movies
      .iter()
      .find(|path| stem(path).is_some_and(|found| found.contains(PREFERRED)))
  };
  named
    .or_else(landscape)
    .or_else(|| movies.first())
    .map(|path| (*path).clone())
}

/// One folder as one wallpaper.
///
/// A folder whose only asset is a movie needs a still copied out of it, which
/// needs somewhere to keep it: without that folder, and when the frame cannot
/// be read, the wallpaper is left out rather than offered as a tile nothing
/// can draw.
fn wallpaper(folder: &Path, stills: Option<&Path>) -> Option<SystemWallpaper> {
  let name = folder.file_name()?.to_str()?.to_owned();
  let files = files(folder);
  if let Some(picture) = files
    .iter()
    .find(|path| is_image(path) && stem(path).as_deref() == Some(name.as_str()))
  {
    return Some(SystemWallpaper {
      path: picture.to_string_lossy().into_owned(),
      thumbnail_path: thumbnail(folder, &name),
      name,
    });
  }
  let movie = chosen_movie(&files, &name)?;
  let still = crate::screenshots::video_still_macos::still_for(&movie, stills?, &name)?;
  let path = still.to_string_lossy().into_owned();
  Some(SystemWallpaper {
    thumbnail_path: Some(path.clone()),
    path,
    name,
  })
}

/// Every folder under `.wallpapers`, as one wallpaper each.
pub(super) fn collect(directory: &Path, stills: Option<&Path>) -> Vec<SystemWallpaper> {
  let Ok(entries) = std::fs::read_dir(directory) else {
    return Vec::new();
  };
  let mut folders: Vec<PathBuf> = entries
    .flatten()
    .map(|entry| entry.path())
    .filter(|path| path.is_dir())
    .collect();
  folders.sort();
  folders
    .iter()
    .filter_map(|folder| wallpaper(folder, stills))
    .collect()
}

#[cfg(test)]
mod tests {
  use super::*;

  fn paths(names: &[&str]) -> Vec<PathBuf> {
    names.iter().map(PathBuf::from).collect()
  }

  /// A throwaway folder for the frames copied out of the wallpaper movies, so
  /// a test never writes into the running app's cache.
  fn stills() -> PathBuf {
    let directory = std::env::temp_dir().join("screenwide-wallpaper-stills-test");
    let _ = std::fs::create_dir_all(&directory);
    directory
  }

  /// The recent systems keep their wallpapers as folders under `.wallpapers`,
  /// and the older flat pictures are not where they live. Each folder there
  /// has to come back as one tile, named by the folder, whether it holds a
  /// picture or only a movie.
  #[test]
  fn the_modern_wallpaper_folders_are_offered() {
    let directory = Path::new(super::super::platform::DIRECTORY).join(DIRECTORY);
    let Ok(entries) = std::fs::read_dir(&directory) else {
      return;
    };
    let folders: Vec<String> = entries
      .flatten()
      .filter(|entry| entry.path().is_dir())
      .filter_map(|entry| entry.file_name().to_str().map(str::to_owned))
      .collect();
    if folders.is_empty() {
      return;
    }
    let found = super::super::system_wallpapers(Some(&stills()));
    for folder in folders {
      let offered = found
        .iter()
        .find(|one| one.name == folder)
        .unwrap_or_else(|| panic!("{folder} is not offered as a wallpaper"));
      assert!(
        Path::new(&offered.path).is_file(),
        "{folder} points at {} which is not a file",
        offered.path
      );
    }
  }

  #[test]
  fn prefers_the_movie_named_after_the_folder() {
    let files = paths(&[
      "/w/Tahoe Day/Tahoe Day Landscape.mov",
      "/w/Tahoe Day/Tahoe Day.mov",
    ]);
    assert_eq!(
      chosen_movie(&files, "Tahoe Day"),
      Some(PathBuf::from("/w/Tahoe Day/Tahoe Day.mov"))
    );
  }

  #[test]
  fn falls_back_to_the_landscape_movie() {
    let files = paths(&[
      "/w/Sonoma/Sonoma Graphic Dark Portrait.mov",
      "/w/Sonoma/Sonoma Graphic Light Landscape.mov",
    ]);
    assert_eq!(
      chosen_movie(&files, "Sonoma"),
      Some(PathBuf::from(
        "/w/Sonoma/Sonoma Graphic Light Landscape.mov"
      ))
    );
  }

  #[test]
  fn falls_back_to_the_first_movie_in_name_order() {
    let files = paths(&["/w/Sonoma/A Portrait.mov", "/w/Sonoma/B Portrait.mov"]);
    assert_eq!(
      chosen_movie(&files, "Sonoma"),
      Some(PathBuf::from("/w/Sonoma/A Portrait.mov"))
    );
  }

  #[test]
  fn answers_nothing_for_a_folder_without_a_movie() {
    assert_eq!(
      chosen_movie(&paths(&["/w/Sonoma/notes.txt"]), "Sonoma"),
      None
    );
  }
}
