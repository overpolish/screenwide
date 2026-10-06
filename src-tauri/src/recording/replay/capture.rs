// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Opening the replay buffer's capture and sidecars, and saving a clip of
//! them as a recording the editor opens like any other.

use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use chrono::Local;
use tauri::AppHandle;

use super::REPLAY_LENGTH;
use crate::annotate::live_clips::ReplayAnnotationRecorder;
use crate::recording::cursor::RollingCursorRecorder;
use crate::recording::keyboard::RollingKeyboardRecorder;
use crate::recording::monitor::RecordingMonitor;
use crate::recording::session::{
  capture_sources, check_inputs, recordings_directory, records_cursor, records_keyboard,
  CaptureSources, FIRST_FRAME_TIMEOUT,
};
use crate::recording::{
  capture, encoding, CameraFinalizeInfo, CaptureStartupConfig, FinalizeInfo, PrimaryRecordingKind,
  RecordingMode, StartRecordingOptions,
};

/// What the sidecars keep beyond the buffer's length. A clip starts on a
/// video keyframe, which can sit a little before the full length.
const SIDECAR_MARGIN: Duration = Duration::from_secs(5);

/// A running replay buffer: the capture, the sidecars beside it, and where
/// the last save ended. Dropping it stops all of them.
pub(super) struct RunningReplay {
  annotations: Option<ReplayAnnotationRecorder>,
  cursor: Option<RollingCursorRecorder>,
  keyboard: Option<RollingKeyboardRecorder>,
  /// The end of the last saved clip, in replay time.
  last_saved_ns: Mutex<Option<i64>>,
  mode: RecordingMode,
  /// Behind a lock because the capture objects are not shareable between
  /// threads, and only one save runs at a time anyway.
  session: Mutex<capture::ReplaySession>,
  source_scale_factor: f32,
}

pub(super) fn start(
  app: &AppHandle,
  options: &StartRecordingOptions,
) -> Result<RunningReplay, String> {
  check_inputs(options)?;
  if crate::fault::active(crate::fault::Fault::ReplayStart) {
    return Err(crate::fault::message(crate::fault::Fault::ReplayStart));
  }
  let CaptureSources {
    camera,
    primary,
    system_audio,
  } = capture_sources(options);
  let include_own_windows = crate::settings::current(app).record_screenwide_windows;
  let reporter = app.clone();
  let capture::ReplayCapture {
    cursor_source,
    first_frame,
    session,
    source_scale_factor,
    timeline_origin,
  } = capture::begin_replay_blocking(
    CaptureStartupConfig {
      camera,
      camera_path: None,
      include_own_windows,
      microphone_id: options.microphone_id.clone(),
      // The buffer's own: the dock shows a recording's levels, not these.
      monitor: Arc::new(RecordingMonitor::default()),
      on_failure: Arc::new(move |reason: String| {
        super::report(&reporter, "Replay buffer stopped", &reason);
        let app = reporter.clone();
        // The writer that failed is the one reporting; stopping joins it, so
        // that happens on a thread of its own.
        std::thread::spawn(move || super::stop(&app));
      }),
      path: std::path::PathBuf::new(),
      primary,
      system_audio,
    },
    REPLAY_LENGTH,
  )?;
  match first_frame.recv_timeout(FIRST_FRAME_TIMEOUT) {
    Ok(Ok(())) => {}
    Ok(Err(error)) => return Err(error),
    Err(_) => return Err("The replay buffer did not receive anything to keep".to_owned()),
  }

  let horizon = REPLAY_LENGTH + SIDECAR_MARGIN;
  let source = cursor_source.filter(|_| records_cursor(options.mode));
  let cursor = source
    .clone()
    .map(|source| {
      RollingCursorRecorder::start(
        timeline_origin.clone(),
        source,
        include_own_windows,
        horizon,
      )
    })
    .transpose()?;
  let keyboard = records_keyboard(options.mode, options.capture_keyboard_shortcuts)
    .then(|| RollingKeyboardRecorder::start(timeline_origin.clone(), horizon))
    .transpose()?;
  let annotations =
    source.map(|source| ReplayAnnotationRecorder::start(timeline_origin, source, horizon));

  Ok(RunningReplay {
    annotations,
    cursor,
    keyboard,
    last_saved_ns: Mutex::new(None),
    mode: options.mode,
    session: Mutex::new(session),
    source_scale_factor,
  })
}

