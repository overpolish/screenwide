// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The pictures a project shows, kept in `media/pictures/`: a background
//! picture chosen for its canvas, and the stored pictures its images show.
//! Each is named by its contents, so the same picture is kept once.

use std::collections::{BTreeSet, HashMap};
use std::path::{Path, PathBuf};
use std::sync::{Mutex, OnceLock};
use std::time::SystemTime;

use serde_json::{Map, Value};

use crate::editor::images::store;
use crate::screenshots::ScreenshotOutputSettings;

pub(super) const PICTURES_FOLDER: &str = "media/pictures";

pub(super) fn folder(project: &Path) -> Result<PathBuf, String> {
  crate::project::resolve(project, PICTURES_FOLDER)
}

/// The content name of `path` as it is now. The look is saved after every
/// settled change while the canvas still names the picture where it was
/// chosen, so the name is kept per file version rather than hashed each time.
fn content_name(path: &Path) -> Result<String, String> {
  type Version = (PathBuf, u64, Option<SystemTime>);
  static NAMES: OnceLock<Mutex<HashMap<Version, String>>> = OnceLock::new();
  let metadata = std::fs::metadata(path).map_err(|error| error.to_string())?;
  let version = (path.to_path_buf(), metadata.len(), metadata.modified().ok());
  let names = NAMES.get_or_init(Default::default);
  if let Some(name) = names
    .lock()
    .ok()
    .and_then(|names| names.get(&version).cloned())
  {
    return Ok(name);
  }
  let name = store::content_file_name(path)?;
  if let Ok(mut names) = names.lock() {
    names.insert(version, name.clone());
  }
  Ok(name)
}

/// Copies `source` into `folder` as `name`, unless it is there already, and
/// says where it is. The name is the content's own, so one that is there is
/// the same picture. Copied rather than moved, as the two can be on
/// different drives.
pub(super) fn copy_into(folder: &Path, source: &Path, name: &str) -> Result<PathBuf, String> {
  let target = folder.join(name);
  if target.is_file() {
    return Ok(target);
  }
  std::fs::create_dir_all(folder).map_err(|error| error.to_string())?;
  let partial = folder.join(format!("{name}.part"));
  std::fs::copy(source, &partial)
    .and_then(|_| std::fs::rename(&partial, &target))
    .map_err(|error| {
      let _ = std::fs::remove_file(&partial);
      error.to_string()
    })?;
  Ok(target)
}

/// Names the background picture of a look remembered for the next capture
/// in Screenwide's own folder, copying it there first. The look the next
/// capture starts from must not point into a project, which can be moved or
/// trashed, nor at a file somewhere else that can be deleted.
pub(crate) fn keep_in_app_data(output: &mut ScreenshotOutputSettings) {
  if let Some(folder) = store::backgrounds_directory() {
    keep_in(&folder, output);
  }
}

pub(super) fn keep_in(folder: &Path, output: &mut ScreenshotOutputSettings) {
  let Some(path) = output.background_image_path.as_deref().map(PathBuf::from) else {
    return;
  };
  if path.starts_with(folder) || !path.is_file() {
    return;
  }
  match content_name(&path).and_then(|name| copy_into(folder, &path, &name)) {
    Ok(kept) => output.background_image_path = Some(kept.to_string_lossy().into_owned()),
    Err(error) => eprintln!("Could not keep {}: {error}", path.display()),
  }
}

/// Calls `visit` with every part of a saved look that has a background:
/// both panes of a recording, or a screenshot's canvas and each of its layers.
pub(super) fn backgrounds(value: &mut Value, visit: &mut impl FnMut(&mut Map<String, Value>)) {
  match value {
    Value::Object(fields) => {
      if fields.contains_key("backgroundImagePath") {
        visit(fields);
      }
      fields
        .values_mut()
        .for_each(|value| backgrounds(value, visit));
    }
    Value::Array(values) => values
      .iter_mut()
      .for_each(|value| backgrounds(value, visit)),
    _ => {}
  }
}

