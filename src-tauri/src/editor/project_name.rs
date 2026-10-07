// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! A project's name, and renaming it.
//!
//! A project is named by its folder and manifest. One open in an editor
//! cannot have them renamed under it: the editor and its player hold its
//! files by where they are, and Windows will not rename a folder holding an
//! open file. So a name given while it is open is kept in its manifest and
//! shown at once, and the folder and manifest take it when the editor lets
//! the project go, or when it is next opened if the app stopped first.

use super::*;

fn stem(file: &Path) -> Option<&str> {
  file.file_stem().and_then(|stem| stem.to_str())
}

/// Renames the project whose manifest is `file` to `title`. Returns its
/// manifest as it is now: the same one while an editor has it open, which
/// takes the name once the editor lets it go, or the renamed one.
pub fn rename_project(app: &AppHandle, file: &Path, title: &str) -> Result<PathBuf, String> {
  let title = sanitize_file_stem(title).ok_or_else(|| "That name cannot be used".to_owned())?;
  let Some(kind) = open_kind(app, file) else {
    return rename_now(app, file, &title);
  };
  crate::project::update(file, |manifest| {
    let pending = (stem(file) != Some(title.as_str())).then(|| title.clone());
    let changed = manifest.title != pending;
    manifest.title = pending;
    changed
  })?;
  {
    let state = app.state::<EditorState>();
    let mut artifact = state
      .slot(kind)
      .artifact
      .lock()
      .unwrap_or_else(|poisoned| poisoned.into_inner());
    if let Some(
      EditorArtifact::Recording {
        suggested_file_stem,
        ..
      }
      | EditorArtifact::Screenshot {
        suggested_file_stem,
        ..
      },
    ) = artifact.as_mut()
    {
      suggested_file_stem.clone_from(&title);
    }
  }
  emit_snapshot(app, kind);
  crate::project::library::notify(app);
  Ok(file.to_path_buf())
}

/// Renames the folder and manifest now, and drops any name kept for later.
fn rename_now(app: &AppHandle, file: &Path, title: &str) -> Result<PathBuf, String> {
  let renamed = if stem(file) == Some(title) {
    file.to_path_buf()
  } else {
    let renamed = crate::project::rename(file, title)?;
    if let Err(error) = crate::project::library::moved(app, file, &renamed) {
      eprintln!("Could not follow the renamed project in the recent list: {error}");
    }
    renamed
  };
  if let Err(error) = crate::project::update(&renamed, |manifest| manifest.title.take().is_some()) {
    eprintln!("Could not settle the project's name: {error}");
  }
  Ok(renamed)
}

/// Gives the project whose manifest is `file` the name it was given while an
/// editor had it open, unless one still does. Returns its manifest as it is
/// now. A name that cannot be taken yet, such as one another project has, is
/// kept for the next chance.
pub(crate) fn settle_name(app: &AppHandle, file: &Path) -> PathBuf {
  if open_kind(app, file).is_some() {
    return file.to_path_buf();
  }
  let Some(title) = crate::project::read(file)
    .ok()
    .and_then(|manifest| manifest.title)
  else {
    return file.to_path_buf();
  };
  rename_now(app, file, &title).unwrap_or_else(|error| {
    eprintln!("Could not rename {} to {title}: {error}", file.display());
    file.to_path_buf()
  })
}
