// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The stored pictures held decoded between frames, so an image is read
//! from disk once rather than every time the atlas draws it again.

use std::sync::{Arc, Mutex, OnceLock};

use super::super::animation::{decode, ImageAnimation};
use super::super::StoredImage;
use super::{directory, fitted, KEPT_EXTENSIONS};

/// How many bytes of decoded pictures are kept in memory between frames.
const CACHE_BYTES: usize = 256 << 20;

#[derive(Default)]
struct Cache {
  /// Decoded pictures, the most recently used last.
  loaded: Vec<(String, StoredImage)>,
  /// Names with no readable picture, so a missing one is not looked for on
  /// every frame.
  failed: Vec<String>,
}

fn cache() -> &'static Mutex<Cache> {
  static CACHE: OnceLock<Mutex<Cache>> = OnceLock::new();
  CACHE.get_or_init(Default::default)
}

fn bytes(picture: &StoredImage) -> usize {
  match picture {
    StoredImage::Still(image) => image.len(),
    StoredImage::Animation(animation) => animation.bytes(),
  }
}

/// Holds `picture` as `name`'s, the most recently used, dropping the least
/// recently used past the budget; a name that had no picture now has one.
pub(super) fn remember(name: &str, picture: StoredImage) {
  let Ok(mut cache) = cache().lock() else {
    return;
  };
  cache.failed.retain(|failed| failed != name);
  cache.loaded.retain(|(known, _)| known != name);
  cache.loaded.push((name.to_owned(), picture));
  let mut total: usize = cache.loaded.iter().map(|(_, picture)| bytes(picture)).sum();
  while total > CACHE_BYTES && cache.loaded.len() > 1 {
    total -= bytes(&cache.loaded.remove(0).1);
  }
}

/// Lets `name` be looked for again, once a picture has arrived for it.
pub(super) fn forget_failure(name: &str) {
  if let Ok(mut cache) = cache().lock() {
    cache.failed.retain(|failed| failed != name);
  }
}

/// The picture stored under `name`, read from whichever of the kept formats
/// it was kept in: frames for one that moves, else the still, premultiplied.
fn read(name: &str) -> Option<StoredImage> {
  let folder = directory()?;
  KEPT_EXTENSIONS.iter().find_map(|extension| {
    let bytes = std::fs::read(folder.join(format!("{name}.{extension}"))).ok()?;
    if let Some(animation) = decode(&bytes) {
      return Some(StoredImage::Animation(Arc::new(animation)));
    }
    let image = image::load_from_memory(&bytes).ok()?.into_rgba8();
    let (width, height) = image.dimensions();
    Some(StoredImage::Still(Arc::new(fitted(image, width, height))))
  })
}

/// The stored picture `name`, or `None` where there is none.
pub(crate) fn load(name: &str) -> Option<StoredImage> {
  {
    let mut cache = cache().lock().ok()?;
    if let Some(index) = cache.loaded.iter().position(|(known, _)| known == name) {
      let entry = cache.loaded.remove(index);
      let picture = entry.1.clone();
      cache.loaded.push(entry);
      return Some(picture);
    }
    if cache.failed.iter().any(|failed| failed == name) {
      return None;
    }
  }
  // Decoded without the lock: a long animation takes a while, and another
  // image's frame must not wait on it.
  let Some(picture) = read(name) else {
    if let Ok(mut cache) = cache().lock() {
      cache.failed.push(name.to_owned());
    }
    return None;
  };
  remember(name, picture.clone());
  Some(picture)
}

/// Holds a just-kept animation, so its first frames do not decode it again.
pub(super) fn remember_animation(name: &str, animation: ImageAnimation) {
  remember(name, StoredImage::Animation(Arc::new(animation)));
}
