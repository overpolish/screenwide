// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The pictures people give stickers: an image file chosen, or one pasted
//! or dropped.
//!
//! Each is kept once in the app's data folder, named by a hash of its
//! contents, so a sticker carries only `image:` and that name, and the same
//! picture pasted twice is kept once. A still is kept as a PNG, shrunk as it
//! arrives where it is larger than any sticker is drawn; a moving one is kept
//! as the file it came as, since no still could hold it. Nothing outside the
//! app points at these files, so each launch removes the ones nothing still
//! names; see [`sweep`].

use std::path::{Path, PathBuf};
use std::sync::OnceLock;

use image::{imageops::FilterType, DynamicImage, RgbaImage};

use super::animation::{decode, extension, StickerAnimation};
use crate::editor::annotations::sticker::{StickerArt, StickerPlay};

mod cache;
pub(crate) use cache::load;
mod sweep;
use sweep::sweep;

/// The prefix a stored picture's asset id carries.
pub(crate) const IMAGE_PREFIX: &str = "image:";
/// How many hex digits a stored picture's name has.
const NAME_DIGITS: usize = 32;
/// The formats pictures are kept in: a still as a PNG, a moving one as
/// whichever of these it came as.
const KEPT_EXTENSIONS: [&str; 3] = ["png", "gif", "webp"];
/// The longest side a still is kept at: the most any sticker is drawn at,
/// `MAX_RASTER` in the sticker atlas.
const MAX_SIDE: u32 = 2_048;

static DIRECTORY: OnceLock<PathBuf> = OnceLock::new();

/// Where stored pictures live, once the app has said.
fn directory() -> Option<&'static Path> {
  DIRECTORY.get().map(PathBuf::as_path)
}

/// Settles where pictures are stored, and clears out every one that no
/// edit in `recordings` still names. Runs at launch, before any editor
/// opens, so nothing on screen can be showing a picture it removes.
pub(crate) fn initialize(data: &Path, recordings: Option<&Path>) {
  let folder = data.join("stickers");
  let _ = DIRECTORY.set(folder.clone());
  sweep(&folder, recordings);
}

