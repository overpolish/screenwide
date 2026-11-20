// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! One audio track of a movie being written: its writer input, the buffers
//! waiting for a busy encoder, and where its audio has reached. A buffer the
//! encoder is not ready for waits in order rather than being lost, and
//! audio lost before a buffer arrived (`crate::lost_audio`) is written back
//! as silence first, so the sound after it stays with the picture.

use std::collections::VecDeque;
use std::time::Instant;

use cidre::{arc, av, cm, ns};

use super::media::{microphone_format_description, microphone_sample_buffer, time_to_ns};
use super::{
  MicrophoneBuffer, MicrophoneFormat, NANOS_PER_MS, SYSTEM_AUDIO_CHANNELS,
  SYSTEM_AUDIO_SAMPLE_RATE, TAIL_APPEND_ATTEMPTS, TAIL_APPEND_WAIT,
};
use crate::lost_audio::{lost_before, silence};

/// How much audio a track holds back while its encoder is not ready. Past
/// this the oldest is let go, and comes back as silence.
const BACKLOG_NS: i64 = 10_000 * NANOS_PER_MS;

/// What ScreenCaptureKit delivers system audio in.
pub(super) const SYSTEM_AUDIO_FORMAT: MicrophoneFormat = MicrophoneFormat {
  channels: SYSTEM_AUDIO_CHANNELS as u16,
  sample_rate: SYSTEM_AUDIO_SAMPLE_RATE as u32,
};

#[derive(Clone, Copy)]
pub(super) enum Track {
  System,
  Microphone,
}

impl Track {
  /// The track's name at the start of a sentence, for failures.
  pub(super) const fn name(self) -> &'static str {
    match self {
      Self::System => "System-audio",
      Self::Microphone => "Microphone",
    }
  }
}

pub(super) enum TrackError {
  /// A buffer could not be made into a sample, or the input refused it.
  Rejected(String),
  /// The input refused a buffer because the writer has stopped.
  Stopped,
}

pub(super) struct AudioTrack {
  backlog: VecDeque<(i64, MicrophoneBuffer)>,
  description: arc::R<cm::AudioFormatDesc>,
  end_ns: Option<i64>,
  format: MicrophoneFormat,
  input: arc::R<av::AssetWriterInput>,
}

impl AudioTrack {
  /// A track of `format` audio, encoded with `settings`, added to `writer`.
  pub(super) fn add_to(
    writer: &mut av::AssetWriter,
    settings: &ns::Dictionary<ns::String, ns::Id>,
    format: MicrophoneFormat,
  ) -> Result<Self, String> {
    let mut input = av::AssetWriterInput::with_media_type_and_output_settings(
      av::MediaType::audio(),
      Some(settings),
    )
    .map_err(|error| error.to_string())?;
    input.set_expects_media_data_in_real_time(true);
    writer
      .add_input(&input)
      .map_err(|error| error.to_string())?;
    Ok(Self {
      backlog: VecDeque::new(),
      description: microphone_format_description(format)?,
      end_ns: None,
      format,
      input,
    })
  }

  pub(super) const fn format(&self) -> MicrophoneFormat {
    self.format
  }

  /// Where the track's audio ends; none before it has any.
  pub(super) const fn end_ns(&self) -> Option<i64> {
    self.end_ns
  }

  /// Whether buffers are waiting for the encoder.
  pub(super) fn is_holding(&self) -> bool {
    !self.backlog.is_empty()
  }

  pub(super) fn finish(&mut self) {
    self.input.mark_as_finished();
  }

  /// Takes a buffer stamped `pts_ns` to write once the buffers before it
  /// are in. Answers how many of the oldest were let go to make room.
  pub(super) fn queue(&mut self, pts_ns: i64, buffer: MicrophoneBuffer) -> u64 {
    self.backlog.push_back((pts_ns, buffer));
    let mut let_go = 0;
    while self
      .backlog
      .front()
      .is_some_and(|(oldest_ns, _)| pts_ns - oldest_ns > BACKLOG_NS)
    {
      self.backlog.pop_front();
      let_go += 1;
    }
    let_go
  }

  /// Writes as much of the backlog as the encoder takes now, silence first
  /// where audio was lost. Answers how much silence went in, in
  /// nanoseconds; silence stamps no time of its own, so it is said to have
  /// been captured at `captured_at`.
  pub(super) fn drain(&mut self, captured_at: Instant) -> Result<u64, TrackError> {
    let mut silence_ns = 0;
    while let Some(pts_ns) = self.backlog.front().map(|(pts_ns, _)| *pts_ns) {
      silence_ns += self.fill_to(pts_ns, captured_at)?;
      if lost_before(self.end_ns, pts_ns).is_some() {
        break;
      }
      let Some((pts_ns, buffer)) = self.backlog.pop_front() else {
        break;
      };
      if !self.write(&buffer, self.end_ns.unwrap_or(pts_ns))? {
        self.backlog.push_front((pts_ns, buffer));
        break;
      }
    }
    Ok(silence_ns)
  }

  /// Drains the backlog, giving a busy encoder a moment to take the rest,
  /// as the closing frame is given one when a recording stops.
  pub(super) fn drain_waiting(&mut self, captured_at: Instant) -> Result<u64, TrackError> {
    let mut silence_ns = 0;
    for _ in 0..TAIL_APPEND_ATTEMPTS {
      silence_ns += self.drain(captured_at)?;
      if !self.is_holding() {
        break;
      }
      std::thread::sleep(TAIL_APPEND_WAIT);
    }
    Ok(silence_ns)
  }

  /// Silence from where the track ends to `end_ns`, and at least a frame of
  /// it, so a track that has had nothing yet still appears in the movie.
  pub(super) fn pad_to(&mut self, end_ns: i64, captured_at: Instant) -> Result<(), TrackError> {
    let from_ns = self.end_ns.unwrap_or(0);
    for (pts_ns, buffer) in silence(self.format, captured_at, from_ns, end_ns.max(from_ns + 1)) {
      if !self.write(&buffer, pts_ns)? {
        break;
      }
    }
    Ok(())
  }

  fn fill_to(&mut self, pts_ns: i64, captured_at: Instant) -> Result<u64, TrackError> {
    let Some((from_ns, to_ns)) = lost_before(self.end_ns, pts_ns) else {
      return Ok(0);
    };
    for (at_ns, buffer) in silence(self.format, captured_at, from_ns, to_ns) {
      if !self.write(&buffer, at_ns)? {
        break;
      }
    }
    Ok(self.end_ns.map_or(0, |end_ns| {
      u64::try_from(end_ns - from_ns).unwrap_or_default()
    }))
  }

  /// Writes `buffer` at `pts_ns`, answering whether the encoder was ready
  /// for it.
  fn write(&mut self, buffer: &MicrophoneBuffer, pts_ns: i64) -> Result<bool, TrackError> {
    if !self.input.is_ready_for_more_media_data() {
      return Ok(false);
    }
    let sample = microphone_sample_buffer(buffer, self.format, &self.description, pts_ns)
      .map_err(TrackError::Rejected)?;
    match self.input.append_sample_buf(&sample) {
      Ok(true) => {
        let duration_ns = time_to_ns(sample.duration()).unwrap_or_default();
        self.end_ns = Some(pts_ns.saturating_add(duration_ns));
        Ok(true)
      }
      Ok(false) => Err(TrackError::Stopped),
      Err(error) => Err(TrackError::Rejected(error.to_string())),
    }
  }
}
