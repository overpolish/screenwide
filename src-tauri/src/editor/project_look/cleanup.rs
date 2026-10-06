// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Takes out of a project the pictures it no longer uses.
//!
//! While the editor has a project open, every background picture shown is
//! copied in and none is taken out, so undo can go back to any of them. Once
//! the project is opened or closed there is no undo history, so what it does
//! not use goes: it keeps the background picture it shows and the pictures
//! its images show. A picture it remembers behind a colour, so the picture
//! is there to switch back to, moves to Screenwide's own folder instead.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use serde_json::Value;

use super::pictures::{
  backgrounds as each_background, copy_into, folder, stored_names, PICTURES_FOLDER,
};
use crate::editor::images::store;

/// Cleans the project whose manifest is `project`, reporting rather than
/// failing: a picture left behind costs only disk space.
pub(crate) fn clean_pictures(project: &Path) {
  if let Err(error) = clean(project, store::backgrounds_directory().as_deref()) {
    eprintln!(
      "Could not tidy the pictures in {}: {error}",
      project.display()
    );
  }
}

/// The name a background path gives a file in the project's pictures, if it
/// names one: as the manifest keeps a shown picture, relative, or as the
/// window last sent a picture it had been shown from the project, absolute.
fn picture_in_project(root: &Path, path: &str) -> Option<String> {
  let inside = match Path::new(path).strip_prefix(root) {
    Ok(inside) => inside.to_path_buf(),
    Err(_) if Path::new(path).is_relative() => PathBuf::from(path),
    Err(_) => return None,
  };
  let name = inside.strip_prefix(PICTURES_FOLDER).ok()?;
  (name.components().count() == 1)
    .then(|| name.to_str().map(str::to_owned))
    .flatten()
}

pub(super) fn clean(project: &Path, backgrounds: Option<&Path>) -> Result<(), String> {
  let folder = folder(project)?;
  if !folder.is_dir() {
    return Ok(());
  }
  let root = project.parent().ok_or("The project has no folder")?;
  let mut kept = BTreeSet::new();
  let mut stored = BTreeSet::new();
  crate::project::update(project, |manifest| {
    if let Ok(timeline) = serde_json::to_value(&manifest.timeline) {
      stored_names(&timeline, &mut stored);
    }
    let Some(look) = manifest.look.as_mut() else {
      return false;
    };
    // A screenshot's images are drawn on its layers, which its look keeps.
    stored_names(look, &mut stored);
    let mut changed = false;
    each_background(look, &mut |fields| {
      let Some(name) = fields
        .get("backgroundImagePath")
        .and_then(Value::as_str)
        .and_then(|path| picture_in_project(root, path))
      else {
        return;
      };
      let shown = fields.get("backgroundType").and_then(Value::as_str) == Some("image");
      let moved = (!shown)
        .then(|| backgrounds.and_then(|to| copy_into(to, &folder.join(&name), &name).ok()))
        .flatten();
      match moved {
        Some(target) => {
          fields.insert(
            "backgroundImagePath".to_owned(),
            Value::String(target.to_string_lossy().into_owned()),
          );
          changed = true;
        }
        // Shown, or with nowhere to go: it stays.
        None => {
          kept.insert(name);
        }
      }
    });
    changed
  })?;
  for entry in std::fs::read_dir(&folder)
    .map_err(|error| error.to_string())?
    .flatten()
  {
    let path = entry.path();
    let name = entry.file_name().to_string_lossy().into_owned();
    let stem = path
      .file_stem()
      .map(|stem| stem.to_string_lossy().into_owned());
    if kept.contains(&name) || stem.is_some_and(|stem| stored.contains(&stem)) {
      continue;
    }
    if let Err(error) = std::fs::remove_file(&path) {
      eprintln!("Could not remove {}: {error}", path.display());
    }
  }
  Ok(())
}
