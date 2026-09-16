// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;

impl CaptureSession {
  pub fn mark_stopped_at(&self, at: Instant) {
    let _ = self.stopped_at.set(at);
  }

  pub fn pause_at(&self, at: Instant) {
    if let Some(audio) = &self.audio {
      audio.pause();
    }
    if let Some(clock) = &self.audio_only_clock {
      clock.pause(at);
    }
    if let Some(commands) = &self.commands {
      let _ = commands.send(Command::Pause(at));
    }
    if let Some(camera) = &self.camera {
      let _ = camera.commands.send(Command::Pause(at));
    }
  }

  pub fn resume_at(&self, at: Instant) -> Result<(), String> {
    if let Some(audio) = &self.audio {
      audio.resume();
    }
    if let Some(clock) = &self.audio_only_clock {
      clock.resume(at);
    }
    self.commands.as_ref().map_or(Ok(()), |commands| {
      commands
        .send(Command::Resume(at))
        .map_err(|_| "The recording is no longer running".to_owned())
    })?;
    if let Some(camera) = &self.camera {
      camera
        .commands
        .send(Command::Resume(at))
        .map_err(|_| "The camera recording is no longer running".to_owned())?;
    }
    Ok(())
  }

  pub fn stop_at(mut self, at: Instant) -> Result<FinalizeInfo, String> {
    self.close_sources();
    let audio = self
      .audio
      .take()
      .map(audio::AudioCaptures::finish)
      .transpose()?;
    if let Some(clock) = self.audio_only_clock.take() {
      let audio = audio.ok_or_else(|| "The audio recording has no inputs".to_owned())?;
      let duration_ms = clock.duration_ms(at).max(1);
      let has_system_audio = audio.has_system_audio;
      let has_microphone = audio.has_microphone;
      let path = self
        .audio_only_path
        .take()
        .ok_or_else(|| "The audio recording path is unavailable".to_owned())?;
      audio::mux_audio_only(&path, duration_ms, audio)?;
      return Ok(FinalizeInfo {
        annotation_clips: Vec::new(),
        camera: None,
        cursor_path: None,
        keyboard_path: None,
        duration_ms,
        has_microphone,
        has_system_audio,
        height: 0,
        path,
        primary_kind: crate::recording::PrimaryRecordingKind::Audio,
        source_scale_factor: 1.0,
        width: 0,
      });
    }
    let (reply, replies) = mpsc::channel();
    self
      .commands
      .as_ref()
      .ok_or_else(|| "The recording writer is unavailable".to_owned())?
      .send(Command::Stop { at, reply })
      .map_err(|_| "The recording is no longer running".to_owned())?;
    let mut result = replies
      .recv_timeout(FINALIZE_TIMEOUT)
      .map_err(|_| "The recording did not finish in time".to_owned())?;
    self.join_writer();
    if result.is_ok() {
      if let Some(mut camera) = self.camera.take() {
        let (reply, replies) = mpsc::channel();
        let camera_result = camera
          .commands
          .send(Command::Stop { at, reply })
          .map_err(|_| "The camera recording is no longer running".to_owned())
          .and_then(|()| {
            replies
              .recv_timeout(FINALIZE_TIMEOUT)
              .map_err(|_| "The camera recording did not finish in time".to_owned())?
          });
        if let Some(worker) = camera.worker.take() {
          let _ = worker.join();
        }
        match camera_result {
          Ok(camera_info) => {
            if let Ok(info) = &mut result {
              info.camera = Some(super::super::encoding::CameraFinalizeInfo {
                duration_ms: camera_info.duration_ms,
                height: camera_info.height,
                path: camera_info.path,
                width: camera_info.width,
              });
            }
          }
          Err(error) => {
            eprintln!("Camera recording could not be finalized: {error}");
            let _ = std::fs::remove_file(&camera.path);
          }
        }
      }
    }
    if let (Ok(info), Some(audio)) = (&mut result, audio) {
      let has_system_audio = audio.has_system_audio;
      let has_microphone = audio.has_microphone;
      audio::mux(&info.path, info.duration_ms, audio)?;
      info.has_system_audio = has_system_audio;
      info.has_microphone = has_microphone;
    }
    result
  }

  pub fn cancel(mut self) {
    self.shutdown();
  }

  fn close_sources(&mut self) {
    for mut capture in self.captures.drain(..) {
      capture.close();
    }
    if let Some(camera) = self.primary_camera.take() {
      camera.stop();
    }
    if let Some(camera) = self.camera.as_mut() {
      if let Some(stream) = camera.stream.take() {
        stream.stop();
      }
    }
  }

  fn join_writer(&mut self) {
    if let Some(worker) = self.worker.take() {
      let _ = worker.join();
    }
  }

  fn shutdown(&mut self) {
    self.close_sources();
    self.audio.take();
    if let Some(mut camera) = self.camera.take() {
      let _ = camera.commands.send(Command::Cancel);
      if let Some(worker) = camera.worker.take() {
        let _ = worker.join();
      }
      let _ = std::fs::remove_file(camera.path);
    }
    if let Some(commands) = &self.commands {
      let _ = commands.send(Command::Cancel);
    }
    self.join_writer();
  }
}

impl Drop for CaptureSession {
  fn drop(&mut self) {
    self.shutdown();
  }
}
