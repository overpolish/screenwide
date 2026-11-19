// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: Elastic-2.0

//! Cues: pictures pinned to words of a script, shown while it is read so the
//! reader sees what they are about to talk about. Each is a still of a
//! recording at a moment, as its edit shows it, or a screenshot as edited,
//! kept as a picture of its own beside the script: a recording renamed,
//! moved or put away leaves its cues as they were. The script's words stay
//! plain text; the cues are kept beside them, each by where its word starts,
//! which the teleprompter moves with every edit and saves here.

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter, Manager};
use ts_rs::TS;

use super::window::LABEL;

/// The cues of a script, in its project's folder.
const CUES_FILE: &str = "cues.json";
/// Their pictures, in its project's folder.
const PICTURES: &str = "cues";
/// How wide a cue's picture is kept, in pixels: the cue's window never
/// grows past its picture, so this is as wide as it can be shown sharp.
const PICTURE_WIDTH: u32 = 1920;
/// How wide a moment is drawn while it is chosen: sharp across the picker
/// on a Retina display.
const PREVIEW_WIDTH: u32 = 800;
const JPEG_QUALITY: u8 = 85;
/// Tells the teleprompter a cue was made, for it to pin and keep.
const MADE_EVENT: &str = "teleprompter://cue";

/// A picture pinned to a word of a script.
#[derive(Clone, Debug, Deserialize, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, rename = "TeleprompterCue")]
pub struct Cue {
  pub id: String,
  /// Where its word starts in the script, in UTF-16 units, as the page
  /// counts them.
  pub at: u32,
  /// Its picture's file name, among the script's cue pictures.
  pub image: String,
  /// Its project's name when it was made.
  pub title: String,
  /// Where in what plays it was taken, in ms; none for a screenshot.
  pub played_ms: Option<u64>,
  /// The project it was taken from, where it was then.
  pub source: PathBuf,
}

/// A cue with its picture's place, for the page to show.
#[derive(Clone, Debug, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, rename = "ShownTeleprompterCue")]
pub struct ShownCue {
  #[serde(flatten)]
  pub cue: Cue,
  pub path: PathBuf,
}

fn folder(script: &Path) -> Result<&Path, String> {
  script
    .parent()
    .ok_or_else(|| "The script has no folder".to_owned())
}

/// `cue` with its picture shown, let through the asset protocol.
fn shown(app: &AppHandle, script: &Path, cue: Cue) -> Result<ShownCue, String> {
  let path = folder(script)?.join(PICTURES).join(&cue.image);
  app
    .asset_protocol_scope()
    .allow_file(&path)
    .map_err(|error| error.to_string())?;
  Ok(ShownCue { cue, path })
}

/// The cues of the script project whose manifest is `project`, none before
/// its first. Pictures no cue holds any more, left by cues taken out, are
/// cleared away as the script is opened, when nothing else is changing them.
#[tauri::command]
pub fn read_cues(app: AppHandle, project: PathBuf) -> Result<Vec<ShownCue>, String> {
  let file = folder(&project)?.join(CUES_FILE);
  let cues: Vec<Cue> = match std::fs::read(&file) {
    Ok(bytes) => serde_json::from_slice(&bytes).map_err(|error| error.to_string())?,
    Err(error) if error.kind() == std::io::ErrorKind::NotFound => Vec::new(),
    Err(error) => return Err(error.to_string()),
  };
  clear_unheld(&project, &cues);
  cues
    .into_iter()
    .map(|cue| shown(&app, &project, cue))
    .collect()
}

fn clear_unheld(script: &Path, cues: &[Cue]) {
  let Ok(pictures) = folder(script).map(|folder| folder.join(PICTURES)) else {
    return;
  };
  let Ok(entries) = std::fs::read_dir(pictures) else {
    return;
  };
  for entry in entries.flatten() {
    let name = entry.file_name();
    if !cues.iter().any(|cue| name == cue.image.as_str()) {
      let _ = std::fs::remove_file(entry.path());
    }
  }
}

/// Keeps `cues` as the cues of the script project whose manifest is
/// `project`, whole or not at all.
#[tauri::command]
pub fn save_cues(project: PathBuf, cues: Vec<Cue>) -> Result<(), String> {
  let file = folder(&project)?.join(CUES_FILE);
  let json = serde_json::to_vec_pretty(&cues).map_err(|error| error.to_string())?;
  let mut writing = file.clone().into_os_string();
  writing.push(".part");
  std::fs::write(&writing, json).map_err(|error| error.to_string())?;
  std::fs::rename(writing, &file).map_err(|error| error.to_string())
}

/// A name no other cue picture has.
fn next_id() -> String {
  static SEQUENCE: AtomicU64 = AtomicU64::new(0);
  let sequence = SEQUENCE.fetch_add(1, Ordering::Relaxed);
  let millis = chrono::Utc::now().timestamp_millis();
  format!("cue-{millis:x}-{sequence:x}")
}

