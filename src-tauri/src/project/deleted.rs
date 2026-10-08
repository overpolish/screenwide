// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Recently Deleted: where a deleted project waits, for 30 days, before it
//! goes to the system Trash.
//!
//! The Trash cannot be listed or emptied back out on every platform without
//! permissions the app has no business asking for, and in use it is full of
//! everything else. So a deleted project is set aside in a hidden folder
//! beside where it was, on the same drive, which makes deleting and
//! restoring a rename, never a copy. The library keeps where each came from.

use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use tauri::AppHandle;

use super::library::{self, DeletedProject};
use super::relocate::{folder_name, name_manifest, rename_into};

/// Hidden on macOS by its dot, and on Windows by its attribute.
pub(crate) const DELETED_FOLDER: &str = ".Screenwide Deleted";
/// How long a deleted project waits before it goes to the Trash.
pub(crate) const KEEP_MS: u64 = 30 * 24 * 60 * 60 * 1_000;

fn now_ms() -> u64 {
  SystemTime::now()
    .duration_since(UNIX_EPOCH)
    .map_or(0, |since| {
      u64::try_from(since.as_millis()).unwrap_or(u64::MAX)
    })
}

/// Sets the project whose manifest is `file` aside in the deleted folder
/// beside it and keeps where it came from, and returns its manifest there.
/// It leaves Recent, to come back there when restored.
pub(crate) fn delete(app: &AppHandle, file: &Path) -> Result<PathBuf, String> {
  let set_aside = set_aside(file)?;
  library::change(app, |library| {
    library.recents.retain(|recent| recent != file);
    library.deleted.push(DeletedProject {
      deleted_ms: now_ms(),
      file: set_aside.clone(),
      original: file.to_path_buf(),
    });
  })?;
  Ok(set_aside)
}

/// Puts the deleted project whose manifest is now `file` back in the folder
/// it came from, under its own name or, while that is taken, the next free
/// one. Returns its manifest there.
pub(crate) fn restore(app: &AppHandle, file: &Path) -> Result<PathBuf, String> {
  let Some(deleted) = library::load(app)
    .deleted
    .into_iter()
    .find(|deleted| deleted.file == file)
  else {
    return Err("That project is no longer in Recently Deleted".to_owned());
  };
  let restored = put_back(&deleted.file, &deleted.original)?;
  library::change(app, |library| {
    library.deleted.retain(|entry| entry.file != file);
    library.remember(&restored);
  })?;
  Ok(restored)
}

/// The projects in Recently Deleted that are there to show, newest first.
/// One whose drive is not connected is kept but not listed; one removed
/// from the deleted folder by hand is forgotten.
pub(crate) fn listed(app: &AppHandle) -> Vec<DeletedProject> {
  let library = library::load(app);
  let (present, missing): (Vec<_>, Vec<_>) = library
    .deleted
    .into_iter()
    .partition(|deleted| deleted.file.is_file());
  let gone: Vec<PathBuf> = missing
    .into_iter()
    .filter(|deleted| deleted_folder_of(&deleted.file).is_some_and(|folder| folder.is_dir()))
    .map(|deleted| deleted.file)
    .collect();
  if !gone.is_empty() {
    if let Err(error) = library::change(app, |library| {
      library
        .deleted
        .retain(|deleted| !gone.contains(&deleted.file));
    }) {
      eprintln!("Could not forget projects removed from Recently Deleted: {error}");
    }
  }
  let mut present = present;
  present.reverse();
  present
}

/// Sends every project that has waited [`KEEP_MS`] in Recently Deleted on
/// to the Trash.
pub(crate) fn expire(app: &AppHandle) {
  let due = now_ms().saturating_sub(KEEP_MS);
  for deleted in listed(app) {
    if deleted.deleted_ms <= due {
      if let Err(error) = library::trash(app, &deleted.file) {
        eprintln!("Could not move an expired project to the Trash: {error}");
      }
    }
  }
}

/// The deleted folder a set-aside manifest is in.
fn deleted_folder_of(file: &Path) -> Option<&Path> {
  file.parent()?.parent()
}

/// Renames the project folder of `file` into the deleted folder beside it,
/// under a name no other deleted project there has, and returns its
/// manifest there. The manifest keeps its own name, so the project is
/// listed by the name it had even when its folder takes " (2)".
pub(super) fn set_aside(file: &Path) -> Result<PathBuf, String> {
  let root = file
    .parent()
    .ok_or_else(|| "The project has no folder".to_owned())?;
  let folder = root
    .parent()
    .ok_or_else(|| "The project's folder has no parent".to_owned())?
    .join(DELETED_FOLDER);
  std::fs::create_dir_all(&folder)
    .map_err(|error| format!("The project could not be deleted: {error}"))?;
  hide(&folder);
  let title = folder_name(root)?;
  let (destination, _) = rename_into(root, &folder, &title)
    .map_err(|error| format!("{title} could not be deleted: {error}"))?;
  Ok(
    destination.join(
      file
        .file_name()
        .ok_or_else(|| "The project has no file name".to_owned())?,
    ),
  )
}

/// Renames the set-aside project of `file` back into the folder `original`
/// was in, under `original`'s name or the next free one.
pub(super) fn put_back(file: &Path, original: &Path) -> Result<PathBuf, String> {
  let root = file
    .parent()
    .ok_or_else(|| "The project has no folder".to_owned())?;
  let original_root = original
    .parent()
    .ok_or_else(|| "The project's original folder is unknown".to_owned())?;
  let directory = original_root
    .parent()
    .ok_or_else(|| "The project's original folder is unknown".to_owned())?;
  let title = folder_name(original_root)?;
  // The folder it was in may have gone meanwhile, such as one emptied in
  // Finder; it is made again rather than leaving the project stranded.
  std::fs::create_dir_all(directory)
    .map_err(|error| format!("{title} could not be restored: {error}"))?;
  let (destination, name) = rename_into(root, directory, &title)
    .map_err(|error| format!("{title} could not be restored: {error}"))?;
  name_manifest(file, &destination, &name)
}

/// Marks the deleted folder hidden on Windows, where a leading dot does not.
#[cfg(target_os = "windows")]
fn hide(folder: &Path) {
  use windows::core::HSTRING;
  use windows::Win32::Storage::FileSystem::{SetFileAttributesW, FILE_ATTRIBUTE_HIDDEN};
  let path = HSTRING::from(folder.as_os_str());
  if let Err(error) = unsafe { SetFileAttributesW(&path, FILE_ATTRIBUTE_HIDDEN) } {
    eprintln!("Could not hide {}: {error}", folder.display());
  }
}

#[cfg(not(target_os = "windows"))]
fn hide(_folder: &Path) {}
