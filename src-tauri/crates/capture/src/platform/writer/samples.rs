// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;

impl Writer {
  pub(super) fn flush_system_audio_preroll(&mut self) {
    let pending = std::mem::take(&mut self.pending_system_audio);
    for sample in pending {
      self.append_system_audio_from_origin(sample);
    }
  }

  /// AVAssetWriter omits an audio input that never receives a sample. A silent
  /// recording would therefore shift every later audio stream index and leave
  /// export metadata pointing at tracks that do not exist. One AAC packet at
  /// time zero materializes the selected track.
  pub(super) fn ensure_system_audio_track(&mut self) {
    let captured_at = self.origin_wall.unwrap_or(self.base);
    let Some(audio) = self
      .system_audio
      .as_mut()
      .filter(|audio| audio.end_ns().is_none() && !audio.is_holding())
    else {
      return;
    };
    let packet_ns = 1_024 * NANOS_PER_SEC / SYSTEM_AUDIO_SAMPLE_RATE;
    if let Err(error) = audio.pad_to(packet_ns, captured_at) {
      self.audio_refused(Track::System, error);
    }
  }

  pub(super) fn append_system_audio_from_origin(&mut self, sample: AudioSample) {
    let Some(origin_source_ns) = self.origin_source_ns else {
      return;
    };
    if let Some(sample) = audio_sample_from_origin(sample, origin_source_ns) {
      self.append_mapped_system_audio(sample);
    }
  }

  fn append_mapped_system_audio(&mut self, sample: AudioSample) {
    let pts = self
      .timeline
      .media_pts_ns(sample.source_ns, self.elapsed_ns(sample.wall));
    let buffer = MicrophoneBuffer {
      captured_at: sample.wall,
      samples: sample.samples,
    };
    self.queue_audio(Track::System, pts, buffer);
  }

  pub(super) fn flush_microphone_preroll(&mut self) {
    let pending = std::mem::take(&mut self.pending_microphone);
    for microphone in pending {
      self.append_microphone_from_origin(microphone);
    }
  }

  pub(super) fn append_microphone_from_origin(&mut self, microphone: MicrophoneBuffer) {
    let Some(format) = self.microphone.as_ref().map(AudioTrack::format) else {
      return;
    };
    let Some(origin) = self.origin_wall else {
      return;
    };
    let Some(microphone) = microphone_buffer_from_origin(microphone, origin, format) else {
      return;
    };
    let pts = self
      .timeline
      .wall_pts_ns(self.elapsed_ns(microphone.captured_at));
    self.queue_audio(Track::Microphone, pts, microphone);
  }

  /// Appends one frame, reporting whether it actually landed.
  ///
  /// A writer that has failed is never asked again: AVFoundation refuses every
  /// subsequent sample, and asking it sixty times a second turns one fault
  /// into a flood of identical complaints.
  pub(super) fn append(&mut self, frame: &Frame, pts_ns: i64) -> bool {
    if self.failed.is_some() {
      self.stats.rejected.fetch_add(1, Ordering::Relaxed);
      return false;
    }
    if !self.video_input_is_ready() {
      self.stats.not_ready.fetch_add(1, Ordering::Relaxed);
      return false;
    }

    match self
      .adaptor
      .append_pixel_buf_with_pts(&frame.buf, nanos(pts_ns))
    {
      Ok(true) => {
        self.stats.appended.fetch_add(1, Ordering::Relaxed);
        self.rejection_streak = 0;
        self.last_appended_ns = Some(pts_ns);
        self.read_picture(frame, pts_ns);
        true
      }
      Ok(false) => {
        self.refused(format!(
          "Video track failed: {}",
          asset_writer_error(&self.writer, "the recording could not continue")
        ));
        false
      }
      Err(error) => {
        self.refused(format!("Video sample was rejected: {error}"));
        false
      }
    }
  }

  /// A screen frame may be replaced by the next changed frame, so it is
  /// dropped immediately under backpressure. Every camera frame represents a
  /// point in continuous motion: wait on this writer thread while the bounded
  /// capture queue absorbs a short hardware-encoder stall. The capture
  /// callback itself always remains non-blocking.
  fn video_input_is_ready(&self) -> bool {
    if self.input.is_ready_for_more_media_data() {
      return true;
    }
    if !matches!(self.source, VideoSource::Camera) {
      return false;
    }

    let deadline = Instant::now() + CAMERA_ENCODER_WAIT;
    while Instant::now() < deadline {
      std::thread::sleep(CAMERA_ENCODER_POLL);
      if self.input.is_ready_for_more_media_data() {
        return true;
      }
    }
    false
  }
}
