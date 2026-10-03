// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The pictures on the macOS desktop.
//!
//! `NSWorkspace.desktopImageURL` answers a placeholder,
//! `DefaultDesktop.heic`, for every wallpaper an extension draws, which since
//! Sonoma is most of them. The wallpaper store's `Index.plist` holds the real
//! choice. A file someone picked is named there directly. An extension's
//! choice is matched to the picture the system rendered for it, kept as
//! `/private/var/db/Wallpapers/<UUID>/Wallpaper.png` beside a
//! `Metadata.plist` that records the same choice.

use std::path::{Path, PathBuf};
use std::time::SystemTime;

use plist::{Dictionary, Value};

/// The store, under the home folder.
const STORE: &str = "Library/Application Support/com.apple.wallpaper/Store/Index.plist";
/// Where the system keeps the pictures its extensions render.
const RENDERED: &str = "/private/var/db/Wallpapers";
const PICTURE: &str = "Wallpaper.png";
const METADATA: &str = "Metadata.plist";

/// One wallpaper choice: the extension that draws it, its options, and the
/// files it shows when it shows a person's own pictures.
#[derive(Debug, PartialEq)]
struct Choice {
  provider: String,
  configuration: Vec<u8>,
  files: Vec<PathBuf>,
}

/// A rendered picture, the choice it was rendered for, and when.
struct Rendered {
  picture: PathBuf,
  choice: Choice,
  written: SystemTime,
}

/// The first choice under a `Content` dictionary. A desktop shows one, and a
/// shuffle lists the rest after it.
fn choice(content: &Dictionary) -> Option<Choice> {
  let first = content
    .get("Choices")?
    .as_array()?
    .first()?
    .as_dictionary()?;
  let files = first
    .get("Files")
    .and_then(Value::as_array)
    .into_iter()
    .flatten()
    .filter_map(|file| file.as_dictionary()?.get("relative")?.as_string())
    .filter_map(|address| url::Url::parse(address).ok()?.to_file_path().ok())
    .collect();
  Some(Choice {
    provider: first.get("Provider")?.as_string()?.to_owned(),
    configuration: first
      .get("Configuration")
      .and_then(Value::as_data)
      .map(<[u8]>::to_vec)
      .unwrap_or_default(),
    files,
  })
}

/// Every desktop choice in the store, in the order it lists them: the one
/// shared by every display and Space, then any a display or a Space overrides
/// it with. A choice sits under `Linked` when the screen saver shares it and
/// under `Desktop` when it does not. `Idle` is the screen saver alone and
/// `SystemDefault` is what a new display starts with; neither is on a desktop.
fn desktop_choices(value: &Value, found: &mut Vec<Choice>) {
  let Some(dictionary) = value.as_dictionary() else {
    return;
  };
  for (key, child) in dictionary {
    match key.as_str() {
      "Idle" | "SystemDefault" => {}
      "Linked" | "Desktop" => {
        let content = child
          .as_dictionary()
          .and_then(|entry| entry.get("Content"))
          .and_then(Value::as_dictionary);
        found.extend(content.and_then(choice));
      }
      _ => desktop_choices(child, found),
    }
  }
}

fn written(path: &Path) -> SystemTime {
  std::fs::metadata(path)
    .and_then(|metadata| metadata.modified())
    .unwrap_or(SystemTime::UNIX_EPOCH)
}

fn rendered_pictures(directory: &Path) -> Vec<Rendered> {
  let Ok(entries) = std::fs::read_dir(directory) else {
    return Vec::new();
  };
  entries
    .flatten()
    .filter_map(|entry| {
      let folder = entry.path();
      let picture = folder.join(PICTURE);
      if !picture.is_file() {
        return None;
      }
      let metadata = plist::Value::from_file(folder.join(METADATA)).ok()?;
      let content = metadata.as_dictionary()?.get("Content")?.as_dictionary()?;
      Some(Rendered {
        written: written(&folder),
        choice: choice(content)?,
        picture,
      })
    })
    .collect()
}

/// The picture a choice puts on the desktop.
///
/// A choice naming a file shows that file. Otherwise it is the picture the
/// system rendered for the same extension, preferring one rendered with the
/// same options and, among equals, the newest: the folder for an old choice
/// stays behind when a newer one is set.
fn picture_for(choice: &Choice, rendered: &[Rendered]) -> Option<PathBuf> {
  if let Some(file) = choice.files.iter().find(|file| file.is_file()) {
    return Some(file.clone());
  }
  rendered
    .iter()
    .filter(|one| one.choice.provider == choice.provider)
    .max_by_key(|one| {
      (
        one.choice.configuration == choice.configuration,
        one.written,
      )
    })
    .map(|one| one.picture.clone())
}

/// The pictures on the desktop, one per choice the store lists. When the
/// store cannot be read, the newest rendered picture is the best guess at
/// what is on screen.
pub(super) fn active_pictures() -> Vec<PathBuf> {
  let rendered = rendered_pictures(Path::new(RENDERED));
  let store = std::env::var_os("HOME")
    .map(|home| PathBuf::from(home).join(STORE))
    .and_then(|path| plist::Value::from_file(path).ok());
  let Some(store) = store else {
    return rendered
      .into_iter()
      .max_by_key(|one| one.written)
      .map(|one| one.picture)
      .into_iter()
      .collect();
  };
  let mut choices = Vec::new();
  desktop_choices(&store, &mut choices);
  choices
    .iter()
    .filter_map(|choice| picture_for(choice, &rendered))
    .collect()
}

#[cfg(test)]
mod tests;
