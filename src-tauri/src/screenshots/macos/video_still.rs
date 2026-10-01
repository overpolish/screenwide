// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! A still off the front of a wallpaper movie.
//!
//! The system's recent desktop pictures are movies: the folder holds a `.mov`
//! and, for some of them, nothing else. A picker cannot show one and the
//! canvas cannot paint one, so the opening frame is copied out once and kept
//! as a PNG, which every part of the pipeline already reads.

use std::ffi::{c_char, CString};
use std::path::{Path, PathBuf};

unsafe extern "C" {
  fn screenwide_video_still_png(video_path: *const c_char, destination_path: *const c_char) -> i32;
}

/// The still for `video`, written into `directory` under `name`.
///
/// A file already there is answered straight away: the frame does not change,
/// and the extraction is the one slow step in listing the wallpapers. `None`
/// when the movie cannot be read or the file cannot be written, which the
/// caller answers by leaving that wallpaper out rather than by failing.
pub(crate) fn still_for(video: &Path, directory: &Path, name: &str) -> Option<PathBuf> {
  let destination = directory.join(format!("{name}.png"));
  if destination.is_file() {
    return Some(destination);
  }
  std::fs::create_dir_all(directory).ok()?;
  let source = CString::new(video.to_str()?).ok()?;
  let target = CString::new(destination.to_str()?).ok()?;
  // SAFETY: both paths outlive the call, and the native side only reads them
  // and writes the file they name.
  let written = unsafe { screenwide_video_still_png(source.as_ptr(), target.as_ptr()) };
  (written != 0 && destination.is_file()).then_some(destination)
}

#[cfg(test)]
mod tests {
  use super::*;

  /// The one movie every recent system carries. This is the proof that the
  /// still comes out as a picture rather than as an empty file.
  const WALLPAPER: &str = "/System/Library/Desktop Pictures/.wallpapers/Tahoe Day/Tahoe Day.mov";

  #[test]
  fn copies_a_frame_out_of_a_wallpaper_movie() {
    let video = Path::new(WALLPAPER);
    if !video.is_file() {
      return;
    }
    let directory = std::env::temp_dir().join("screenwide-wallpaper-still-test");
    let _ = std::fs::remove_dir_all(&directory);
    let still = still_for(video, &directory, "Tahoe Day").expect("a still came out of the movie");
    let picture = image::open(&still).expect("the still is a readable picture");
    assert!(picture.width() > 0 && picture.height() > 0);
    assert!(picture.width() <= 1024 && picture.height() <= 1024);
    // The same call again answers the file that is already there.
    assert_eq!(
      still_for(video, &directory, "Tahoe Day").as_deref(),
      Some(still.as_path())
    );
    let _ = std::fs::remove_dir_all(&directory);
  }

  #[test]
  fn answers_nothing_for_a_movie_that_is_not_there() {
    let directory = std::env::temp_dir().join("screenwide-wallpaper-still-missing");
    assert!(still_for(Path::new("/no/such/wallpaper.mov"), &directory, "Missing").is_none());
    let _ = std::fs::remove_dir_all(&directory);
  }
}
