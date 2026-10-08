// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! When the editor's pictures of its recording are made: once its edit has
//! rested, and, when the window closes with some still owed, before the
//! recording is let go, so a change made just before closing is never left
//! out of them.

use std::sync::{LazyLock, Mutex, MutexGuard};
use std::time::Duration;

use super::*;
use crate::editor::recording_preview_player::composed_frames::{
  compose_preview_frames, PreviewFrameRequest,
};

/// How long the edit rests before its pictures are made. The editor waits a
/// moment of its own before saying, so this is most of the wait, not all.
const SETTLE: Duration = Duration::from_millis(750);

/// Pictures asked for and not yet made: the newest edit's.
struct Owed {
  request: PreviewFrameRequest,
  revision: u64,
  still_position_ms: u64,
  strip_positions_ms: Vec<u64>,
}

static OWED: Mutex<Option<Owed>> = Mutex::new(None);
static REVISIONS: Mutex<u64> = Mutex::new(0);
/// One pass at a time, so an older pass never lands over a newer one.
static PASS: LazyLock<tauri::async_runtime::Mutex<()>> = LazyLock::new(Default::default);
/// The recording whose window closed while its pictures were owed. It is let
/// go once they are made, unless it is opened again first.
static CLOSING: Mutex<Option<u64>> = Mutex::new(None);

fn lock<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
  mutex
    .lock()
    .unwrap_or_else(|poisoned| poisoned.into_inner())
}

/// Keeps the open recording's edit as the one its pictures are owed for, and
/// makes them once no newer edit has come for [`SETTLE`]. The still is taken
/// at `still_position_ms` and the scrub strip at each of
/// `strip_positions_ms`, in order along the edit.
#[tauri::command]
pub async fn request_recording_project_pictures(
  app: AppHandle,
  frame: PreviewFrameRequest,
  still_position_ms: u64,
  strip_positions_ms: Vec<u64>,
) -> Result<(), String> {
  super::strip::check_positions(&strip_positions_ms)?;
  super::edited::open_project(&app, frame.artifact_id)?;
  let revision = {
    let mut revisions = lock(&REVISIONS);
    *revisions += 1;
    *revisions
  };
  *lock(&OWED) = Some(Owed {
    request: frame,
    revision,
    still_position_ms,
    strip_positions_ms,
  });
  tokio::time::sleep(SETTLE).await;
  make_owed(&app, Some(revision)).await
}

/// Makes the owed pictures, if any: with `only`, just when they are still
/// that revision's, so a pass whose edit was overtaken leaves them to the
/// newer one.
async fn make_owed(app: &AppHandle, only: Option<u64>) -> Result<(), String> {
  let _pass = PASS.lock().await;
  let Some(owed) = ({
    let mut owed = lock(&OWED);
    match owed.as_ref() {
      Some(waiting) if only.is_none_or(|revision| waiting.revision == revision) => owed.take(),
      _ => None,
    }
  }) else {
    return Ok(());
  };
  let project = super::edited::open_project(app, owed.request.artifact_id)?;
  let mut positions = Vec::with_capacity(owed.strip_positions_ms.len() + 1);
  positions.push(owed.still_position_ms);
  positions.extend(&owed.strip_positions_ms);
  let mut frames = compose_preview_frames(app, owed.request, positions).await?;
  let strip = frames.split_off(1);
  let still = frames
    .pop()
    .ok_or_else(|| "The still was not composed".to_owned())?;
  super::edited::keep_composed_preview(app, project.clone(), still).await?;
  super::strip::keep_strip(app, project, strip).await?;
  Ok(())
}

fn open_recording(app: &AppHandle) -> Option<u64> {
  let state = app.state::<EditorState>();
  let artifact = lock(&state.recording.artifact);
  match artifact.as_ref() {
    Some(EditorArtifact::Recording { id, .. }) => Some(*id),
    _ => None,
  }
}

/// Closes the recording editor as its window asks. Pictures still owed for
/// its edit are made first, with the window already put away, and the
/// recording is let go after; opening it again meanwhile keeps it open.
pub fn close_recording_editor(app: &AppHandle) {
  let owed_for = lock(&OWED).as_ref().map(|owed| owed.request.artifact_id);
  let Some(open) = open_recording(app).filter(|open| owed_for == Some(*open)) else {
    crate::editor::close(app, EditorKind::Recording);
    return;
  };
  let _ = crate::editor::window::hide(app, EditorKind::Recording);
  *lock(&CLOSING) = Some(open);
  let app = app.clone();
  tauri::async_runtime::spawn(async move {
    if let Err(error) = make_owed(&app, None).await {
      eprintln!("Could not make the closed recording's pictures: {error}");
    }
    let still_closing = lock(&CLOSING).take_if(|closing| *closing == open).is_some();
    if still_closing && open_recording(&app) == Some(open) {
      crate::editor::close(&app, EditorKind::Recording);
    }
  });
}

/// Keeps the recording editor open when it is opened again while a close
/// waits on its pictures.
pub fn keep_recording_editor_open() {
  *lock(&CLOSING) = None;
}
