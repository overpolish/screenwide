// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

#[path = "session/begin.rs"]
mod begin;
pub(super) use begin::begin_capture;

use std::{
  path::PathBuf,
  sync::{mpsc::Receiver, Arc},
  time::{Duration, Instant},
};

use chrono::{Local, NaiveDateTime};
use serde::Serialize;
use tauri::{AppHandle, Emitter, Manager};

use super::{
  capture, encoding, meta_sidecar, snapshot, state, CameraCaptureMode, CaptureStartupConfig,
  FinalizeInfo, PrimaryCaptureSource, RecordingMode, RecordingStatus, StartRecordingOptions,
  SystemAudioSelection,
};

mod cancellation;
mod sidecars;

pub(crate) use cancellation::cancelled_marker;
pub(super) use cancellation::{discard_capture, mark_capture_cancelled};
use sidecars::{RecordingSidecars, SidecarPlan};
const RECORDING_ERROR_EVENT: &str = "recording://error";
/// Emitted when a recording starts without one or more selected inputs whose
/// devices were no longer available; the bar tells the user instead of the
/// start failing outright.
const RECORDING_INPUTS_SKIPPED_EVENT: &str = "recording://inputs-skipped";
/// The folder working files are written to, under the app's data directory.
const RECORDINGS_DIRECTORY: &str = "Recordings";
/// How long a start may go without producing a frame before it is called a
/// failure. Permission prompts and display wake-ups are the slow cases and
/// both resolve well inside this.
pub(super) const FIRST_FRAME_TIMEOUT: Duration = Duration::from_secs(5);

