// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The canvas's own background picture, uploaded once and sampled by the
//! preview shader. The CPU compose path fills the picture to the canvas with
//! `screenshots::background_image_canvas`; here the picture keeps its own size
//! on the GPU and the shader does the cover fit, so one upload serves every
//! canvas size, every preview frame, and every exported frame.
//!
//! Nothing here reports an error. A picture that cannot be read or decoded
//! answers `None`, the caller leaves the shader's flag clear, and the solid
//! colour paints instead: a background whose file has moved must never stop a
//! preview or an export.

use std::{
  collections::HashMap,
  sync::{Arc, Mutex},
  time::SystemTime,
};

/// Backgrounds kept resident at once. A workspace composes one picture per
/// layer, so a handful covers a whole compose pass without letting the cache
/// grow with every picture a session ever chose.
const CACHE_ENTRIES: usize = 4;

pub(super) struct BackgroundImage {
  pub(super) view: wgpu::TextureView,
}

/// What the cached upload was made from. A picture edited in place keeps its
/// path, so the file's modification time and length are part of the identity.
#[derive(Clone, Debug, Eq, PartialEq)]
struct Stamp {
  length: u64,
  modified: Option<SystemTime>,
}

struct Entry {
  image: Arc<BackgroundImage>,
  stamp: Stamp,
}

#[derive(Default)]
pub(super) struct BackgroundImageCache {
  entries: Mutex<HashMap<String, Entry>>,
}

fn stamp(path: &str) -> Stamp {
  let metadata = std::fs::metadata(path).ok();
  Stamp {
    length: metadata.as_ref().map_or(0, std::fs::Metadata::len),
    modified: metadata.and_then(|metadata| metadata.modified().ok()),
  }
}

fn upload(gpu: &crate::gpu::Gpu, path: &str) -> Option<BackgroundImage> {
  let pixels = image::open(path).ok()?.into_rgba8();
  let (width, height) = pixels.dimensions();
  // A picture larger than the device takes is treated as unreadable, which
  // paints the solid colour rather than a device error.
  let largest = gpu.device.limits().max_texture_dimension_2d;
  if width == 0 || height == 0 || width > largest || height > largest {
    return None;
  }
  let texture = gpu.texture_with_pixels(
    "Screenwide background picture",
    (width, height, 1),
    wgpu::TextureFormat::Rgba8Unorm,
    pixels.as_raw(),
  );
  Some(BackgroundImage {
    view: texture.create_view(&Default::default()),
  })
}

impl BackgroundImageCache {
  /// The uploaded picture behind this canvas, decoded at its own size. The
  /// upload is reused until the path changes or the file behind it does.
  pub(super) fn resolve(&self, gpu: &crate::gpu::Gpu, path: &str) -> Option<Arc<BackgroundImage>> {
    let stamp = stamp(path);
    let mut entries = self.entries.lock().ok()?;
    if let Some(entry) = entries.get(path) {
      if entry.stamp == stamp {
        return Some(Arc::clone(&entry.image));
      }
    }
    let image = Arc::new(upload(gpu, path)?);
    if entries.len() >= CACHE_ENTRIES {
      entries.clear();
    }
    entries.insert(
      path.to_owned(),
      Entry {
        image: Arc::clone(&image),
        stamp,
      },
    );
    Some(image)
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn a_picture_that_is_not_there_has_an_empty_stamp() {
    assert_eq!(
      stamp("/no/such/background.png"),
      Stamp {
        length: 0,
        modified: None,
      }
    );
  }
}