/// Writes the clip ending at `at` and opens it in the editor. Only a clip
/// the editor took moves where the next one starts.
pub(super) fn save(app: &AppHandle, running: &RunningReplay, at: Instant) -> Result<(), String> {
  let directory = recordings_directory(app)?;
  std::fs::create_dir_all(&directory).map_err(|error| error.to_string())?;
  crate::editor::reserve_recording_workspace(app)?;
  let since_ns = *running
    .last_saved_ns
    .lock()
    .unwrap_or_else(|poisoned| poisoned.into_inner());
  let presented = write_clip(running, at, since_ns, &directory).and_then(|(info, stem, end_ns)| {
    crate::editor::present_recording(app, info, stem)?;
    Ok(end_ns)
  });
  match presented {
    Ok(end_ns) => {
      *running
        .last_saved_ns
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner()) = Some(end_ns);
      Ok(())
    }
    Err(error) => {
      crate::editor::release_recording_workspace(app);
      Err(error)
    }
  }
}

fn write_clip(
  running: &RunningReplay,
  at: Instant,
  since_ns: Option<i64>,
  directory: &Path,
) -> Result<(FinalizeInfo, String, i64), String> {
  let saved_at = Local::now().naive_local();
  let audio_only = running.mode == RecordingMode::Audio;
  let path = directory.join(if audio_only {
    encoding::audio_temp_file_name(saved_at)
  } else {
    encoding::temp_file_name(saved_at)
  });
  let camera_path = directory.join(encoding::camera_temp_file_name(saved_at));
  let clip = running
    .session
    .lock()
    .unwrap_or_else(|poisoned| poisoned.into_inner())
    .save(at, since_ns, &path, &camera_path)?;

  let start_us = u64::try_from(clip.start_ns / 1_000).unwrap_or_default();
  let end_us = u64::try_from(clip.end_ns / 1_000).unwrap_or_default();
  let cursor_path = directory.join(encoding::cursor_temp_file_name(saved_at));
  let keyboard_path = directory.join(encoding::keyboard_temp_file_name(saved_at));
  let (cursor, keyboard) = write_sidecars(running, &cursor_path, &keyboard_path, start_us, end_us)
    .inspect_err(|_| remove(&[&path, &camera_path, &cursor_path, &keyboard_path]))?;

  // The suggested name reads as when the clip begins, as a recording's reads
  // as when it was started.
  let clip_length = Duration::from_millis(clip.duration_ms);
  let started_at = saved_at - chrono::Duration::from_std(clip_length).unwrap_or_default();
  let info = FinalizeInfo {
    annotation_clips: running
      .annotations
      .as_ref()
      .map(|annotations| annotations.clips(start_us, end_us))
      .unwrap_or_default(),
    camera: clip.camera.map(|camera| CameraFinalizeInfo {
      duration_ms: camera.duration_ms,
      height: camera.height,
      path: camera_path,
      width: camera.width,
    }),
    cursor_path: cursor,
    keyboard_path: keyboard,
    has_microphone: clip.has_microphone,
    has_system_audio: clip.has_system_audio,
    duration_ms: clip.duration_ms,
    height: clip.height,
    path,
    primary_kind: match running.mode {
      RecordingMode::Audio => PrimaryRecordingKind::Audio,
      RecordingMode::Camera => PrimaryRecordingKind::Camera,
      _ => PrimaryRecordingKind::Screen,
    },
    source_scale_factor: running.source_scale_factor,
    width: clip.width,
  };
  Ok((
    info,
    crate::screenshots::capture_file_stem(started_at),
    clip.end_ns,
  ))
}

fn write_sidecars(
  running: &RunningReplay,
  cursor_path: &Path,
  keyboard_path: &Path,
  start_us: u64,
  end_us: u64,
) -> Result<(Option<PathBuf>, Option<PathBuf>), String> {
  let cursor = match &running.cursor {
    Some(cursor) => {
      cursor.write_clip(cursor_path, start_us, end_us)?;
      Some(cursor_path.to_path_buf())
    }
    None => None,
  };
  let keyboard = match &running.keyboard {
    Some(keyboard) => {
      keyboard.write_clip(keyboard_path, start_us, end_us)?;
      Some(keyboard_path.to_path_buf())
    }
    None => None,
  };
  Ok((cursor, keyboard))
}

fn remove(paths: &[&Path]) {
  for path in paths {
    let _ = std::fs::remove_file(path);
  }
}
