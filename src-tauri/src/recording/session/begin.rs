// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;

/// Opens the capture and the file behind it. Blocking, and slow enough to be
/// worth keeping off the thread that draws.
pub(in crate::recording) fn begin_capture(
  app: &AppHandle,
  options: &StartRecordingOptions,
) -> Result<(CaptureHandles, Receiver<Result<(), String>>), String> {
  // A device that vanished since it was selected must not sink the whole
  // start: drop it, record what was dropped, and tell the user below once the
  // capture is actually running.
  let mut options = options.clone();
  let mut skipped_inputs = drop_unavailable_inputs(&mut options);
  let system_audio_skipped = Arc::new(std::sync::atomic::AtomicBool::new(false));
  let options = &options;
  let camera_primary = options.mode == RecordingMode::Camera;
  let camera = options
    .camera_id
    .as_ref()
    .map(|device_id| CameraCaptureMode {
      device_id: device_id.clone(),
      flipped: options.camera_flipped,
      fps: options.camera_fps.expect("validated above"),
      height: options.camera_height.expect("validated above"),
      pal: options.camera_pal,
      width: options.camera_width.expect("validated above"),
    });
  let directory = recordings_directory(app)?;
  std::fs::create_dir_all(&directory).map_err(|error| error.to_string())?;
  let started_at = Local::now().naive_local();
  let output_path = directory.join(if options.mode == RecordingMode::Audio {
    encoding::audio_temp_file_name(started_at)
  } else {
    encoding::temp_file_name(started_at)
  });
  let camera_path = options
    .camera_id
    .as_ref()
    .filter(|_| !camera_primary)
    .map(|_| directory.join(encoding::camera_temp_file_name(started_at)));
  // Cursor metadata remains available independently of whether native capture
  // pixels include the pointer. This lets an original-cursor recording turn
  // baking off, or a clean recording turn the editable cursor layer on.
  let cursor_path = records_cursor(options.mode)
    .then(|| directory.join(encoding::cursor_temp_file_name(started_at)));
  let keyboard_path = records_keyboard(options.mode, options.capture_keyboard_shortcuts)
    .then(|| directory.join(encoding::keyboard_temp_file_name(started_at)));

  // Reported at most once per recording, from the writer thread, however many
  // frames the failure goes on to affect.
  let reporter = app.clone();
  let on_failure = std::sync::Arc::new(move |reason: String| {
    emit_error(&reporter, "capture", &reason);
  });

  crate::camera_preview::stop_all(app);
  let monitor = Arc::clone(&state(app).monitor);
  monitor.configure(
    options.system_audio,
    options.microphone_id.is_some(),
    options.camera_id.is_some(),
  );
  let primary = match options.mode {
    RecordingMode::Screen => PrimaryCaptureSource::Screen {
      fps: options.fps,
      monitor_id: options.monitor_id.expect("validated above"),
      show_cursor: options.show_cursor,
    },
    RecordingMode::Region => PrimaryCaptureSource::Region {
      fps: options.fps,
      monitor_id: options.monitor_id.expect("validated above"),
      region: options.region.expect("validated above"),
      show_cursor: options.show_cursor,
    },
    RecordingMode::Window => PrimaryCaptureSource::Window {
      fps: options.fps,
      show_cursor: options.show_cursor,
      window_id: options.window_id.expect("validated above"),
    },
    RecordingMode::Camera => PrimaryCaptureSource::Camera,
    RecordingMode::Audio => PrimaryCaptureSource::Audio,
  };
  let capture::CaptureStart {
    cursor_source,
    first_frame,
    session,
    source_scale_factor,
    timeline_origin,
  } = capture::begin_blocking(CaptureStartupConfig {
    camera,
    camera_path: camera_path.clone(),
    include_own_windows: crate::settings::current(app).record_screenwide_windows,
    microphone_id: options.microphone_id.clone(),
    monitor,
    on_failure,
    path: output_path.clone(),
    primary,
    system_audio: SystemAudioSelection {
      application_ids: options.system_audio_application_ids.clone(),
      enabled: options.system_audio,
      process_ids: options.system_audio_process_ids.clone(),
    },
    system_audio_skipped: Arc::clone(&system_audio_skipped),
  })
  .inspect_err(|_| {
    // A start that never got going leaves an empty container behind.
    let _ = std::fs::remove_file(&output_path);
    if let Some(camera_path) = &camera_path {
      let _ = std::fs::remove_file(camera_path);
    }
    if let Some(keyboard_path) = &keyboard_path {
      let _ = std::fs::remove_file(keyboard_path);
    }
  })?;

  let sidecars =
    match RecordingSidecars::start(cursor_path, cursor_source, keyboard_path, timeline_origin) {
      Ok(sidecars) => sidecars,
      Err(error) => {
        session.cancel();
        let _ = std::fs::remove_file(&output_path);
        if let Some(camera_path) = &camera_path {
          let _ = std::fs::remove_file(camera_path);
        }
        return Err(error);
      }
    };

  let system_audio_recorded =
    options.system_audio && !system_audio_skipped.load(std::sync::atomic::Ordering::Acquire);
  if options.system_audio && !system_audio_recorded {
    skipped_inputs.push("systemAudio");
    // The dock was configured before startup discovered the drop; align its
    // layout with what is actually being recorded.
    state(app).monitor.configure(
      false,
      options.microphone_id.is_some(),
      options.camera_id.is_some(),
    );
  }
  if !skipped_inputs.is_empty() {
    // The capture is up; the dock layout already reflects the sanitized
    // inputs via `monitor.configure` above. This tells the user why.
    let _ = app.emit(
      RECORDING_INPUTS_SKIPPED_EVENT,
      RecordingInputsSkippedPayload {
        inputs: skipped_inputs,
      },
    );
  }

  // The scale factor, and which inputs actually made it into the capture, are
  // known only here. Recording them next to the movie is what lets a recovered
  // recording be offered back as what it was rather than as a 1x guess.
  meta_sidecar::write(
    &output_path,
    &meta_sidecar::RecordingMetaSidecar {
      has_microphone: options.microphone_id.is_some(),
      has_system_audio: system_audio_recorded,
      primary_kind: match options.mode {
        RecordingMode::Audio => crate::recording::PrimaryRecordingKind::Audio,
        RecordingMode::Camera => crate::recording::PrimaryRecordingKind::Camera,
        _ => crate::recording::PrimaryRecordingKind::Screen,
      },
      source_scale_factor,
    },
  );

  Ok((
    CaptureHandles {
      sidecars,
      output_path,
      session,
      source_scale_factor,
      started_at,
    },
    first_frame,
  ))
}
