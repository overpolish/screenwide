// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Your own templates: read from a folder holding a folder each, every one
//! with its manifest at the top, laid out as the app's own are. One that
//! cannot be read, or would take an id already taken, is left out and said
//! why, and the rest are read regardless.

use std::path::{Path, PathBuf};
use std::sync::{LazyLock, RwLock};

use super::fonts::{files_of, TemplateFont};
use super::package::MAX_FILE_BYTES;
use super::{bundled, parse_manifest, TextTemplate, TextTemplateSource};

/// The file a template's folder holds its manifest in.
pub const MANIFEST_FILE: &str = "template.json";
/// The most a manifest may weigh, in bytes: far past any real one, short
/// of a file that is not one at all.
const MAX_MANIFEST_BYTES: u64 = 256 * 1024;

/// Your templates as last read, by name, each with the folder it is in.
static USER: LazyLock<RwLock<Vec<(TextTemplate, PathBuf)>>> = LazyLock::new(RwLock::default);

/// Held by each test that reads a folder of templates, since they all read
/// into the one list.
#[cfg(test)]
pub(crate) static TESTS_READING_FOLDERS: std::sync::Mutex<()> = std::sync::Mutex::new(());

/// Your templates as last read, by name.
pub(super) fn templates() -> Vec<TextTemplate> {
  USER
    .read()
    .unwrap_or_else(|poisoned| poisoned.into_inner())
    .iter()
    .map(|(template, _)| template.clone())
    .collect()
}

/// The folder your template `id` was read from, if it is one of yours.
pub(super) fn folder_of(id: &str) -> Option<PathBuf> {
  USER
    .read()
    .unwrap_or_else(|poisoned| poisoned.into_inner())
    .iter()
    .find(|(template, _)| template.id.as_str() == id)
    .map(|(_, folder)| folder.clone())
}

/// The fonts your templates bring, each with where its file is.
pub fn user_template_fonts() -> Vec<(TemplateFont, PathBuf)> {
  let user = USER.read().unwrap_or_else(|poisoned| poisoned.into_inner());
  user
    .iter()
    .flat_map(|(template, folder)| {
      template
        .fonts
        .iter()
        .map(|font| (font.clone(), folder.join(&font.file)))
    })
    .collect()
}

/// The fonts your templates bring, each with its file, read from their
/// folders. One that can no longer be read is left out, and its family set
/// in the type the text engines fall back on.
pub(super) fn font_files() -> Vec<(TemplateFont, Vec<u8>)> {
  user_template_fonts()
    .into_iter()
    .filter_map(|(font, path)| Some((font, std::fs::read(path).ok()?)))
    .collect()
}

/// The template in `folder`, which `taken` ids are not free for.
fn read_one(folder: &Path, taken: &[&str]) -> Result<TextTemplate, String> {
  let path = folder.join(MANIFEST_FILE);
  let size = std::fs::metadata(&path)
    .map_err(|error| format!("its {MANIFEST_FILE} cannot be read: {error}"))?
    .len();
  if size > MAX_MANIFEST_BYTES {
    return Err(format!("its {MANIFEST_FILE} is too large to be one"));
  }
  let json = std::fs::read_to_string(&path)
    .map_err(|error| format!("its {MANIFEST_FILE} cannot be read: {error}"))?;
  let template =
    parse_manifest(&json, TextTemplateSource::User).map_err(|error| error.to_string())?;
  if taken.contains(&template.id.as_str()) {
    return Err(format!("the id \"{}\" is already taken", template.id));
  }
  for file in files_of(&template.fonts) {
    let size = std::fs::metadata(folder.join(file))
      .map_err(|error| format!("its {file} cannot be read: {error}"))?
      .len();
    if size > MAX_FILE_BYTES {
      return Err(format!(
        "its {file} is larger than {} MB",
        MAX_FILE_BYTES >> 20
      ));
    }
  }
  Ok(template)
}

/// Reads every template in `folder` in place of the ones read before, and
/// says, in English for a log, why any were left out. A folder that is not
/// there yet holds none.
pub fn load_user_templates(folder: &Path) -> Vec<String> {
  let mut problems = Vec::new();
  let mut folders: Vec<_> = match std::fs::read_dir(folder) {
    Ok(entries) => entries
      .filter_map(Result::ok)
      .map(|entry| entry.path())
      // A folder being written by an import is not a template yet.
      .filter(|path| {
        path.is_dir()
          && !path
            .file_name()
            .is_some_and(|name| name.to_string_lossy().starts_with('.'))
      })
      .collect(),
    Err(error) if error.kind() == std::io::ErrorKind::NotFound => Vec::new(),
    Err(error) => {
      problems.push(format!("{} cannot be read: {error}", folder.display()));
      Vec::new()
    }
  };
  folders.sort();
  let mut loaded: Vec<(TextTemplate, PathBuf)> = Vec::new();
  for template_folder in folders {
    let taken: Vec<&str> = bundled::templates()
      .iter()
      .chain(loaded.iter().map(|(template, _)| template))
      .map(|template| template.id.as_str())
      .collect();
    match read_one(&template_folder, &taken) {
      Ok(template) => loaded.push((template, template_folder)),
      Err(problem) => problems.push(format!("{}: {problem}", template_folder.display())),
    }
  }
  loaded.sort_by(|(a, _), (b, _)| {
    (a.info.name.to_lowercase(), a.id.as_str()).cmp(&(b.info.name.to_lowercase(), b.id.as_str()))
  });
  *USER
    .write()
    .unwrap_or_else(|poisoned| poisoned.into_inner()) = loaded;
  problems
}
