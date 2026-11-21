// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The pictures of apps' own cursors, kept with the recordings that show
//! them (`screenwide_records::cursor::picture_path`), drawn where the
//! recording shows them instead of the system arrow.
//!
//! Each picture opened is given a slot in the GPU artwork after the system
//! styles and the custom slot ([`GPU_CUSTOM_ARTWORK_INDEX`]), for the life of
//! the process, so a compositor made before or after draws it from the same
//! index. Pictures are small (an app's cursor at its own size), so the app
//! draws each larger with its upscaling model (`to_upscale`,
//! `use_upscaled`), as it does the system's small artwork.

use std::path::Path;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, RwLock};

use image::RgbaImage;

use super::GPU_CUSTOM_ARTWORK_INDEX;

/// How many pictures the process keeps slots for. A recording shows a
/// handful; past this a picture is drawn as the arrow, as before pictures
/// were kept.
const MOST: usize = 64;
/// Pictures no longer than this are drawn larger: an app's cursor at its
/// own size, or twice it on a retina display, which blurs at the larger
/// pointer sizes.
const UPSCALE_UP_TO: u32 = 128;

/// One picture: its name in the recording and how it is drawn, the
/// picture as kept, or drawn larger once the app has.
pub(super) struct Picture {
  name: String,
  pub(super) image: Arc<RgbaImage>,
  upscaled: bool,
}

static PICTURES: RwLock<Vec<Picture>> = RwLock::new(Vec::new());
/// How many pictures have been added or drawn larger, which the GPU set is
/// made again for.
static GENERATION: AtomicU64 = AtomicU64::new(0);

pub(super) fn generation() -> u64 {
  GENERATION.load(Ordering::Acquire)
}

/// The artwork index of the picture `name`, read from `file` the first time
/// it is asked for; `None` where it cannot be read or there is no slot left.
pub(super) fn artwork_index(name: &str, file: &Path) -> Option<u32> {
  let slot = |pictures: &[Picture]| pictures.iter().position(|picture| picture.name == name);
  if let Some(index) = slot(&read()) {
    return Some(index_of(index));
  }
  let image = image::open(file).ok()?.into_rgba8();
  if image.width() == 0 || image.height() == 0 {
    return None;
  }
  let mut pictures = PICTURES
    .write()
    .unwrap_or_else(|poisoned| poisoned.into_inner());
  // Another opening may have read it meanwhile.
  if let Some(index) = slot(&pictures) {
    return Some(index_of(index));
  }
  if pictures.len() >= MOST {
    return None;
  }
  pictures.push(Picture {
    name: name.to_owned(),
    image: Arc::new(image),
    upscaled: false,
  });
  GENERATION.fetch_add(1, Ordering::AcqRel);
  Some(index_of(pictures.len() - 1))
}

fn index_of(slot: usize) -> u32 {
  GPU_CUSTOM_ARTWORK_INDEX + 1 + slot as u32
}

pub(super) fn read() -> std::sync::RwLockReadGuard<'static, Vec<Picture>> {
  PICTURES
    .read()
    .unwrap_or_else(|poisoned| poisoned.into_inner())
}

/// The pictures small enough to draw larger and not drawn larger yet, by
/// name.
pub(super) fn to_upscale() -> Vec<(String, RgbaImage)> {
  read()
    .iter()
    .filter(|picture| {
      !picture.upscaled && picture.image.width().max(picture.image.height()) <= UPSCALE_UP_TO
    })
    .map(|picture| (picture.name.clone(), (*picture.image).clone()))
    .collect()
}

/// Draws the picture `name` from `image` from now on: the same picture
/// drawn larger, at the same proportions.
pub(super) fn use_upscaled(name: &str, image: RgbaImage) {
  let mut pictures = PICTURES
    .write()
    .unwrap_or_else(|poisoned| poisoned.into_inner());
  let Some(picture) = pictures.iter_mut().find(|picture| picture.name == name) else {
    return;
  };
  if picture.upscaled || image.width() <= picture.image.width() {
    return;
  }
  picture.image = Arc::new(image);
  picture.upscaled = true;
  GENERATION.fetch_add(1, Ordering::AcqRel);
}
