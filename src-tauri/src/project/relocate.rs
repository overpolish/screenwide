// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Moving and duplicating whole projects. A project is a folder whose
//! manifest is named after it and whose media is named relative to it, so
//! either is the folder taken somewhere else under a name no other project
//! there has, with the manifest renamed to match.

use std::fs::{File, FileTimes};
use std::io::{ErrorKind, Read, Write};
use std::path::{Path, PathBuf};

use super::folder::{rename_settling, reserve};
use super::EXTENSION;

/// Large enough that a multi-gigabyte recording copies at disk speed, small
/// enough that progress moves often.
const COPY_CHUNK: usize = 4 * 1024 * 1024;

fn root_of(file: &Path) -> Result<&Path, String> {
  file
    .parent()
    .ok_or_else(|| "The project has no folder".to_owned())
}

/// Moves the project whose manifest is `file` into `directory`, and returns
/// its manifest there. On the same disk it is a rename; across disks the
/// folder is copied, checked by the copy finishing, and only then removed.
/// `on_copied` hears every byte that has reached its new place, a renamed
/// project's all at once.
pub(crate) fn move_into(
  file: &Path,
  directory: &Path,
  size: u64,
  on_copied: &mut dyn FnMut(u64),
) -> Result<PathBuf, String> {
  let root = root_of(file)?;
  let here = root
    .parent()
    .is_some_and(|parent| same_place(parent, directory));
  if here {
    on_copied(size);
    return Ok(file.to_path_buf());
  }
  let title = folder_name(root)?;
  let (destination, name) = match rename_into(root, directory, &title) {
    Ok(moved) => {
      on_copied(size);
      moved
    }
    Err(error) if error.kind() == ErrorKind::CrossesDevices => {
      let copied = copy_into(root, directory, &title, on_copied)?;
      std::fs::remove_dir_all(root).map_err(|error| {
        format!(
          "{title} was copied to its new place, but the original could not be removed: {error}"
        )
      })?;
      copied
    }
    Err(error) => return Err(format!("{title} could not be moved: {error}")),
  };
  name_manifest(file, &destination, &name)
}

/// Copies the project whose manifest is `file` beside itself, named
/// `title` followed by the platform's word for a copy, and returns the
/// copy's manifest. A name the original was given while open is its own,
/// so the copy does not take it.
pub(crate) fn duplicate(
  file: &Path,
  title: &str,
  on_copied: &mut dyn FnMut(u64),
) -> Result<PathBuf, String> {
  let root = root_of(file)?;
  let directory = root
    .parent()
    .ok_or_else(|| "The project's folder has no parent".to_owned())?;
  let copy_title = if cfg!(target_os = "windows") {
    format!("{title} - Copy")
  } else {
    format!("{title} copy")
  };
  let (destination, name) = copy_into(root, directory, &copy_title, on_copied)?;
  let copied = name_manifest(file, &destination, &name)?;
  super::update(&copied, |manifest| manifest.title.take().is_some())?;
  Ok(copied)
}

pub(super) fn folder_name(root: &Path) -> Result<String, String> {
  root
    .file_name()
    .and_then(|name| name.to_str())
    .map(str::to_owned)
    .ok_or_else(|| "The project's folder has no name".to_owned())
}

fn same_place(a: &Path, b: &Path) -> bool {
  match (std::fs::canonicalize(a), std::fs::canonicalize(b)) {
    (Ok(a), Ok(b)) => a == b,
    _ => a == b,
  }
}

/// Renames `root` into `directory` under the first free name for `title`.
/// Fails with the platform's cross-device error when the two are on
/// different disks, for the caller to copy instead.
pub(super) fn rename_into(
  root: &Path,
  directory: &Path,
  title: &str,
) -> std::io::Result<(PathBuf, String)> {
  let _turn = super::manifest::hold_writes();
  let mut suffix = 1_u32;
  loop {
    let name = if suffix == 1 {
      title.to_owned()
    } else {
      format!("{title} ({suffix})")
    };
    let destination = directory.join(&name);
    // A rename onto an empty folder replaces it on Unix, so a taken name is
    // looked for first rather than left to the rename to refuse.
    if destination.exists() {
      suffix += 1;
      continue;
    }
    match rename_settling(root, &destination) {
      Ok(()) => return Ok((destination, name)),
      Err(error)
        if matches!(
          error.kind(),
          ErrorKind::AlreadyExists | ErrorKind::DirectoryNotEmpty
        ) =>
      {
        suffix += 1;
      }
      Err(error) => return Err(error),
    }
  }
}

/// Copies everything in `root` into a new folder in `directory` named for
/// `title`. A copy that fails part way is removed rather than left half
/// made.
fn copy_into(
  root: &Path,
  directory: &Path,
  title: &str,
  on_copied: &mut dyn FnMut(u64),
) -> Result<(PathBuf, String), String> {
  let (destination, name) =
    reserve(directory, title).map_err(|error| format!("{title} could not be copied: {error}"))?;
  if let Err(error) = copy_tree(root, &destination, on_copied) {
    let _ = std::fs::remove_dir_all(&destination);
    return Err(format!("{title} could not be copied: {error}"));
  }
  Ok((destination, name))
}

/// Copies the folders and files under `from` into `to`, keeping each file's
/// modification time: a project's manifest time is when it was last edited,
/// which a move or a copy does not change. Links are skipped; a project
/// holds none the app made.
fn copy_tree(from: &Path, to: &Path, on_copied: &mut dyn FnMut(u64)) -> std::io::Result<()> {
  let mut folders = vec![(from.to_path_buf(), to.to_path_buf())];
  while let Some((source, target)) = folders.pop() {
    for entry in std::fs::read_dir(&source)? {
      let entry = entry?;
      let kind = entry.file_type()?;
      let destination = target.join(entry.file_name());
      if kind.is_dir() {
        std::fs::create_dir(&destination)?;
        folders.push((entry.path(), destination));
      } else if kind.is_file() {
        copy_file(&entry.path(), &destination, on_copied)?;
      }
    }
  }
  Ok(())
}

fn copy_file(from: &Path, to: &Path, on_copied: &mut dyn FnMut(u64)) -> std::io::Result<()> {
  let mut source = File::open(from)?;
  let modified = source.metadata()?.modified()?;
  let mut target = File::create_new(to)?;
  let mut buffer = vec![0; COPY_CHUNK];
  loop {
    let read = match source.read(&mut buffer) {
      Ok(0) => break,
      Ok(read) => read,
      Err(error) if error.kind() == ErrorKind::Interrupted => continue,
      Err(error) => return Err(error),
    };
    target.write_all(&buffer[..read])?;
    on_copied(read as u64);
  }
  target.set_times(FileTimes::new().set_modified(modified))?;
  target.sync_all()
}

/// The manifest `file` had, now in `destination`, renamed to the folder's
/// `name` when that differs, as a project's manifest always matches its
/// folder.
pub(super) fn name_manifest(
  file: &Path,
  destination: &Path,
  name: &str,
) -> Result<PathBuf, String> {
  let moved = destination.join(
    file
      .file_name()
      .ok_or_else(|| "The project has no file name".to_owned())?,
  );
  let named = destination.join(format!("{name}.{EXTENSION}"));
  if moved != named {
    rename_settling(&moved, &named)
      .map_err(|error| format!("The project's file could not be renamed: {error}"))?;
  }
  Ok(named)
}
