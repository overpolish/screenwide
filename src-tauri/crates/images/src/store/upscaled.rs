// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Stored pictures drawn larger by the app's upscaling model, for an image
//! drawn larger than its picture, which stretching would blur.
//!
//! They live in `upscaled/` beside the stored pictures: `<name>.png` for one
//! drawn larger, `<name>.skip` for one the model is not to draw because it
//! holds text, whose letters the model can change, or is a photo, which it
//! paints over. The app decides each picture once; the image atlas reads
//! what it decided.

use std::path::PathBuf;
use std::sync::{Arc, Mutex, OnceLock};

use image::RgbaImage;

use super::{directory, fitted, is_name};

/// How many upscaled pictures are held decoded between frames.
const KEPT: usize = 8;

/// Decoded upscaled pictures, the most recently used last, and the names
/// found to have none, so a missing one is not looked for on every frame.
#[derive(Default)]
struct Held {
  loaded: Vec<(String, Arc<RgbaImage>)>,
  missing: Vec<String>,
}

fn held() -> &'static Mutex<Held> {
  static HELD: OnceLock<Mutex<Held>> = OnceLock::new();
  HELD.get_or_init(Default::default)
}

fn folder() -> Option<PathBuf> {
  Some(directory()?.join("upscaled"))
}

/// The upscaled picture for the stored picture `name`, premultiplied as
/// stored stills are, or `None` where it has none (yet).
pub fn upscaled(name: &str) -> Option<Arc<RgbaImage>> {
  if !is_name(name) {
    return None;
  }
  {
    let mut held = held().lock().ok()?;
    if let Some(index) = held.loaded.iter().position(|(known, _)| known == name) {
      let entry = held.loaded.remove(index);
      let picture = Arc::clone(&entry.1);
      held.loaded.push(entry);
      return Some(picture);
    }
    if held.missing.iter().any(|missing| missing == name) {
      return None;
    }
  }
  let read = folder()
    .and_then(|folder| image::open(folder.join(format!("{name}.png"))).ok())
    .map(|picture| {
      let picture = picture.into_rgba8();
      let (width, height) = picture.dimensions();
      Arc::new(fitted(picture, width, height))
    });
  let mut held = held().lock().ok()?;
  match &read {
    Some(picture) => {
      held.loaded.push((name.to_owned(), Arc::clone(picture)));
      if held.loaded.len() > KEPT {
        held.loaded.remove(0);
      }
    }
    None => held.missing.push(name.to_owned()),
  }
  read
}

/// Whether the app has decided about the stored picture `name`: drawn it
/// larger, or found it is not to be.
pub fn upscale_decided(name: &str) -> bool {
  folder().is_some_and(|folder| {
    folder.join(format!("{name}.png")).is_file() || folder.join(format!("{name}.skip")).is_file()
  })
}

/// The stored stills the app has not decided about yet, each with its file.
/// A moving picture is kept as the GIF or WebP it came as, and is never
/// upscaled.
pub fn undecided() -> Vec<(String, PathBuf)> {
  let Some(store) = directory() else {
    return Vec::new();
  };
  let Ok(entries) = std::fs::read_dir(store) else {
    return Vec::new();
  };
  entries
    .flatten()
    .map(|entry| entry.path())
    .filter(|path| path.extension().is_some_and(|extension| extension == "png"))
    .filter_map(|path| {
      let name = path.file_stem()?.to_str()?.to_owned();
      (is_name(&name) && !upscale_decided(&name)).then_some((name, path))
    })
    .collect()
}

/// Keeps `picture`, straight alpha, as the stored picture `name` drawn
/// larger, for the image atlas from now on.
pub fn keep_upscaled(name: &str, picture: &RgbaImage) -> Result<(), String> {
  let folder = folder().ok_or("Pictures cannot be kept yet")?;
  std::fs::create_dir_all(&folder)
    .map_err(|error| format!("The upscaled pictures' folder could not be made: {error}"))?;
  let partial = folder.join(format!("{name}.png.tmp"));
  picture
    .save_with_format(&partial, image::ImageFormat::Png)
    .map_err(|error| error.to_string())
    .and_then(|()| {
      std::fs::rename(&partial, folder.join(format!("{name}.png")))
        .map_err(|error| error.to_string())
    })
    .map_err(|error| format!("The upscaled picture could not be kept: {error}"))?;
  if let Ok(mut held) = held().lock() {
    held.missing.retain(|missing| missing != name);
  }
  Ok(())
}

/// Notes that the stored picture `name` is not to be drawn by the model,
/// and `why`.
pub fn skip_upscale(name: &str, why: &str) -> Result<(), String> {
  let folder = folder().ok_or("Pictures cannot be kept yet")?;
  std::fs::create_dir_all(&folder)
    .map_err(|error| format!("The upscaled pictures' folder could not be made: {error}"))?;
  std::fs::write(folder.join(format!("{name}.skip")), why)
    .map_err(|error| format!("Could not note the picture is not upscaled: {error}"))
}
