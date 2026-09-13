// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

pub(crate) mod commands;
pub(crate) mod cursor;
mod encoding;
pub(crate) mod keyboard;
pub(crate) mod meta_sidecar;
mod microphone;
mod monitor;
#[cfg(target_os = "macos")]
mod platform;
#[cfg(not(any(target_os = "macos", target_os = "windows")))]
mod platform_unsupported;
#[cfg(target_os = "windows")]
mod platform_windows;
mod session;
mod state;
mod types;
mod ui;

#[path = "recording/start_capture.rs"]
mod start_capture;
pub use start_capture::start;

use std::time::Instant;
use tauri::AppHandle;

use crate::windows;

#[cfg(target_os = "macos")]
use platform as capture;
#[cfg(not(any(target_os = "macos", target_os = "windows")))]
use platform_unsupported as capture;
#[cfg(target_os = "windows")]
use platform_windows as capture;

pub use encoding::{CameraFinalizeInfo, FinalizeInfo, PrimaryRecordingKind};
pub use session::recordings_directory;
pub use state::{is_idle, snapshot, RecordingState};
pub use types::{
  RecordingMode, RecordingSnapshot, RecordingStatus, Region, StartRecordingOptions,
  SystemAudioSelection,
};

pub(crate) use session::cancelled_marker;
use session::{
  begin_capture, discard_capture, emit_error, finalize_capture, mark_capture_cancelled,
  pause_capture, require_status, resume_capture, store_handles, take_handles, validate_options,
  FIRST_FRAME_TIMEOUT,
};
#[cfg(test)]
use state::apply_transition;
use state::{set_countdown, state, transition};
pub(crate) use types::CameraCaptureMode;
#[cfg(test)]
use types::DEFAULT_FPS;
pub(crate) use types::{CaptureStartupConfig, PrimaryCaptureSource};
use ui::{prepare_windows, restore_windows, show_recording_ui};

// ---------------------------------------------------------------------------
// Lifecycle. Each entry point validates before it causes any side effect, and
// is callable from both the commands below and the tray menu.
// ---------------------------------------------------------------------------

pub fn pause(app: &AppHandle) -> Result<(), String> {
  transition(app, RecordingStatus::Paused, None).inspect_err(|error| {
    emit_error(app, "pause", error);
  })?;

  if let Some(handles) = state(app)
    .handles
    .lock()
    .unwrap_or_else(|poisoned| poisoned.into_inner())
    .as_ref()
  {
    pause_capture(handles);
  }

  Ok(())
}

pub fn resume(app: &AppHandle) -> Result<(), String> {
  require_status(app, &[RecordingStatus::Paused], "resume").inspect_err(|error| {
    emit_error(app, "resume", error);
  })?;

  let resumed = {
    let state = state(app);
    let handles = state
      .handles
      .lock()
      .unwrap_or_else(|poisoned| poisoned.into_inner());
    handles.as_ref().map_or(Ok(()), resume_capture)
  };
  if let Err(error) = resumed {
    emit_error(app, "resume", &error);
    return Err(error);
  }

  transition(app, RecordingStatus::Recording, None).inspect_err(|error| {
    emit_error(app, "resume", error);
  })?;

  Ok(())
}

pub fn toggle_pause(app: &AppHandle) -> Result<(), String> {
  if snapshot(app).status == RecordingStatus::Paused {
    resume(app)
  } else {
    pause(app)
  }
}

pub fn stop(app: &AppHandle) -> Result<(), String> {
  // This is the user's end point. Finalization runs on a blocking worker and
  // may not be scheduled immediately; sampling the clock there would turn
  // that scheduling delay into a frozen tail in the movie.
  let stopped_at = Instant::now();
  transition(app, RecordingStatus::Stopping, None).inspect_err(|error| {
    emit_error(app, "stop", error);
  })?;
  state(app).cancel();
  let handles = take_handles(app);
  if let Some(handles) = &handles {
    handles.mark_stopped_at(stopped_at);
  }

  let app = app.clone();
  // `tokio` is macOS-only in this crate, so the finalize wait uses a blocking
  // task the way the window animations do.
  tauri::async_runtime::spawn_blocking(move || {
    let finalized = handles.map(|handles| finalize_capture(handles, stopped_at));

    restore_windows(&app);
    if let Err(error) = transition(&app, RecordingStatus::Idle, None) {
      emit_error(&app, "stop", &error);
    }

    match finalized {
      Some(Ok((info, suggested_file_stem))) => {
        if let Err(error) = crate::editor::present_recording(&app, info, suggested_file_stem) {
          crate::editor::release_recording_workspace(&app);
          emit_error(&app, "stop", &error);
          show_recording_ui(&app);
        }
      }
      Some(Err(error)) => {
        crate::editor::release_recording_workspace(&app);
        emit_error(&app, "stop", &error);
        show_recording_ui(&app);
      }
      None => {
        crate::editor::release_recording_workspace(&app);
        show_recording_ui(&app);
      }
    }
  });

  Ok(())
}

pub fn cancel(app: &AppHandle) -> Result<(), String> {
  let status = require_status(
    app,
    &[
      RecordingStatus::Starting,
      RecordingStatus::Recording,
      RecordingStatus::Paused,
    ],
    "be discarded",
  )?;

  if matches!(status, RecordingStatus::Recording | RecordingStatus::Paused) {
    transition(app, RecordingStatus::Stopping, None)?;
  }
  state(app).cancel();

  let handles = take_handles(app);
  if let Some(handles) = &handles {
    // This tiny write must happen before the UI reports Idle. Native capture
    // teardown remains off-thread, but recovery can already distinguish an
    // intentional cancellation from a crash.
    if let Err(error) = mark_capture_cancelled(handles) {
      eprintln!("Could not mark the cancelled recording for cleanup: {error}");
    }
  }
  restore_windows(app);
  transition(app, RecordingStatus::Idle, None)?;
  crate::editor::release_recording_workspace(app);
  show_recording_ui(app);

  // Closing Windows Graphics Capture and joining encoder workers can block,
  // particularly when cancellation lands after the handles were stored but
  // before the first frame was confirmed. The generation was invalidated and
  // the handles were detached above, so late startup completion cannot revive
  // this recording; finish the destructive teardown away from the command/UI
  // path just as normal recording finalization is handled off-thread.
  if handles.is_some() {
    tauri::async_runtime::spawn_blocking(move || discard_capture(handles));
  }

  Ok(())
}

#[cfg(test)]
mod tests;
