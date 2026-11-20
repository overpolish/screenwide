// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use std::ops::ControlFlow;

use cidre::objc;

use super::*;

impl AudioWriter {
  fn finish(&mut self, at: Instant) -> Result<FinalizeInfo, String> {
    if !self.timeline.has_started() {
      self.writer.cancel_writing();
      return Err("The audio recording did not start".to_owned());
    }
    let stop_ns = self.timeline.stop_pts_ns(self.elapsed_ns(at)).max(1);
    self.drain_backlogs(true);
    self.pad_tracks_to(stop_ns);
    for audio in [self.system_audio.as_mut(), self.microphone.as_mut()]
      .into_iter()
      .flatten()
    {
      audio.finish();
    }
    let end_ns = [self.system_audio.as_ref(), self.microphone.as_ref()]
      .into_iter()
      .flatten()
      .filter_map(AudioTrack::end_ns)
      .fold(0, i64::max);
    self
      .writer
      .end_session_at_src_time(nanos(end_ns))
      .map_err(|error| error.to_string())?;
    self.writer.finish_writing();
    if self.writer.status() != av::AssetWriterStatus::Completed {
      return Err(asset_writer_error(
        &self.writer,
        "The audio recording could not be saved",
      ));
    }
    if let Some(error) = self.failed.take() {
      return Err(error);
    }
    Ok(FinalizeInfo {
      annotation_clips: Vec::new(),
      camera: None,
      corner_radius: None,
      cursor_path: None,
      keyboard_path: None,
      has_microphone: self.microphone.is_some(),
      has_system_audio: self.system_audio.is_some(),
      duration_ms: u64::try_from(end_ns / NANOS_PER_MS).unwrap_or_default(),
      height: 0,
      path: self.path.clone(),
      primary_kind: PrimaryRecordingKind::Audio,
      source_scale_factor: 1.0,
      width: 0,
    })
  }

  pub(crate) fn run(mut self, inbox: &Receiver<Command>, ready: mpsc::Sender<Result<(), String>>) {
    while let Ok(command) = inbox.recv() {
      // A plain `std::thread` has no autorelease pool of its own, and every
      // append leaves autoreleased AVFoundation and CoreMedia scratch behind.
      // Without a pool per command that scratch accumulates for the whole
      // session and is only released when the thread exits: measured on the
      // video writer at ~12MB/s, reaching 13GB in eleven minutes.
      if objc::ar_pool(|| self.handle(command, &ready)).is_break() {
        return;
      }
    }
    self.writer.cancel_writing();
  }

  /// One command, start to finish. `ControlFlow::Break` ends the session:
  /// the writer has been finished or cancelled and must not be used again.
  fn handle(
    &mut self,
    command: Command,
    ready: &mpsc::Sender<Result<(), String>>,
  ) -> ControlFlow<()> {
    // Every command is a chance for audio held back by a busy encoder to go
    // in, ahead of whatever the command brings.
    self.drain_backlogs(false);
    match command {
      Command::Begin { at } => {
        let _ = ready.send(self.begin(at));
      }
      Command::SystemAudio(sample) if self.origin.is_none() => {
        if self.pending_system_audio.len() == SYSTEM_AUDIO_PREROLL_LIMIT {
          self.pending_system_audio.pop_front();
        }
        self.pending_system_audio.push_back(sample);
      }
      Command::SystemAudio(sample) => self.append_system_audio(sample),
      Command::Microphone(buffer) => {
        if self.origin.is_none() {
          if self.pending_microphone.len() == MICROPHONE_PREROLL_LIMIT {
            self.pending_microphone.pop_front();
          }
          self.pending_microphone.push_back(buffer);
        } else {
          self.append_pcm(buffer, Track::Microphone);
        }
      }
      Command::MicrophoneFailed(error) => self.fail(error),
      Command::Pause { at } => self.timeline.pause(self.elapsed_ns(at)),
      Command::Resume { at } => self.timeline.resume(self.elapsed_ns(at)),
      Command::Stop { at, reply } => {
        let _ = reply.send(self.finish(at));
        return ControlFlow::Break(());
      }
      Command::Cancel => {
        self.writer.cancel_writing();
        return ControlFlow::Break(());
      }
      Command::Frame(_) => self.fail("Audio-only capture received a video frame".to_owned()),
      Command::Replay(request) => request.refuse(),
    }
    ControlFlow::Continue(())
  }
}