/// Writes `frame` as a JPEG at `target`, laid over black where it is
/// see-through, as the project's own pictures are.
fn write_jpeg(frame: screenwide_imaging::CapturedImage, target: &Path) -> Result<(), String> {
  let picture = image::RgbaImage::from_raw(frame.width, frame.height, frame.rgba)
    .ok_or_else(|| "A composed frame is not the size it says".to_owned())?;
  let over_black = image::RgbImage::from_fn(frame.width, frame.height, |x, y| {
    let [red, green, blue, alpha] = picture.get_pixel(x, y).0;
    let mix = |channel: u8| ((u16::from(channel) * u16::from(alpha)) / 255) as u8;
    image::Rgb([mix(red), mix(green), mix(blue)])
  });
  let mut writing = target.to_path_buf().into_os_string();
  writing.push(".part");
  let file = std::fs::File::create(&writing).map_err(|error| error.to_string())?;
  image::codecs::jpeg::JpegEncoder::new_with_quality(std::io::BufWriter::new(file), JPEG_QUALITY)
    .encode_image(&over_black)
    .map_err(|error| error.to_string())?;
  std::fs::rename(writing, target).map_err(|error| error.to_string())
}

/// The moment `source_ms` into the recording of `source`, as its edit shows
/// it, small, for choosing a moment by: its width and height as two
/// little-endian 32-bit numbers, then its pixels as RGBA laid over black.
/// Sent as they are rather than as a picture file, which took longer to
/// encode, write and read back than the frame took to draw.
#[tauri::command]
pub async fn cue_preview(
  app: AppHandle,
  source: PathBuf,
  source_ms: u64,
) -> Result<tauri::ipc::Response, String> {
  let frame = crate::editor::recording_preview_player::project_frames::compose_project_frame(
    &app,
    &source,
    source_ms,
    PREVIEW_WIDTH,
  )
  .await?;
  let mut bytes = Vec::with_capacity(8 + frame.rgba.len());
  bytes.extend_from_slice(&frame.width.to_le_bytes());
  bytes.extend_from_slice(&frame.height.to_le_bytes());
  for &[red, green, blue, alpha] in frame.rgba.as_chunks::<4>().0 {
    let mix = |channel: u8| ((u16::from(channel) * u16::from(alpha)) / 255) as u8;
    bytes.extend_from_slice(&[mix(red), mix(green), mix(blue), 255]);
  }
  Ok(tauri::ipc::Response::new(bytes))
}

/// Makes a cue of `source` for the script project whose manifest is
/// `script`, pinned to the word starting `at`, and tells the teleprompter,
/// which keeps it with the rest. A recording's is its frame `source_ms`
/// into it, as its edit shows it, `played_ms` into what plays; a
/// screenshot's is its picture as edited.
#[tauri::command]
pub async fn add_cue(
  app: AppHandle,
  script: PathBuf,
  at: u32,
  source: PathBuf,
  timing: Option<CueTiming>,
) -> Result<(), String> {
  let pictures = folder(&script)?.join(PICTURES);
  std::fs::create_dir_all(&pictures).map_err(|error| error.to_string())?;
  let id = next_id();
  let image = format!("{id}.jpg");
  let target = pictures.join(&image);
  match timing {
    Some(timing) => {
      let frame = crate::editor::recording_preview_player::project_frames::compose_project_frame(
        &app,
        &source,
        timing.source_ms,
        PICTURE_WIDTH,
      )
      .await?;
      tauri::async_runtime::spawn_blocking(move || write_jpeg(frame, &target))
        .await
        .map_err(|error| error.to_string())??;
    }
    None => {
      // The screenshot as edited, kept large as its editor last saved it.
      // One not edited since that was kept has only its preview, or its
      // own picture where it was never edited.
      let large = screenwide_project::still_path(&source);
      if large.is_file() {
        std::fs::copy(&large, &target).map_err(|error| error.to_string())?;
      } else {
        let still = crate::editor::project_thumbnail(&source)?
          .ok_or_else(|| "The screenshot has no picture".to_owned())?;
        let picture = image::open(&still)
          .map_err(|error| error.to_string())?
          .into_rgba8();
        let frame = screenwide_imaging::CapturedImage {
          width: picture.width(),
          height: picture.height(),
          rgba: picture.into_raw(),
        };
        tauri::async_runtime::spawn_blocking(move || {
          write_jpeg(
            screenwide_export::still_size::shrunk(frame, PICTURE_WIDTH)?,
            &target,
          )
        })
        .await
        .map_err(|error| error.to_string())??;
      }
    }
  }
  let cue = Cue {
    id,
    at,
    image,
    title: screenwide_project::summarize(&source).title,
    played_ms: timing.map(|timing| timing.played_ms),
    source,
  };
  let made = shown(&app, &script, cue)?;
  app
    .emit_to(LABEL, MADE_EVENT, &made)
    .map_err(|error| error.to_string())
}

/// Where a recording's cue was taken: in the recording, and in what plays.
#[derive(Clone, Copy, Debug, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, rename = "TeleprompterCueTiming")]
pub struct CueTiming {
  pub played_ms: u64,
  pub source_ms: u64,
}
