// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! A template as a package of files, to be imported, exported or removed:
//! its manifest at the top and the fonts it brings, with their licences, in
//! `fonts/`. Only those files travel: anything else in a folder or a zip is
//! left behind, so a package can carry nothing the app would not read.
//! Errors are in English, for an alert's detail.

mod zip_files;

use std::path::{Path, PathBuf};

use super::fonts::{files_of, is_font_file, FONTS};
use super::{
  bundled, parse_manifest, user, TextTemplate, TextTemplateId, TextTemplateSource, MANIFEST_FILE,
};

/// The most one file in a package may weigh, and all of them together.
pub(super) const MAX_FILE_BYTES: u64 = 16 * 1024 * 1024;
const MAX_PACKAGE_BYTES: u64 = 32 * 1024 * 1024;
/// The most files a package may carry.
const MAX_FILES: usize = 64;

/// A template's files, each at its place in the package, `/` between
/// folders.
#[derive(Clone, Debug)]
pub struct TemplatePackage {
  pub template: TextTemplate,
  files: Vec<(String, Vec<u8>)>,
}

/// Whether a package carries a file at `path` within it.
fn is_kept(path: &str) -> bool {
  path == MANIFEST_FILE || is_font_file(path)
}

/// Files gathered into a package, held to its limits as they come.
#[derive(Default)]
struct Gathered {
  files: Vec<(String, Vec<u8>)>,
  bytes: u64,
}

impl Gathered {
  fn add(
    &mut self,
    path: String,
    size: u64,
    read: &mut dyn FnMut() -> std::io::Result<Vec<u8>>,
  ) -> Result<(), String> {
    if !is_kept(&path) {
      return Ok(());
    }
    if self.files.len() >= MAX_FILES {
      return Err(format!("it holds more than {MAX_FILES} files"));
    }
    if size > MAX_FILE_BYTES {
      return Err(format!("{path} is larger than {} MB", MAX_FILE_BYTES >> 20));
    }
    self.bytes += size;
    if self.bytes > MAX_PACKAGE_BYTES {
      return Err(format!("it is larger than {} MB", MAX_PACKAGE_BYTES >> 20));
    }
    let bytes = read().map_err(|error| format!("{path} cannot be read: {error}"))?;
    if bytes.len() as u64 > size {
      return Err(format!("{path} is larger than it says"));
    }
    self.files.push((path, bytes));
    Ok(())
  }

  /// The package these files make: a manifest that reads, naming a template
  /// whose id is not one of the app's own, with every file it declares.
  fn into_package(self) -> Result<TemplatePackage, String> {
    let manifest = self
      .files
      .iter()
      .find(|(path, _)| path == MANIFEST_FILE)
      .ok_or_else(|| format!("it has no {MANIFEST_FILE}"))?;
    let json =
      std::str::from_utf8(&manifest.1).map_err(|_| format!("its {MANIFEST_FILE} is not text"))?;
    let template =
      parse_manifest(json, TextTemplateSource::User).map_err(|error| error.to_string())?;
    if bundled::manifest(template.id.as_str()).is_some() {
      return Err(format!(
        "the id \"{}\" is one of the app's own templates",
        template.id
      ));
    }
    if let Some(missing) =
      files_of(&template.fonts).find(|file| self.files.iter().all(|(path, _)| path != file))
    {
      return Err(format!("it declares {missing}, which it does not carry"));
    }
    Ok(TemplatePackage {
      template,
      files: self.files,
    })
  }
}

impl TemplatePackage {
  /// The package in `folder`: its manifest, and its fonts.
  pub fn read_folder(folder: &Path) -> Result<Self, String> {
    let mut gathered = Gathered::default();
    let mut add = |relative: String, path: PathBuf| {
      let size = std::fs::metadata(&path)
        .map_err(|error| format!("{relative} cannot be read: {error}"))?
        .len();
      gathered.add(relative, size, &mut || std::fs::read(&path))
    };
    add(MANIFEST_FILE.to_owned(), folder.join(MANIFEST_FILE))?;
    if let Ok(entries) = std::fs::read_dir(folder.join(FONTS)) {
      let mut fonts: Vec<_> = entries
        .filter_map(Result::ok)
        .filter(|entry| entry.file_type().is_ok_and(|kind| kind.is_file()))
        .collect();
      fonts.sort_by_key(|entry| entry.file_name());
      for entry in fonts {
        let name = entry.file_name().to_string_lossy().into_owned();
        add(format!("{FONTS}/{name}"), entry.path())?;
      }
    }
    gathered.into_package()
  }

