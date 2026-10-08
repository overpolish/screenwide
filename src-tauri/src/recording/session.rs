// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

mod begin;
pub(super) use begin::begin_capture;
#[cfg(any(target_os = "macos", target_os = "windows"))]
pub(super) use begin::{capture_sources, primary_kind, CaptureSources};

use std::{
  path::{Path, PathBuf},
  sync::{mpsc::Receiver, Arc},
  time::{Duration, Instant},
};

use chrono::Local;
use serde::Serialize;
use tauri::{AppHandle, Emitter};

use super::{
  capture, encoding, snapshot, state, CameraCaptureMode, CaptureStartupConfig, FinalizeInfo,
  PrimaryCaptureSource, PrimaryRecordingKind, RecordingMode, RecordingStatus,
  StartRecordingOptions, SystemAudioSelection,
};

mod cancellation;
mod inputs;
mod sidecars;

pub(super) use cancellation::{discard_capture, mark_capture_cancelled};
pub(super) use inputs::check_inputs;
use sidecars::{RecordingSidecars, SidecarPlan};
const RECORDING_ERROR_EVENT: &str = "recording://error";
/// How long a start may go without producing a frame before it is called a
/// failure. Permission prompts and display wake-ups are the slow cases and
/// both resolve well inside this.
pub(super) const FIRST_FRAME_TIMEOUT: Duration = Duration::from_secs(5);

/// Everything a running recording is, from the state machine's side: a live
/// capture session and the project it is filling.
pub(super) struct CaptureHandles {
  sidecars: RecordingSidecars,
  project: crate::project::NewProject,
  session: capture::CaptureSession,
  source_scale_factor: f32,
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

/// A failure the user has to be told about: emitted as `emit_error` does, and
/// said in an alert. Rejections by the state machine, such as a second Stop
/// or a Pause while idle, go through `emit_error` alone, since nothing failed.
pub(super) fn report_failure(app: &AppHandle, phase: &'static str, message: &str) {
  emit_error(app, phase, message);
  let title = match phase {
    "start" => crate::i18n::t!("alert-recording-start-failed"),
    "capture" => crate::i18n::t!("alert-recording-capture-failed"),
    "resume" => crate::i18n::t!("alert-recording-resume-failed"),
    _ => crate::i18n::t!("alert-recording-finish-failed"),
  };
  crate::alert::show(app, &title, message);
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

/// Finishes the capture. A stop that produced nothing playable takes its
/// project with it; otherwise the project's manifest is brought up to date
/// with the tracks that were actually written, and returned.
pub(super) fn finalize_capture(
  handles: CaptureHandles,
  stopped_at: Instant,
) -> Result<(FinalizeInfo, PathBuf), String> {
  let CaptureHandles {
    sidecars,
    project,
    session,
    source_scale_factor,
  } = handles;

  let mut stopped_sidecars = sidecars.stop(stopped_at);
  let moments = stopped_sidecars.moments.take();
  let finished = session.stop_at(stopped_at).and_then(|mut info| {
    info.cursor_path = stopped_sidecars.cursor?;
    info.keyboard_path = stopped_sidecars.keyboard?;
    info.annotation_clips = stopped_sidecars.annotation_clips;
    info.source_scale_factor = source_scale_factor;
    Ok(info)
  });
  let info = finished.inspect_err(|_| project.remove())?;
  // The manifest written at the start already opens this project, so a
  // failure here costs only the tracks it would have dropped.
  if let Err(error) = write_manifest(
    &project.file,
    &info,
    moments.as_deref(),
    crate::project::RecordingOrigin::Capture,
  ) {
    eprintln!("Could not update the project's manifest: {error}");
  }
  Ok((info, project.file))
}

/// Describes the finished recording `info`, with the moments placed in it and
/// made as `origin` says, in the manifest at `file`.
pub(super) fn write_manifest(
  file: &Path,
  info: &FinalizeInfo,
  moments: Option<&Path>,
  origin: crate::project::RecordingOrigin,
) -> Result<(), String> {
  let media = crate::project::RecordingMedia::relative_to(
    file,
    &info.path,
    info.camera.as_ref().map(|camera| camera.path.as_path()),
    info.cursor_path.as_deref(),
    info.keyboard_path.as_deref(),
    moments,
  )?;
  crate::project::write(
    file,
    &crate::project::Manifest::recording(crate::project::RecordingManifest {
      duration_ms: Some(info.duration_ms),
      has_microphone: info.has_microphone,
      has_system_audio: info.has_system_audio,
      media,
      primary_kind: info.primary_kind,
      source_scale_factor: info.source_scale_factor,
      origin,
    }),
  )
}
