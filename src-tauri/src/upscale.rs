// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Small pictures drawn larger with the bundled upscaling model, Real-ESRGAN
//! x4plus anime 6B, which keeps drawn art (cursors, icons, stickers) sharp
//! where stretching blurs it. The helper draws them; each picture is drawn
//! once and kept in the cache under a hash of its pixels, so asking again is
//! a file read. One picture is drawn at a time, as each takes the GPU.

mod pictures;

pub(crate) use pictures::prepare as prepare_pictures;

use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

use screenwide_imaging::CapturedImage;
use screenwide_speech::runner::Transcriber;
use sha2::{Digest, Sha256};
use tauri::{AppHandle, Manager};

/// The upscaling model, bundled as a resource.
const MODEL: &str = "realesrgan-x4plus-anime-6b.onnx";
/// How much larger the model draws a picture, across and down.
pub(crate) const SCALE: u32 = 4;
/// Changes whenever pictures upscaled before would now come out otherwise,
/// so they are drawn again rather than read from the cache.
const CACHE_VERSION: &str = "1";

/// Held while a picture is drawn, so only one is at a time.
static DRAWING: Mutex<()> = Mutex::new(());

/// The model, bundled as a resource, or in the source tree when the app runs
/// unbundled during development.
fn model_path(app: &AppHandle) -> Result<PathBuf, String> {
  let bundled = app
    .path()
    .resource_dir()
    .ok()
    .map(|resources| resources.join("models").join(MODEL));
  let source = Path::new(env!("CARGO_MANIFEST_DIR"))
    .join("assets")
    .join(MODEL);
  bundled
    .into_iter()
    .chain([source])
    .find(|path| path.is_file())
    .ok_or_else(|| "The upscaling model is missing from this installation".to_owned())
}

fn cache_directory(app: &AppHandle) -> Result<PathBuf, String> {
  let directory = app
    .path()
    .app_cache_dir()
    .map_err(|error| error.to_string())?
    .join("upscaled");
  std::fs::create_dir_all(&directory)
    .map_err(|error| format!("Could not make the upscaled pictures' folder: {error}"))?;
  Ok(directory)
}

/// What a picture is kept under: its size and pixels, hashed.
fn key(image: &CapturedImage) -> String {
  let mut hash = Sha256::new();
  hash.update(CACHE_VERSION.as_bytes());
  hash.update(image.width.to_le_bytes());
  hash.update(image.height.to_le_bytes());
  hash.update(&image.rgba);
  hash
    .finalize()
    .iter()
    .map(|byte| format!("{byte:02x}"))
    .collect()
}

/// `image` four times as wide and as high, read from the cache or drawn now,
/// which blocks while the helper draws it.
pub(crate) fn upscaled(app: &AppHandle, image: &CapturedImage) -> Result<CapturedImage, String> {
  let directory = cache_directory(app)?;
  let kept = directory.join(format!("{}.png", key(image)));
  if let Some(found) = read_png(&kept) {
    return Ok(found);
  }
  let _drawing = DRAWING
    .lock()
    .unwrap_or_else(|poisoned| poisoned.into_inner());
  // Another caller may have drawn it while this one waited.
  if let Some(found) = read_png(&kept) {
    return Ok(found);
  }
  let upscaled = draw(app, &directory, image)?;
  if let Ok(png) = screenwide_output::encode_png(&upscaled) {
    let _ = std::fs::write(&kept, png);
  }
  Ok(upscaled)
}

/// `image` four times as wide and as high, drawn now by the helper, without
/// the cache: for a picture its caller keeps itself.
pub(crate) fn upscaled_uncached(
  app: &AppHandle,
  image: &CapturedImage,
) -> Result<CapturedImage, String> {
  let directory = cache_directory(app)?;
  let _drawing = DRAWING
    .lock()
    .unwrap_or_else(|poisoned| poisoned.into_inner());
  draw(app, &directory, image)
}

/// Has the helper draw `image`, passing it through files in `directory`.
/// The caller holds `DRAWING`.
fn draw(app: &AppHandle, directory: &Path, image: &CapturedImage) -> Result<CapturedImage, String> {
  if image.width == 0 || image.height == 0 {
    return Err("There is no picture to upscale".to_owned());
  }
  let model = model_path(app)?;
  let input = directory.join(format!("{}.in.rgba", std::process::id()));
  let output = directory.join(format!("{}.out.rgba", std::process::id()));
  std::fs::File::create(&input)
    .and_then(|mut file| file.write_all(&image.rgba))
    .map_err(|error| format!("Could not hand the picture to the upscaler: {error}"))?;
  let drawn = Transcriber::start()
    .and_then(|mut helper| {
      helper.upscale(
        model,
        (input.clone(), image.width, image.height),
        output.clone(),
        &mut |_| {},
      )
    })
    .and_then(|()| {
      std::fs::read(&output)
        .map_err(|error| format!("Could not read the upscaled picture: {error}"))
    });
  let _ = std::fs::remove_file(&input);
  let _ = std::fs::remove_file(&output);
  let upscaled = CapturedImage {
    width: image.width * SCALE,
    height: image.height * SCALE,
    rgba: drawn?,
  };
  if upscaled.rgba.len() != (upscaled.width * upscaled.height * 4) as usize {
    return Err("The upscaled picture is not the size asked for".to_owned());
  }
  Ok(upscaled)
}

fn read_png(path: &Path) -> Option<CapturedImage> {
  let picture = image::open(path).ok()?.into_rgba8();
  Some(CapturedImage {
    width: picture.width(),
    height: picture.height(),
    rgba: picture.into_raw(),
  })
}

/// Draws the cursors that are too small to draw large four times as large,
/// in the background, for every compositor from its next frame: the
/// system's (a cursor whose vector drawing could not be read, a Windows
/// scheme's small cursor) and apps' own pictures, kept with the recordings
/// opened so far. Asked again while a run is under way, the pictures it
/// already drew are read from the cache.
pub(crate) fn upscale_small_cursors(app: &AppHandle) {
  let small = screenwide_cursor_effects::to_upscale();
  if small.is_empty() {
    return;
  }
  let app = app.clone();
  tauri::async_runtime::spawn_blocking(move || {
    for (style, artwork) in small {
      let picture = CapturedImage {
        width: artwork.width(),
        height: artwork.height(),
        rgba: artwork.into_raw(),
      };
      match upscaled(&app, &picture).and_then(|drawn| {
        image::RgbaImage::from_raw(drawn.width, drawn.height, drawn.rgba)
          .ok_or_else(|| "The upscaled cursor does not fill its size".to_owned())
      }) {
        Ok(drawn) => screenwide_cursor_effects::use_upscaled(style, drawn),
        Err(error) => eprintln!("Could not upscale the {style:?} cursor: {error}"),
      }
    }
  });
}
