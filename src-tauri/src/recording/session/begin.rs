// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;

/// Opens the capture and the file behind it. Blocking, and slow enough to be
/// worth keeping off the thread that draws.
pub(in crate::recording) fn begin_capture(
  app: &AppHandle,
  options: &StartRecordingOptions,
) -> Result<(CaptureHandles, Receiver<Result<(), String>>), String> {
  if crate::fault::active(crate::fault::Fault::RecordingStart) {
    return Err(crate::fault::message(crate::fault::Fault::RecordingStart));
  }
  let camera_primary = options.mode == RecordingMode::Camera;
  let CaptureSources {
    camera,
    primary,
    system_audio,
  } = capture_sources(options);
  let started_at = Local::now().naive_local();
  let project = crate::project::create(app, &crate::screenshots::capture_file_stem(started_at))?;
  let output_path = project.media.join(if options.mode == RecordingMode::Audio {
    encoding::AUDIO_FILE
  } else {
    encoding::PRIMARY_FILE
  });
  let camera_path = options
    .camera_id
    .as_ref()
    .filter(|_| !camera_primary)
    .map(|_| project.media.join(encoding::CAMERA_FILE));
  // Cursor metadata remains available independently of whether native capture
  // pixels include the pointer. This lets an original-cursor recording turn
  // baking off, or a clean recording turn the editable cursor layer on.
  let cursor_path = records_cursor(options.mode).then(|| project.media.join(encoding::CURSOR_FILE));
  let keyboard_path = records_keyboard(options.mode, options.capture_keyboard_shortcuts)
    .then(|| project.media.join(encoding::KEYBOARD_FILE));
  let include_own_windows = crate::settings::current(app).record_screenwide_windows;

  // Reported at most once per recording, from the writer thread, however many
  // frames the failure goes on to affect.
  let reporter = app.clone();
  let on_failure = std::sync::Arc::new(move |reason: String| {
    report_failure(&reporter, "capture", &reason);
  });

  crate::camera_preview::stop_all(app);
  let monitor = Arc::clone(&state(app).monitor);
  monitor.configure(
    options.system_audio,
    options.microphone_id.is_some(),
    options.camera_id.is_some(),
  );
  let capture::CaptureStart {
    cursor_source,
    first_frame,
    session,
    source_scale_factor,
    timeline_origin,
  } = capture::begin_blocking(CaptureStartupConfig {
    camera,
    camera_path: camera_path.clone(),
    include_own_windows,
    microphone_id: options.microphone_id.clone(),
    monitor,
    on_failure,
    path: output_path.clone(),
    primary,
    system_audio,
  })
  .inspect_err(|_| {
    // A start that never got going leaves no project behind.
    project.remove();
  })?;

  let sidecars = match RecordingSidecars::start(
    SidecarPlan {
      cursor_path: cursor_path.clone(),
      cursor_source,
      keyboard_path: keyboard_path.clone(),
      include_own_windows,
      records_annotations: records_cursor(options.mode),
    },
    timeline_origin,
  ) {
    Ok(sidecars) => sidecars,
    Err(error) => {
      session.cancel();
      project.remove();
      return Err(error);
    }
  };

  // Written as soon as the capture runs, so the project opens even if the app
  // dies before it stops. The scale factor is known only here.
  let manifest = crate::project::RecordingMedia::relative_to(
    &project.file,
    &output_path,
    camera_path.as_deref(),
    cursor_path.as_deref(),
    keyboard_path.as_deref(),
  )
  .and_then(|media| {
    crate::project::write(
      &project.file,
      &crate::project::Manifest::recording(crate::project::RecordingManifest {
        // Unknown until the capture stops.
        duration_ms: None,
        has_microphone: options.microphone_id.is_some(),
        has_system_audio: options.system_audio,
        media,
        primary_kind: primary_kind(options.mode),
        source_scale_factor,
        origin: crate::project::RecordingOrigin::Capture,
      }),
    )
  });
  if let Err(error) = manifest {
    sidecars.cancel();
    session.cancel();
    project.remove();
    return Err(error);
  }

  Ok((
    CaptureHandles {
      sidecars,
      project,
      session,
      source_scale_factor,
    },
    first_frame,
  ))
}

/// What a recording made in `mode` is mainly of.
pub(in crate::recording) fn primary_kind(mode: RecordingMode) -> PrimaryRecordingKind {
  match mode {
    RecordingMode::Audio => PrimaryRecordingKind::Audio,
    RecordingMode::Camera => PrimaryRecordingKind::Camera,
    _ => PrimaryRecordingKind::Screen,
  }
}

/// What `options` asks the platform to capture.
pub(in crate::recording) struct CaptureSources {
  pub camera: Option<CameraCaptureMode>,
  pub primary: PrimaryCaptureSource,
  pub system_audio: SystemAudioSelection,
}

/// `options` as the platform's capture sources. Validated options only:
/// every source the mode needs is present.
pub(in crate::recording) fn capture_sources(options: &StartRecordingOptions) -> CaptureSources {
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
  CaptureSources {
    camera,
    primary,
    system_audio: SystemAudioSelection {
      application_ids: options.system_audio_application_ids.clone(),
      enabled: options.system_audio,
      process_ids: options.system_audio_process_ids.clone(),
    },
  }
}