/// Names each background picture the look shows inside the project, copying
/// it in if it is somewhere else. A picture remembered behind a colour stays
/// named where it is: the editor tidies it away once it lets the project go,
/// so undo can still reach it until then. One that cannot be read is left
/// named where it is too, so the save goes ahead.
pub(crate) fn embed_backgrounds(project: &Path, look: &mut Value) {
  backgrounds(look, &mut |fields| {
    if fields.get("backgroundType").and_then(Value::as_str) != Some("image") {
      return;
    }
    let Some(path) = fields
      .get("backgroundImagePath")
      .and_then(Value::as_str)
      .map(PathBuf::from)
    else {
      return;
    };
    // A picture that is gone has nothing to keep; the canvas draws without
    // it, and choosing another names one that is there.
    if !path.is_absolute() || !path.is_file() {
      return;
    }
    let embedded = (|| {
      let root = project.parent().ok_or("The project has no folder")?;
      if let Ok(inside) = path.strip_prefix(root) {
        return inside
          .to_str()
          .map(|inside| inside.replace('\\', "/"))
          .ok_or_else(|| "The picture's name cannot be kept".to_owned());
      }
      let name = content_name(&path)?;
      copy_into(&folder(project)?, &path, &name)?;
      Ok::<_, String>(format!("{PICTURES_FOLDER}/{name}"))
    })();
    match embedded {
      Ok(name) => {
        fields.insert("backgroundImagePath".to_owned(), Value::String(name));
      }
      Err(error) => eprintln!("Could not keep {} in the project: {error}", path.display()),
    }
  });
}

/// Names each background picture where it is now. One named in the project
/// that reaches outside it is dropped, as a project can come from anyone.
pub(crate) fn resolve_backgrounds(project: &Path, look: &mut Value) {
  backgrounds(look, &mut |fields| {
    let Some(name) = fields.get("backgroundImagePath").and_then(Value::as_str) else {
      return;
    };
    if Path::new(name).is_absolute() {
      return;
    }
    let resolved = crate::project::resolve(project, name).map_or(Value::Null, |path| {
      Value::String(path.to_string_lossy().into_owned())
    });
    fields.insert("backgroundImagePath".to_owned(), resolved);
  });
}

/// Every stored picture `value` names, wherever in it.
pub(super) fn stored_names(value: &serde_json::Value, names: &mut BTreeSet<String>) {
  match value {
    serde_json::Value::String(text) => {
      if let Some(name) = text.strip_prefix(store::IMAGE_PREFIX) {
        if store::is_name(name) {
          names.insert(name.to_owned());
        }
      }
    }
    serde_json::Value::Array(values) => values.iter().for_each(|value| stored_names(value, names)),
    serde_json::Value::Object(fields) => {
      fields.values().for_each(|value| stored_names(value, names));
    }
    _ => {}
  }
}

/// Copies into the project each stored picture `content` shows that it does
/// not hold yet. A picture the store has lost is skipped: the project then
/// shows it only where the store still has it.
pub(crate) fn carry_pictures(project: &Path, content: &impl serde::Serialize) {
  let Ok(value) = serde_json::to_value(content) else {
    return;
  };
  let mut names = BTreeSet::new();
  stored_names(&value, &mut names);
  for name in names {
    let Some(source) = store::stored_file(&name) else {
      continue;
    };
    let Some(file_name) = source.file_name().and_then(|file| file.to_str()) else {
      continue;
    };
    if let Err(error) = folder(project).and_then(|folder| copy_into(&folder, &source, file_name)) {
      eprintln!("Could not keep a picture in {}: {error}", project.display());
    }
  }
}

/// Hands the store every picture the project carries that it lacks, so the
/// project's images show on a computer that has never seen them.
pub(crate) fn adopt_pictures(project: &Path) {
  let Ok(entries) =
    folder(project).and_then(|folder| std::fs::read_dir(folder).map_err(|error| error.to_string()))
  else {
    return;
  };
  for entry in entries.flatten() {
    if let Err(error) = store::adopt(&entry.path()) {
      eprintln!("Could not use {}: {error}", entry.path().display());
    }
  }
}
