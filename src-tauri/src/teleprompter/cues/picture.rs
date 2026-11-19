// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: Elastic-2.0

//! A cue's picture: a recording's frame as its edit shows it, or a
//! screenshot as edited, kept as a JPEG among the script's cue pictures.

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use tauri::AppHandle;

use screenwide_imaging::CapturedImage;

/// How wide a cue's picture is kept, in pixels: the cue's window never
/// grows past its picture, so this is as wide as it can be shown sharp.
const PICTURE_WIDTH: u32 = 1920;
const JPEG_QUALITY: u8 = 85;

/// A name no other cue picture has, for a cue new or redrawn.
pub(super) fn picture_name() -> String {
  static SEQUENCE: AtomicU64 = AtomicU64::new(0);
  let sequence = SEQUENCE.fetch_add(1, Ordering::Relaxed);
  let millis = chrono::Utc::now().timestamp_millis();
  format!("cue-{millis:x}-{sequence:x}")
}

/// Draws the picture of `source` at `target`: the frame `source_ms` into a
/// recording, or a screenshot without one.
pub(super) async fn draw(
  app: &AppHandle,
  source: &Path,
  source_ms: Option<u64>,
  target: PathBuf,
) -> Result<(), String> {
  let frame = match source_ms {
    Some(source_ms) => {
      crate::editor::recording_preview_player::project_frames::compose_project_frame(
        app,
        source,
        source_ms,
        PICTURE_WIDTH,
      )
      .await?
    }
    None => {
      // The screenshot as edited, kept large as its editor last saved it.
      let large = screenwide_project::still_path(source);
      if large.is_file() {
        return std::fs::copy(&large, &target)
          .map(|_| ())
          .map_err(|error| error.to_string());
      }
      // One not edited since that was kept has only its preview, or its
      // own picture where it was never edited.
      let still = crate::editor::project_thumbnail(source)?
        .ok_or_else(|| "The screenshot has no picture".to_owned())?;
      let picture = image::open(&still)
        .map_err(|error| error.to_string())?
        .into_rgba8();
      CapturedImage {
        width: picture.width(),
        height: picture.height(),
        rgba: picture.into_raw(),
      }
    }
  };
  tauri::async_runtime::spawn_blocking(move || write(frame, &target))
    .await
    .map_err(|error| error.to_string())?
}

/// Writes `frame`, at most `PICTURE_WIDTH` wide, as a JPEG at `target`,
/// whole or not at all.
fn write(frame: CapturedImage, target: &Path) -> Result<(), String> {
  let frame = screenwide_export::still_size::shrunk(frame, PICTURE_WIDTH)?;
  let mut writing = target.to_path_buf().into_os_string();
  writing.push(".part");
  let file = std::fs::File::create(&writing).map_err(|error| error.to_string())?;
  screenwide_export::still_size::write_jpeg(
    &frame,
    JPEG_QUALITY,
    &mut std::io::BufWriter::new(file),
  )?;
  std::fs::rename(writing, target).map_err(|error| error.to_string())
}

/// When `path` was last written, if it can be read.
pub(super) fn modified(path: &Path) -> Option<std::time::SystemTime> {
  std::fs::metadata(path)
    .and_then(|metadata| metadata.modified())
    .ok()
}
