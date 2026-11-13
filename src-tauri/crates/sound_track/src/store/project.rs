// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The sounds a project carries: a copy of each its clips play, with the
//! file kept beside it, so the project is whole on its own and hands them
//! to the library of any computer it is opened on.

use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use super::singing::copy_if_more;
use super::{directory, is_name, is_stem, read_kept, removed, write_kept, Sound};
use crate::AudioClipKind;

/// Copies the sound `name` from the folder `from` into the folder `to`
/// where `to` lacks it, and the singing separated from it where `from` holds
/// more of that than `to`.
fn copy_sound(from: &Path, to: &Path, name: &str) -> Result<(), String> {
  std::fs::create_dir_all(to).map_err(|error| error.to_string())?;
  if read_kept(to, name).is_none() {
    let Some(sound) = read_kept(from, name) else {
      return Ok(());
    };
    copy_whole(&from.join(name), &to.join(name))?;
    write_kept(to, &sound)?;
  }
  copy_if_more(&from.join(name), &to.join(name), copy_over)
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

/// Takes out of the project in `project_sounds` every sound none of `used`
/// names, with the files kept beside it: its record, the singing separated
/// from it, and any copy left half made. Undo can bring a clip back only
/// while the project is open, and the library keeps every sound, so this
/// runs as the project is opened or closed. Anything that is not a sound's
/// is left alone.
pub fn prune<'a>(project_sounds: &Path, used: impl IntoIterator<Item = &'a str>) {
  let kept: Vec<&str> = used
    .into_iter()
    .filter_map(|name| name.split_once('.').map(|(stem, _)| stem))
    .collect();
  let Ok(entries) = std::fs::read_dir(project_sounds) else {
    return;
  };
  for entry in entries.flatten() {
    let Ok(name) = entry.file_name().into_string() else {
      continue;
    };
    let Some((stem, _)) = name.split_once('.') else {
      continue;
    };
    if is_stem(stem) && !kept.contains(&stem) {
      if let Err(error) = std::fs::remove_file(entry.path()) {
        eprintln!("Could not take {name} out of the project: {error}");
      }
    }
  }
}

/// Hands the library every sound the project in `project_sounds` carries
/// that it lacks, with the singing separated from it, so the project plays
/// on a computer that has never seen them, and they can be used again in
/// another; but not one taken out of the library here, which the project
/// keeps playing from its own copy.
pub fn adopt(project_sounds: &Path) {
  let (Ok(folder), Ok(entries)) = (directory(), std::fs::read_dir(project_sounds)) else {
    return;
  };
  for entry in entries.flatten() {
    let Ok(name) = entry.file_name().into_string() else {
      continue;
    };
    if !is_name(&name) || removed::was_removed(folder, &name) {
      continue;
    }
    if let Err(error) = copy_sound(project_sounds, folder, &name) {
      eprintln!("Could not add {name} to the sound library: {error}");
    }
  }
}

/// Takes the sound `name` the project in `project_sounds` carries into the
/// library, as though added again: listed as `kind`, newest, and no longer
/// noted as taken out. Says what it is there.
pub fn keep_from_project(
  project_sounds: &Path,
  name: &str,
  kind: AudioClipKind,
) -> Result<Sound, String> {
  let folder = directory()?;
  let carried = is_name(name)
    .then(|| read_kept(project_sounds, name))
    .flatten()
    .ok_or("The project holds no copy of that sound")?;
  copy_sound(project_sounds, folder, name)?;
  removed::forget(folder, name);
  let sound = Sound {
    added_ms: SystemTime::now()
      .duration_since(UNIX_EPOCH)
      .map_or(0, |since| since.as_millis() as u64),
    kind: Some(kind),
    ..read_kept(folder, name).unwrap_or(carried)
  };
  write_kept(folder, &sound)?;
  Ok(sound)
}

fn copy_whole(from: &Path, to: &Path) -> Result<(), String> {
  if to.is_file() {
    return Ok(());
  }
  copy_over(from, to)
}

/// Copies `from` over `to`, which only ever holds a whole file.
fn copy_over(from: &Path, to: &Path) -> Result<(), String> {
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
