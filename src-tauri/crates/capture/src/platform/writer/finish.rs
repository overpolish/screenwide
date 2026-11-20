// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;
use screenwide_project::PrimaryRecordingKind;

impl Writer {
  /// Appends the closing frame, giving a busy encoder a moment to catch up.
  ///
  /// Ordinary frames are dropped the instant the encoder is busy - there is
  /// another one along in sixteen milliseconds. This one is the last there
  /// will ever be, and the movie's length depends on it landing.
  pub(super) fn append_insisting(&mut self, frame: &Frame, pts_ns: i64) {
    for _ in 0..TAIL_APPEND_ATTEMPTS {
      if self.append(frame, pts_ns) || self.failed.is_some() {
        return;
      }
      std::thread::sleep(TAIL_APPEND_WAIT);
    }
  }

  /// Records a refused frame, and tells the user once if they have stopped
  /// landing altogether.
  pub(super) fn refused(&mut self, reason: String) {
    self.stats.rejected.fetch_add(1, Ordering::Relaxed);
    self.rejection_streak += 1;

    // A failed writer never recovers, so there is no point waiting out the
    // streak before saying so. Short of that, a long enough run of refusals
    // means the same thing by another route.
    let hopeless = self.writer.status() == av::AssetWriterStatus::Failed
      || self.rejection_streak >= REJECTION_STREAK_LIMIT;
    if hopeless {
      self.fail(reason);
    }
  }

  /// Latches the failure and reports it exactly once.
  pub(super) fn fail(&mut self, reason: String) {
    if self.failed.is_some() {
      return;
    }
    eprintln!("Recording stopped accepting frames: {reason}");
    (self.on_failure)(reason.clone());
    self.failed = Some(reason);
  }

  pub(super) fn finish(&mut self, at: Instant) -> Result<FinalizeInfo, String> {
    if !self.timeline.has_started() {
      self.writer.cancel_writing();
      return Err("The recording captured no frames".to_owned());
    }

    let stop_ns = self.timeline.stop_pts_ns(self.elapsed_ns(at));
    // Holding the final frame until the true stop time is what gives a
    // recording of a static screen its real duration, so a busy encoder is
    // worth waiting a moment for rather than giving up on.
    let tail = self.tail.take();
    if let Some(frame) = &tail {
      self.append_insisting(frame, stop_ns);
    }

    self.flush_audio_backlogs();

    self.input.mark_as_finished();
    for audio in [self.system_audio.as_mut(), self.microphone.as_mut()]
      .into_iter()
      .flatten()
    {
      audio.finish();
    }
    // The session ends exactly where the media ends. Ending it any later
    // leaves the movie claiming a duration it has no sample to fill, and the
    // writer refuses the whole file for it - which is what a skipped final
    // frame used to cause, intermittently and only at the very end.
    let end_ns = [self.system_audio.as_ref(), self.microphone.as_ref()]
      .into_iter()
      .flatten()
      .filter_map(AudioTrack::end_ns)
      .fold(self.last_appended_ns.unwrap_or(0), i64::max);
    self
      .writer
      .end_session_at_src_time(nanos(end_ns))
      .map_err(|error| error.to_string())?;
    self.writer.finish_writing();
    self.finish_picture(end_ns);

    if self.writer.status() != av::AssetWriterStatus::Completed {
      return Err(asset_writer_error(
        &self.writer,
        "The recording could not be saved",
      ));
    }

    self.report_capture();

    Ok(FinalizeInfo {
      annotation_clips: Vec::new(),
      camera: None,
      corner_radius: None,
      cursor_path: None,
      keyboard_path: None,
      has_microphone: self.microphone.is_some(),
      has_system_audio: self.system_audio.is_some(),
      duration_ms: u64::try_from(end_ns / NANOS_PER_MS).unwrap_or_default(),
      height: self.height,
      path: self.path.clone(),
      primary_kind: match self.source {
        VideoSource::Camera => PrimaryRecordingKind::Camera,
        VideoSource::Screen => PrimaryRecordingKind::Screen,
      },
      // The recording state owns source geometry and fills this before the
      // artifact is presented. The writer itself only deals in pixels.
      source_scale_factor: 1.0,
      width: self.width,
    })
  }
}
