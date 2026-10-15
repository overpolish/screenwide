// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The pictures behind the person, one for each range that shows one, kept
//! in the project so the project carries them when it moves. Each is kept as
//! a JPEG whatever format it was chosen in: the FFmpeg bundled on macOS is
//! built without a PNG decoder, and reads JPEG through its MJPEG one.

use std::collections::HashMap;
use std::io::BufWriter;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::SystemTime;

/// The longest side kept: no camera is drawn larger, enhanced or not.
const LONGEST_SIDE: u32 = 3_840;
/// Near enough lossless for a picture that is only ever a backdrop.
const QUALITY: u8 = 95;
const PREFIX: &str = "camera-background-";
const EXTENSION: &str = "jpg";

/// The picture of the range `id`. Range ids are checked with the edit to
/// hold only letters, digits, `-` and `_`, so the name stays in `folder`.
pub(super) fn path(folder: &Path, id: &str) -> PathBuf {
  folder.join(format!("{PREFIX}{id}.{EXTENSION}"))
}

/// The one picture a project kept before backgrounds had ranges.
pub(super) fn legacy_path(folder: &Path) -> PathBuf {
  folder.join("camera-background.jpg")
}

/// The ids of the ranges with a picture in `folder`.
pub(super) fn chosen(folder: &Path) -> Vec<String> {
  let Ok(entries) = std::fs::read_dir(folder) else {
    return Vec::new();
  };
  let mut ids: Vec<String> = entries
    .flatten()
    .filter_map(|entry| {
      let name = entry.file_name();
      let name = name.to_str()?;
      let id = name
        .strip_prefix(PREFIX)?
        .strip_suffix(EXTENSION)?
        .strip_suffix('.')?;
      // A picture still being written is named `<id>.partial.jpg`.
      (!id.is_empty() && !id.contains('.')).then(|| id.to_owned())
    })
    .collect();
  ids.sort();
  ids
}

/// Keeps the picture at `chosen` in `folder` as the range `id`'s, in place
/// of any before it.
pub(super) fn import(chosen: &Path, folder: &Path, id: &str) -> Result<(), String> {
  let picture =
    ::image::open(chosen).map_err(|error| format!("Could not read that picture: {error}"))?;
  let picture = if picture.width().max(picture.height()) > LONGEST_SIDE {
    picture.resize(
      LONGEST_SIDE,
      LONGEST_SIDE,
      ::image::imageops::FilterType::Lanczos3,
    )
  } else {
    picture
  };
  let unwritable =
    |error: &dyn std::fmt::Display| format!("Could not keep the background picture: {error}");
  let destination = path(folder, id);
  let partial = destination.with_extension("partial.jpg");
  let file = std::fs::File::create(&partial).map_err(|error| unwritable(&error))?;
  ::image::codecs::jpeg::JpegEncoder::new_with_quality(BufWriter::new(file), QUALITY)
    .encode_image(&picture.to_rgb8())
    .map_err(|error| unwritable(&error))?;
  std::fs::rename(&partial, &destination).map_err(|error| unwritable(&error))
}

type Decoded = HashMap<PathBuf, (SystemTime, Arc<::image::RgbaImage>)>;

/// Pictures decoded for the effects last loaded, so a range edit that leaves
/// a picture as it was does not decode it again.
static DECODED: Mutex<Option<Decoded>> = Mutex::new(None);

/// The pictures at `paths`, each decoded once while it is unchanged, or
/// nothing for one that cannot be read. Pictures not asked for are let go.
pub(super) fn decoded(paths: &[PathBuf]) -> Vec<Option<Arc<::image::RgbaImage>>> {
  let mut held = DECODED
    .lock()
    .unwrap_or_else(|poisoned| poisoned.into_inner());
  let before = held.take().unwrap_or_default();
  let mut now = Decoded::new();
  let pictures = paths
    .iter()
    .map(|path| {
      let modified = std::fs::metadata(path)
        .and_then(|metadata| metadata.modified())
        .ok()?;
      let picture = match before.get(path).or_else(|| now.get(path)) {
        Some((at, picture)) if *at == modified => Arc::clone(picture),
        _ => Arc::new(::image::open(path).ok()?.to_rgba8()),
      };
      now.insert(path.clone(), (modified, Arc::clone(&picture)));
      Some(picture)
    })
    .collect();
  *held = Some(now);
  pictures
}