  /// The package zipped at `path`: its manifest at the top, or in one folder
  /// at the top, as zipping a template's folder makes it.
  pub fn read_zip(path: &Path) -> Result<Self, String> {
    let mut gathered = Gathered::default();
    zip_files::read(path, |relative, size, read| {
      gathered.add(relative, size, read)
    })?;
    gathered.into_package()
  }

  /// The package of template `id` as the app has it: one of its own, with
  /// the shared files it declares, or one of yours from its folder.
  pub fn of(id: &TextTemplateId) -> Result<Self, String> {
    if let Some(json) = bundled::manifest(id.as_str()) {
      let template =
        parse_manifest(json, TextTemplateSource::Bundled).map_err(|error| error.to_string())?;
      let mut files = vec![(MANIFEST_FILE.to_owned(), json.as_bytes().to_vec())];
      for file in files_of(&template.fonts) {
        if files.iter().all(|(path, _)| path != file) {
          let bytes =
            bundled::font_file(file).ok_or_else(|| format!("the app does not carry {file}"))?;
          files.push((file.to_owned(), bytes.to_vec()));
        }
      }
      return Ok(Self { template, files });
    }
    let folder =
      user::folder_of(id.as_str()).ok_or_else(|| format!("there is no template \"{id}\""))?;
    Self::read_folder(&folder)
  }

  /// Writes the package as a zip at `destination`, its files in a folder
  /// named for the template, so unzipping it gives the template's folder.
  pub fn write_zip(&self, destination: &Path) -> Result<(), String> {
    zip_files::write(destination, self.template.id.as_str(), &self.files)
  }

  /// Puts the package in your folder of templates, `folder`, in place of
  /// any of yours with its id, and reads your templates again. It is
  /// written beside them first and only then moved into place, so a package
  /// that cannot be written leaves nothing half there.
  pub fn install(&self, folder: &Path) -> Result<(), String> {
    let id = self.template.id.as_str();
    let destination = folder.join(id);
    let staging = folder.join(format!(".importing-{id}"));
    let _ = std::fs::remove_dir_all(&staging);
    let written = (|| {
      for (path, bytes) in &self.files {
        let file = staging.join(path);
        if let Some(parent) = file.parent() {
          std::fs::create_dir_all(parent)?;
        }
        std::fs::write(file, bytes)?;
      }
      // One of yours with this id, wherever it was read from, gives way.
      for old in user::folder_of(id).into_iter().chain([destination.clone()]) {
        if old.exists() {
          std::fs::remove_dir_all(old)?;
        }
      }
      std::fs::rename(&staging, &destination)
    })();
    if let Err(error) = written {
      let _ = std::fs::remove_dir_all(&staging);
      return Err(format!(
        "it cannot be written to {}: {error}",
        folder.display()
      ));
    }
    super::load_user_templates(folder);
    Ok(())
  }
}

/// Moves your template `id` to the Trash, or the Recycle Bin, where it can
/// still be put back, and reads your templates in `folder` again. The app's
/// own cannot be removed.
pub fn remove_user_template(id: &TextTemplateId, folder: &Path) -> Result<(), String> {
  let template_folder =
    user::folder_of(id.as_str()).ok_or_else(|| format!("\"{id}\" is not one of your templates"))?;
  trash::delete(&template_folder).map_err(|error| {
    let bin = if cfg!(target_os = "windows") {
      "Recycle Bin"
    } else {
      "Trash"
    };
    format!("it could not be moved to the {bin}: {error}")
  })?;
  super::load_user_templates(folder);
  Ok(())
}

#[cfg(test)]
mod tests;
