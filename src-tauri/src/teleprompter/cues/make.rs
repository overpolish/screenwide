// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: Elastic-2.0

//! Making cues: from the teleprompter's picker, at a word chosen there, and
//! from an editor, waiting in the teleprompter for a word to be picked.

use std::path::PathBuf;
use std::sync::{Mutex, PoisonError};

use serde::Serialize;
use tauri::{AppHandle, Emitter, Manager};

use super::picture::{draw, picture_name};
use super::{folder, shown, Cue, CueTiming, PICTURES};
use crate::teleprompter::window::LABEL;

/// How wide a moment is drawn while it is chosen: sharp across the picker
/// on a Retina display.
const PREVIEW_WIDTH: u32 = 800;
/// Tells the teleprompter a cue was made, for it to pin and keep.
const MADE_EVENT: &str = "teleprompter://cue";
/// Tells the teleprompter a cue from an editor is waiting for its word.
const REQUEST_EVENT: &str = "teleprompter://cue-request";

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

/// A cue of `source` for the script `script`, pinned to the word starting
/// `at`, its picture drawn: a recording's frame at `timing`, a screenshot's
/// as edited.
async fn made(
  app: &AppHandle,
  script: &std::path::Path,
  at: u32,
  source: PathBuf,
  timing: Option<CueTiming>,
) -> Result<Cue, String> {
  let pictures = folder(script)?.join(PICTURES);
  std::fs::create_dir_all(&pictures).map_err(|error| error.to_string())?;
  let id = picture_name();
  let image = format!("{id}.jpg");
  let source_ms = timing.map(|timing| timing.source_ms);
  draw(app, &source, source_ms, pictures.join(&image)).await?;
  Ok(Cue {
    id,
    at,
    image,
    title: screenwide_project::summarize(&source).title,
    played_ms: timing.map(|timing| timing.played_ms),
    source_ms,
    source,
  })
}

/// Makes a cue of `source` for the script project whose manifest is
/// `script`, pinned to the word starting `at`, and tells the teleprompter,
/// which keeps it with the rest.
#[tauri::command]
pub async fn add_cue(
  app: AppHandle,
  script: PathBuf,
  at: u32,
  source: PathBuf,
  timing: Option<CueTiming>,
) -> Result<(), String> {
  let cue = made(&app, &script, at, source, timing).await?;
  let made = shown(&app, &script, cue)?;
  app
    .emit_to(LABEL, MADE_EVENT, &made)
    .map_err(|error| error.to_string())
}

/// An editor's project, and where in it, waiting for the teleprompter to
/// have its word picked.
#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Request {
  source: PathBuf,
  timing: Option<CueTiming>,
}

/// The cue an editor asked for that the teleprompter has not taken yet. It
/// waits here rather than in an event, which a teleprompter still opening,
/// or opening another script, would miss.
static WAITING: Mutex<Option<Request>> = Mutex::new(None);

/// Keeps `request` for the teleprompter and brings it forward, so the
/// pointer over its words is followed and the first click picks one.
fn wait_for_word(app: &AppHandle, request: Request) -> Result<(), String> {
  *WAITING.lock().unwrap_or_else(PoisonError::into_inner) = Some(request);
  if let Some(window) = app.get_webview_window(LABEL) {
    window.set_focus().map_err(|error| error.to_string())?;
  }
  app
    .emit_to(LABEL, REQUEST_EVENT, ())
    .map_err(|error| error.to_string())
}

/// The project an editor has open as `artifact_id`, which a cue it asks for
/// is made of.
#[tauri::command]
pub fn editor_cue_source(
  app: AppHandle,
  editor: crate::editor::EditorKind,
  artifact_id: u64,
) -> Result<PathBuf, String> {
  crate::editor::open_editor_project(&app, editor, artifact_id)
    .ok_or_else(|| "That project is no longer open in the editor".to_owned())
}

/// Asks the open teleprompter to have a word picked for a cue of the
/// editor's project `source`: the frame at `timing` for a recording, the
/// screenshot as edited without one. False when no teleprompter is open,
/// for the editor to ask which script the cue is for.
#[tauri::command]
pub fn request_cue(
  app: AppHandle,
  source: PathBuf,
  timing: Option<CueTiming>,
) -> Result<bool, String> {
  if app.get_webview_window(LABEL).is_none() {
    return Ok(false);
  }
  wait_for_word(&app, Request { source, timing })?;
  Ok(true)
}

/// Opens the teleprompter on the script project whose manifest is `script`,
/// to have a word picked there for a cue of `source`.
#[tauri::command]
pub fn place_cue_in_script(
  app: AppHandle,
  script: PathBuf,
  source: PathBuf,
  timing: Option<CueTiming>,
) -> Result<(), String> {
  if !crate::licensing::allow(&app, crate::licensing::Tool::Teleprompter) {
    return Ok(());
  }
  wait_for_word(&app, Request { source, timing })?;
  crate::teleprompter::open(&app, &script)
}

/// The cue waiting for its word, taken by the teleprompter to place.
#[tauri::command]
pub fn take_cue_request() -> Option<Request> {
  WAITING
    .lock()
    .unwrap_or_else(PoisonError::into_inner)
    .take()
}
