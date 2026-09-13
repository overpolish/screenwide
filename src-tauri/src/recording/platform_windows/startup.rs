// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;

pub fn begin_blocking(config: CaptureStartupConfig) -> Result<CaptureStart, String> {
  let CaptureStartupConfig {
    camera,
    camera_path,
    include_own_windows: _,
    microphone_id,
    monitor: recording_monitor,
    on_failure,
    path,
    primary,
    system_audio,
    system_audio_skipped: _,
  } = config;
  let primary = match primary {
    PrimaryCaptureSource::Audio => {
      return begin_audio_only(
        microphone_id.as_deref(),
        &system_audio,
        recording_monitor,
        on_failure,
        path,
      );
    }
    primary => primary,
  };
  let camera_primary = matches!(primary, PrimaryCaptureSource::Camera);
  let mut camera_spec = camera.map(camera::CameraSpec::resolve).transpose()?;
  if camera_primary && camera_spec.is_none() {
    return Err("No camera is selected to record".to_owned());
  }
  let camera_selected = camera_spec.is_some();
  let ResolvedSource {
    width,
    height,
    fps,
    primary_kind,
    source_scale_factor,
    cursor_source,
    wall_timestamped_frames,
    graphics_source,
    desktop_plan,
    source_crop,
  } = resolve_source(primary, camera_spec.as_ref())?;

  let timeline_origin = Arc::new(OnceLock::new());
  let audio = audio::AudioCaptures::start(
    microphone_id.as_deref(),
    &system_audio,
    Arc::clone(&timeline_origin),
    Arc::clone(&recording_monitor),
    Arc::clone(&on_failure),
    &path,
  )?;
  let device = capture::create_device()?;
  let stopped_at = Arc::new(OnceLock::new());
  let (commands, first_frame, worker) = spawn_writer(
    "screenwide-windows-recording-writer",
    WriterConfig {
      device: device.clone(),
      establish_timeline_origin: !camera_selected,
      fps,
      height,
      on_failure: Arc::clone(&on_failure),
      path,
      primary_kind,
      source_crop,
      stopped_at: Arc::clone(&stopped_at),
      timeline_origin: Arc::clone(&timeline_origin),
      wall_timestamped_frames,
      width,
    },
  )?;
  let mut session = CaptureSession {
    audio: Some(audio),
    audio_only_clock: None,
    audio_only_path: None,
    camera: None,
    captures: Vec::new(),
    commands: Some(commands.clone()),
    primary_camera: None,
    stopped_at,
    worker: Some(worker),
  };
  let mut camera_first_frame = None;
  if !camera_primary {
    if let Some(spec) = camera_spec.take() {
      let camera_path = camera_path.ok_or_else(|| "The camera has nowhere to record".to_owned())?;
      let (camera_commands, camera_ready, camera_worker) = spawn_writer(
        "screenwide-windows-camera-writer",
        WriterConfig {
          device: device.clone(),
          establish_timeline_origin: false,
          fps: spec.fps,
          height: spec.height,
          on_failure: Arc::clone(&on_failure),
          path: camera_path.clone(),
          primary_kind: super::super::encoding::PrimaryRecordingKind::Camera,
          source_crop: None,
          stopped_at: Arc::clone(&session.stopped_at),
          timeline_origin: Arc::clone(&timeline_origin),
          wall_timestamped_frames: false,
          width: spec.width,
        },
      )?;
      let stream = camera::start(
        spec,
        device.clone(),
        camera_commands.clone(),
        Arc::clone(&timeline_origin),
        Arc::clone(&recording_monitor),
        Arc::clone(&on_failure),
      )?;
      camera_first_frame = Some(camera_ready);
      session.camera = Some(CameraRecording {
        commands: camera_commands,
        path: camera_path,
        stream: Some(stream),
        worker: Some(camera_worker),
      });
    }
  }
  if let Some((plan, show_cursor)) = desktop_plan {
    let coordinator = Arc::new(Mutex::new(DesktopFrameCoordinator::new(
      device.clone(),
      &plan,
    )?));
    let failed = Arc::new(AtomicBool::new(false));
    for (source_index, piece) in plan.pieces.iter().enumerate() {
      let piece = *piece;
      let target = CaptureTarget::Monitor(piece.display_id);
      let (capture_width, capture_height) = capture::target_size(target)?;
      let coordinator = Arc::clone(&coordinator);
      let commands = commands.clone();
      let failed = Arc::clone(&failed);
      let report = Arc::clone(&on_failure);
      session.captures.push(CaptureObjects::start_with_handler(
        device.clone(),
        target,
        capture_width,
        capture_height,
        show_cursor,
        move |frame| {
          let result = coordinator
            .lock()
            .map_err(|_| "The desktop compositor lock was poisoned".to_owned())
            .and_then(|mut coordinator| coordinator.update(source_index, frame));
          match result {
            Ok(Some(frame)) => match commands.try_send(Command::Frame(frame)) {
              Ok(())
              | Err(mpsc::TrySendError::Full(_))
              | Err(mpsc::TrySendError::Disconnected(_)) => {}
            },
            Ok(None) => {}
            Err(error) if !failed.swap(true, Ordering::AcqRel) => report(error),
            Err(_) => {}
          }
        },
      )?);
    }
  } else if let Some((target, capture_width, capture_height, show_cursor)) = graphics_source {
    // Match the user's capture choice exactly. Cursor metadata remains a
    // separate editable layer even when native pixels include the pointer.
    session.captures.push(CaptureObjects::start(
      device,
      target,
      capture_width,
      capture_height,
      show_cursor,
      commands,
    )?);
  } else {
    let spec = camera_spec.expect("camera-primary checked above");
    session.primary_camera = Some(camera::start(
      spec,
      device,
      commands,
      Arc::clone(&timeline_origin),
      recording_monitor,
      on_failure,
    )?);
  }
  let first_frame = match camera_first_frame {
    Some(camera) => both_first_frames(first_frame, camera),
    None => first_frame,
  };

  Ok(CaptureStart {
    cursor_source,
    first_frame,
    session,
    source_scale_factor,
    timeline_origin,
  })
}