/// Everything a running recording is, from the state machine's side: a live
/// capture session and the file it is filling.
pub(super) struct CaptureHandles {
  sidecars: RecordingSidecars,
  output_path: PathBuf,
  session: capture::CaptureSession,
  source_scale_factor: f32,
  /// Stamped when capture begins, so the suggested file name reads as the
  /// moment the user started rather than the moment they stopped.
  started_at: NaiveDateTime,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct RecordingErrorPayload {
  phase: &'static str,
  message: String,
}

pub(super) fn emit_error(app: &AppHandle, phase: &'static str, message: &str) {
  eprintln!("Recording {phase} failed: {message}");
  let _ = app.emit(
    RECORDING_ERROR_EVENT,
    RecordingErrorPayload {
      phase,
      message: message.to_owned(),
    },
  );
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct RecordingInputsSkippedPayload {
  inputs: Vec<&'static str>,
}

/// Drops selected secondary inputs whose devices no longer exist, so a stale
/// selection degrades the recording instead of failing the start. The primary
/// source is never dropped: a camera recording without its camera must still
/// fail loudly, and an audio recording keeps its sole input so the resolve
/// error names what actually went wrong.
pub(super) fn drop_unavailable_inputs(options: &mut StartRecordingOptions) -> Vec<&'static str> {
  let mut skipped = Vec::new();
  if let Some(microphone_id) = options.microphone_id.clone() {
    let sole_audio_source = options.mode == RecordingMode::Audio && !options.system_audio;
    if !sole_audio_source
      && crate::recording_inputs::resolve_microphone(Some(&microphone_id)).is_err()
    {
      options.microphone_id = None;
      skipped.push("microphone");
    }
  }
  if options.mode != RecordingMode::Camera {
    if let Some(camera_id) = options.camera_id.clone() {
      if !crate::recording_inputs::camera_is_available(&camera_id) {
        options.camera_id = None;
        options.camera_width = None;
        options.camera_height = None;
        options.camera_fps = None;
        options.camera_flipped = false;
        options.camera_pal = false;
        skipped.push("camera");
      }
    }
  }
  skipped
}

pub(super) fn require_status(
  app: &AppHandle,
  allowed: &[RecordingStatus],
  action: &str,
) -> Result<RecordingStatus, String> {
  let status = snapshot(app).status;
  if allowed.contains(&status) {
    Ok(status)
  } else {
    Err(format!(
      "A recording that is {} cannot {action}",
      status.label()
    ))
  }
}

pub(super) fn take_handles(app: &AppHandle) -> Option<CaptureHandles> {
  state(app)
    .handles
    .lock()
    .unwrap_or_else(|poisoned| poisoned.into_inner())
    .take()
}

pub(super) fn store_handles(app: &AppHandle, handles: CaptureHandles) {
  *state(app)
    .handles
    .lock()
    .unwrap_or_else(|poisoned| poisoned.into_inner()) = Some(handles);
}

// ---------------------------------------------------------------------------
// Capture. Every entry point here is called from a blocking task, never from
// the thread that services the UI, and never with a recording lock held.
// ---------------------------------------------------------------------------

/// Where working files live: inside the app's own data directory, so a
/// recording that is never saved leaves nothing in a folder the user looks at.
pub fn recordings_directory(app: &AppHandle) -> Result<PathBuf, String> {
  app
    .path()
    .app_data_dir()
    .map(|directory| directory.join(RECORDINGS_DIRECTORY))
    .map_err(|error| error.to_string())
}

pub(super) fn records_cursor(mode: RecordingMode) -> bool {
  cfg!(any(target_os = "macos", target_os = "windows"))
    && matches!(
      mode,
      RecordingMode::Screen | RecordingMode::Region | RecordingMode::Window
    )
}

pub(super) fn records_keyboard(mode: RecordingMode, enabled: bool) -> bool {
  cfg!(any(target_os = "macos", target_os = "windows"))
    && enabled
    && matches!(
      mode,
      RecordingMode::Screen | RecordingMode::Region | RecordingMode::Window
    )
}

impl CaptureHandles {
  pub(super) fn mark_stopped_at(&self, at: Instant) {
    #[cfg(target_os = "windows")]
    self.session.mark_stopped_at(at);

    #[cfg(not(target_os = "windows"))]
    let _ = at;
  }
}

/// Defence in depth, run before anything is hidden or transitioned. The Record
/// button is already gated on a selected source, but a mode without one could
/// never produce a file.
pub(super) fn validate_options(options: &StartRecordingOptions) -> Result<(), String> {
  match options.mode {
    RecordingMode::Screen | RecordingMode::Region if options.monitor_id.is_none() => {
      Err("No monitor is selected to record".to_owned())
    }
    RecordingMode::Region if options.region.is_none() => {
      Err("No region is selected to record".to_owned())
    }
    RecordingMode::Window if options.window_id.is_none() => {
      Err("No window is selected to record".to_owned())
    }
    RecordingMode::Audio if !options.system_audio && options.microphone_id.is_none() => {
      Err("No audio source is selected to record".to_owned())
    }
    RecordingMode::Camera if options.camera_id.is_none() => {
      Err("No camera is selected to record".to_owned())
    }
    _ if options.camera_id.is_some()
      && (options.camera_width.is_none()
        || options.camera_height.is_none()
        || options.camera_fps.is_none()) =>
    {
      Err("The selected camera mode is incomplete".to_owned())
    }
    _ => Ok(()),
  }
}

pub(super) fn pause_capture(handles: &CaptureHandles) {
  let at = Instant::now();
  handles.session.pause_at(at);
  handles.sidecars.pause(at);
}

pub(super) fn resume_capture(handles: &CaptureHandles) -> Result<(), String> {
  let at = Instant::now();
  handles.session.resume_at(at)?;
  handles.sidecars.resume(at);
  Ok(())
}

pub(super) fn finalize_capture(
  handles: CaptureHandles,
  stopped_at: Instant,
) -> Result<(FinalizeInfo, String), String> {
  // Whatever this stop turns out to be, the metadata sidecar has done its job:
  // either the values reach the editor through `FinalizeInfo` below, or there
  // is no longer a recording for them to describe.
  let meta_path = handles.output_path.clone();
  let finalized = finalize_stopped_capture(handles, stopped_at);
  meta_sidecar::remove(&meta_path);
  finalized
}

fn finalize_stopped_capture(
  handles: CaptureHandles,
  stopped_at: Instant,
) -> Result<(FinalizeInfo, String), String> {
  let CaptureHandles {
    sidecars,
    output_path,
    session,
    source_scale_factor,
    started_at,
  } = handles;

  let stopped_sidecars = sidecars.stop(stopped_at);
  let mut info = match session.stop_at(stopped_at) {
    Ok(info) => info,
    Err(error) => {
      // Nothing playable came out, so nothing is left lying around either.
      let _ = std::fs::remove_file(&output_path);
      sidecars::remove_stopped(&stopped_sidecars);
      return Err(error);
    }
  };
  info.cursor_path = match stopped_sidecars.cursor {
    Ok(path) => path,
    Err(error) => {
      let _ = std::fs::remove_file(&info.path);
      if let Some(camera) = &info.camera {
        let _ = std::fs::remove_file(&camera.path);
      }
      if let Ok(Some(path)) = &stopped_sidecars.keyboard {
        let _ = std::fs::remove_file(path);
      }
      return Err(error);
    }
  };
  info.keyboard_path = match stopped_sidecars.keyboard {
    Ok(path) => path,
    Err(error) => {
      let _ = std::fs::remove_file(&info.path);
      if let Some(camera) = &info.camera {
        let _ = std::fs::remove_file(&camera.path);
      }
      if let Some(path) = &info.cursor_path {
        let _ = std::fs::remove_file(path);
      }
      return Err(error);
    }
  };
  info.annotation_clips = stopped_sidecars.annotation_clips;
  info.source_scale_factor = source_scale_factor;
  Ok((info, crate::screenshots::capture_file_stem(started_at)))
}
