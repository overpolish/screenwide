// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The sounds a project carries: a copy of each its clips play, with the
//! file kept beside it, so the project is whole on its own and hands them
//! to the library of any computer it is opened on.

use std::path::{Path, PathBuf};

use super::{directory, is_name, read_kept, write_kept, Sound};

/// Where the singing separated from the sound file `sound` is kept: beside
/// it, named after it.
pub fn singing_path(sound: &Path) -> PathBuf {
  let stem = sound
    .file_stem()
    .map_or_else(String::new, |stem| stem.to_string_lossy().into_owned());
  sound.with_file_name(format!("{stem}.singing.flac"))
}

/// Copies the sound `name` from the folder `from` into the folder `to`
/// where `to` lacks it, and the singing separated from it where `from` has
/// that and `to` does not.
fn copy_sound(from: &Path, to: &Path, name: &str) -> Result<(), String> {
  std::fs::create_dir_all(to).map_err(|error| error.to_string())?;
  if read_kept(to, name).is_none() {
    let Some(sound) = read_kept(from, name) else {
      return Ok(());
    };
    copy_whole(&from.join(name), &to.join(name))?;
    write_kept(to, &sound)?;
  }
  let singing = singing_path(&from.join(name));
  if singing.is_file() {
    copy_whole(&singing, &singing_path(&to.join(name)))?;
  }
  Ok(())
}

/// Copies each of `names`, and the singing separated from it, out of the
/// library into the project in `project_sounds` where the project does not
/// hold it yet. A sound the library has lost is skipped: the project keeps
/// whatever copy it had.
pub fn carry<'a>(project_sounds: &Path, names: impl IntoIterator<Item = &'a str>) {
  let Ok(folder) = directory() else {
    return;
  };
  for name in names.into_iter().filter(|name| is_name(name)) {
    if let Err(error) = copy_sound(folder, project_sounds, name) {
      eprintln!(
        "Could not keep a sound in {}: {error}",
        project_sounds.display()
      );
    }
  }
}

/// Hands the library every sound the project in `project_sounds` carries
/// that it lacks, with the singing separated from it, so the project plays
/// on a computer that has never seen them, and they can be used again in
/// another.
pub fn adopt(project_sounds: &Path) {
  let (Ok(folder), Ok(entries)) = (directory(), std::fs::read_dir(project_sounds)) else {
    return;
  };
  for entry in entries.flatten() {
    let Ok(name) = entry.file_name().into_string() else {
      continue;
    };
    if !is_name(&name) {
      continue;
    }
    if let Err(error) = copy_sound(project_sounds, folder, &name) {
      eprintln!("Could not add {name} to the sound library: {error}");
    }
  }
}

fn copy_whole(from: &Path, to: &Path) -> Result<(), String> {
  if to.is_file() {
    return Ok(());
  }
  let mut partial = to.as_os_str().to_owned();
  partial.push(".part");
  let partial = PathBuf::from(partial);
  std::fs::copy(from, &partial)
    .and_then(|_| std::fs::rename(&partial, to))
    .map_err(|error| {
      let _ = std::fs::remove_file(&partial);
      error.to_string()
    })
}

/// The sound `name` and its file, from the project's own copy in
/// `project_sounds` where it has one, or else from the library.
pub fn resolve(project_sounds: Option<&Path>, name: &str) -> Option<(PathBuf, Sound)> {
  if !is_name(name) {
    return None;
  }
  let library = directory().ok();
  project_sounds
    .into_iter()
    .chain(library)
    .find_map(|folder| read_kept(folder, name).map(|sound| (folder.join(name), sound)))
}