/// Whether `name` could be one this store gave out.
pub(crate) fn is_name(name: &str) -> bool {
  name.len() == NAME_DIGITS
    && name
      .bytes()
      .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

/// A stable 128-bit FNV-1a of `bytes`, as the store's name for them. The
/// name only has to tell pictures apart, not stand up to anyone forging one.
fn content_name(bytes: &[u8]) -> String {
  let mut hash: u128 = 0x6c62_272e_07bb_0142_62b8_2175_6295_c58d;
  for byte in bytes {
    hash ^= u128::from(*byte);
    hash = hash.wrapping_mul(0x0000_0000_0100_0000_0000_0000_0000_013b);
  }
  format!("{hash:032x}")
}

/// `image` with its colour scaled by its alpha, or back, so a resize blends
/// neighbours by how much of each is there rather than letting a clear
/// pixel's hidden colour bleed into the edge.
fn premultiply(image: &mut RgbaImage, into: bool) {
  for pixel in image.pixels_mut() {
    let alpha = u16::from(pixel[3]);
    for channel in &mut pixel.0[..3] {
      let value = u16::from(*channel);
      *channel = if into {
        ((value * alpha + 127) / 255) as u8
      } else {
        (value * 255 + alpha / 2)
          .checked_div(alpha)
          .map_or(0, |straight| straight.min(255) as u8)
      };
    }
  }
}

/// `image` premultiplied and drawn at `width` by `height`.
pub(crate) fn fitted(mut image: RgbaImage, width: u32, height: u32) -> RgbaImage {
  premultiply(&mut image, true);
  if image.width() == width && image.height() == height {
    return image;
  }
  resized(&image, width, height)
}

/// A premultiplied `image` drawn at `width` by `height`.
pub(crate) fn resized(image: &RgbaImage, width: u32, height: u32) -> RgbaImage {
  if image.width() == width && image.height() == height {
    return image.clone();
  }
  image::imageops::resize(image, width.max(1), height.max(1), FilterType::Triangle)
}

/// Writes `bytes` as `name`'s file in `extension`, unless it is there
/// already: the name is the content's own, so an existing file is the same.
fn keep(name: &str, extension: &str, bytes: &[u8]) -> Result<(), String> {
  let folder = directory().ok_or("Pictures cannot be kept yet")?;
  let path = folder.join(format!("{name}.{extension}"));
  if path.is_file() {
    return Ok(());
  }
  std::fs::create_dir_all(folder)
    .map_err(|error| format!("The sticker folder could not be made: {error}"))?;
  let partial = folder.join(format!("{name}.{extension}.tmp"));
  std::fs::write(&partial, bytes)
    .and_then(|()| std::fs::rename(&partial, &path))
    .map_err(|error| format!("The picture could not be kept: {error}"))
}

/// Keeps `image` and says what a sticker showing it carries: its own size
/// and proportions, which the kept copy may be smaller than.
pub(crate) fn import(image: DynamicImage) -> Result<StickerArt, String> {
  let image = image.into_rgba8();
  let (original_width, original_height) = image.dimensions();
  if original_width == 0 || original_height == 0 {
    return Err("That picture is empty".to_owned());
  }
  let longer = original_width.max(original_height);
  let scale = (f64::from(MAX_SIDE) / f64::from(longer)).min(1.0);
  let (width, height) = (
    ((f64::from(original_width) * scale).round() as u32).max(1),
    ((f64::from(original_height) * scale).round() as u32).max(1),
  );
  let mut image = fitted(image, width, height);
  premultiply(&mut image, false);
  let mut png = Vec::new();
  DynamicImage::ImageRgba8(image)
    .write_to(&mut std::io::Cursor::new(&mut png), image::ImageFormat::Png)
    .map_err(|error| format!("The picture could not be kept: {error}"))?;
  let name = content_name(&png);
  keep(&name, "png", &png)?;
  Ok(StickerArt {
    asset: format!("{IMAGE_PREFIX}{name}"),
    aspect: f64::from(original_width) / f64::from(original_height),
    pixels: Some(f64::from(longer)),
    play: None,
  })
}

/// Keeps the moving picture in `bytes` as the file it is, since no still
/// could hold it, and says what a sticker showing it carries: its own size
/// and proportions, and how long a run of it lasts.
fn import_animation(bytes: &[u8], animation: StickerAnimation) -> Result<StickerArt, String> {
  let (width, height) = animation.canvas;
  if width == 0 || height == 0 {
    return Err("That picture is empty".to_owned());
  }
  let extension = image::guess_format(bytes)
    .ok()
    .and_then(extension)
    .ok_or("That picture could not be read")?;
  let name = content_name(bytes);
  keep(&name, extension, bytes)?;
  let play = StickerPlay {
    cycle_ms: f64::from(animation.cycle_ms),
    frames: animation.frames.len() as u32,
    frame: 0,
    once: false,
    clock_ms: None,
  };
  cache::remember_animation(&name, animation);
  Ok(StickerArt {
    asset: format!("{IMAGE_PREFIX}{name}"),
    aspect: f64::from(width) / f64::from(height),
    pixels: Some(f64::from(width.max(height))),
    play: Some(play),
  })
}

/// Reads the picture in `bytes`, in whatever format they say they are: a
/// picture's own file data, as a pasteboard or a drag hands it over. One
/// that moves is kept moving.
pub(crate) fn import_bytes(bytes: &[u8]) -> Result<StickerArt, String> {
  if let Some(animation) = decode(bytes) {
    return import_animation(bytes, animation);
  }
  let image = image::load_from_memory(bytes)
    .map_err(|error| format!("The picture could not be read: {error}"))?;
  import(image)
}

/// Reads the picture at `path`.
pub(crate) fn import_file(path: &Path) -> Result<StickerArt, String> {
  let bytes =
    std::fs::read(path).map_err(|error| format!("That picture could not be opened: {error}"))?;
  import_bytes(&bytes)
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn a_shrunk_picture_keeps_its_proportions_and_its_clear_edge() {
    // A half-clear red square on a clear field: the blend at its edge must
    // stay red, not darken towards the clear pixels' hidden black.
    let mut image = RgbaImage::from_pixel(8, 4, image::Rgba([0, 0, 0, 0]));
    for x in 0..4 {
      for y in 0..4 {
        image.put_pixel(x, y, image::Rgba([255, 0, 0, 255]));
      }
    }
    let shrunk = fitted(image, 4, 2);
    assert_eq!(shrunk.dimensions(), (4, 2));
    let edge = shrunk.get_pixel(1, 0);
    assert!(
      edge[3] > 0 && u16::from(edge[0]) + 2 >= u16::from(edge[3]),
      "{edge:?}"
    );
  }

  #[test]
  fn only_a_full_lowercase_hash_is_a_name() {
    assert!(is_name("0123456789abcdef0123456789abcdef"));
    for refused in [
      "",
      "0123",
      "0123456789ABCDEF0123456789ABCDEF",
      "../../../../etc/passwd/xx",
    ] {
      assert!(!is_name(refused), "{refused}");
    }
  }
}
