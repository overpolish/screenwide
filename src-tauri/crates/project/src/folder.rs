// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Making and unmaking project folders. Which folder new projects go in is
//! the app's, from its settings.

use std::io::ErrorKind;
use std::path::{Path, PathBuf};

use super::EXTENSION;

const MEDIA_FOLDER: &str = "media";
const PREVIEW_FILE: &str = "preview.png";
/// Frames along the edit side by side, for the browser's card to scrub.
const SCRUB_STRIP_FILE: &str = "scrub.jpg";
/// Each frame's width in the strip: twice the widest card, so it stays sharp
/// on a Retina display. How many frames a strip holds follows from it.
pub const SCRUB_FRAME_WIDTH: u32 = 480;
/// Left in a project whose recording was discarded. Capture teardown runs off
/// the thread that discards, so the app can quit before the folder is gone;
/// the next launch finishes the job instead of leaving a project nobody kept.
const CANCELLED_MARKER: &str = ".cancelled";

/// A project folder made for a capture that is about to write into it.
pub struct NewProject {
  /// The manifest, named after the folder.
  pub file: PathBuf,
  /// Where the capture writes its tracks.
  pub media: PathBuf,
  pub root: PathBuf,
}

impl NewProject {
  pub fn mark_cancelled(&self) -> Result<(), String> {
    std::fs::write(self.root.join(CANCELLED_MARKER), []).map_err(|error| error.to_string())
  }

  /// Deletes the folder and everything in it, for a capture that produced
  /// nothing worth keeping.
  pub fn remove(&self) {
    let _ = std::fs::remove_dir_all(&self.root);
  }
}

/// Makes the folder with an exclusive create, so two captures started in the
/// same second can never share one.
pub fn create_in(directory: &Path, title: &str) -> Result<NewProject, String> {
  std::fs::create_dir_all(directory)
    .map_err(|error| format!("The projects folder could not be created: {error}"))?;
  let (root, name) = reserve(directory, title)
    .map_err(|error| format!("The project folder could not be created: {error}"))?;
  let media = root.join(MEDIA_FOLDER);
  if let Err(error) = std::fs::create_dir(&media) {
    let _ = std::fs::remove_dir_all(&root);
    return Err(format!("The project folder could not be created: {error}"));
  }
  Ok(NewProject {
    file: root.join(format!("{name}.{EXTENSION}")),
    media,
    root,
  })
}

/// Makes an empty folder in `directory` named `title`, adding " (2)", " (3)"
/// and so on as file managers do while that name is taken. Returns the
/// folder and the name it took.
pub(super) fn reserve(directory: &Path, title: &str) -> std::io::Result<(PathBuf, String)> {
  let mut suffix = 1_u32;
  loop {
    let name = if suffix == 1 {
      title.to_owned()
    } else {
      format!("{title} ({suffix})")
    };
    let root = directory.join(&name);
    match std::fs::create_dir(&root) {
      Ok(()) => return Ok((root, name)),
      Err(error) if error.kind() == ErrorKind::AlreadyExists => suffix += 1,
      Err(error) => return Err(error),
    }
  }
}

/// Renames the project whose manifest is `file`: its folder and its manifest
/// both take `title`. Media is named relative to the folder, so nothing inside
/// changes. Returns the renamed manifest.
pub fn rename(file: &Path, title: &str) -> Result<PathBuf, String> {
  let root = file
    .parent()
    .ok_or_else(|| "The project has no folder".to_owned())?;
  let new_root = root
    .parent()
    .ok_or_else(|| "The project's folder has no parent".to_owned())?
    .join(title);
  // On a case-insensitive disk a change of case names the same folder.
  let same_folder = std::fs::canonicalize(&new_root)
    .is_ok_and(|existing| std::fs::canonicalize(root).is_ok_and(|root| root == existing));
  if new_root.exists() && !same_folder {
    return Err(format!("There is already a project called {title} there"));
  }
  let _turn = super::manifest::hold_writes();
  rename_settling(root, &new_root)
    .map_err(|error| format!("The project could not be renamed: {error}"))?;
  let moved = new_root.join(
    file
      .file_name()
      .ok_or_else(|| "The project has no file name".to_owned())?,
  );
  let renamed = new_root.join(format!("{title}.{EXTENSION}"));
  if moved != renamed {
    if let Err(error) = rename_settling(&moved, &renamed) {
      // Put the folder back rather than leave a project named two ways.
      let _ = rename_settling(&new_root, root);
      return Err(format!("The project could not be renamed: {error}"));
    }
  }
  Ok(renamed)
}

/// Windows refuses to rename a folder while anything inside it is open, and a
/// reader can outlast the editor letting go of a project by a moment: a
/// decoder winding down, or a virus scanner looking at what just changed.
/// Those let go on their own, so the rename is tried again for a little while.
pub(super) fn rename_settling(from: &Path, to: &Path) -> std::io::Result<()> {
  #[cfg(target_os = "windows")]
  {
    const ATTEMPTS: u32 = 40;
    const ERROR_SHARING_VIOLATION: i32 = 32;
    let mut attempt = 1;
    loop {
      match std::fs::rename(from, to) {
        Err(error)
          if attempt < ATTEMPTS
            && (error.kind() == ErrorKind::PermissionDenied
              || error.raw_os_error() == Some(ERROR_SHARING_VIOLATION)) =>
        {
          std::thread::sleep(std::time::Duration::from_millis(50));
          attempt += 1;
        }
        result => return result,
      }
    }
  }
  #[cfg(not(target_os = "windows"))]
  std::fs::rename(from, to)
}

/// Moves the whole project whose manifest is `file` to the Trash, or the
/// Recycle Bin, where it can still be put back.
pub fn move_to_trash(file: &Path) -> Result<(), String> {
  let root = file
    .parent()
    .ok_or_else(|| "The project has no folder".to_owned())?;
  trash::delete(root).map_err(|error| {
    let bin = if cfg!(target_os = "windows") {
      "Recycle Bin"
    } else {
      "Trash"
    };
    format!("The project could not be moved to the {bin}: {error}")
  })
}

/// Whether the project whose manifest is `file` was discarded while it was
/// being recorded.
pub fn is_cancelled(file: &Path) -> bool {
  file
    .parent()
    .is_some_and(|root| root.join(CANCELLED_MARKER).exists())
}

/// The picture the browser shows for the project whose manifest is `file`.
/// Kept beside the manifest, so a project shows it wherever it is copied.
pub fn preview_path(file: &Path) -> PathBuf {
  file.parent().unwrap_or(file).join(PREVIEW_FILE)
}

/// Kept beside the preview, so it travels with the project as the preview
/// does.
pub fn scrub_strip_path(file: &Path) -> PathBuf {
  file.parent().unwrap_or(file).join(SCRUB_STRIP_FILE)
}

/// Finishes deleting projects in `directory` whose recording was discarded
/// just before the app last quit. Only that folder is looked at, one level
/// deep.
pub fn sweep_cancelled(directory: &Path) {
  let Ok(entries) = std::fs::read_dir(directory) else {
    return;
  };
  for entry in entries.flatten() {
    let root = entry.path();
    if root.join(CANCELLED_MARKER).is_file() {
      let _ = std::fs::remove_dir_all(root);
    }
  }
}
