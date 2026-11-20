// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The writer's audio tracks (`AudioTrack`): buffers queued in order, what
//! a busy encoder made wait or lost counted, and refusals reported as frames'
//! are.

use super::*;

impl Writer {
  fn track_mut(&mut self, track: Track) -> Option<&mut AudioTrack> {
    match track {
      Track::System => self.system_audio.as_mut(),
      Track::Microphone => self.microphone.as_mut(),
    }
  }

  /// Takes a buffer stamped `pts_ns` for `track`, written now or as soon as
  /// the encoder is ready for it.
  pub(super) fn queue_audio(&mut self, track: Track, pts_ns: i64, buffer: MicrophoneBuffer) {
    let Some(audio) = self.track_mut(track) else {
      return;
    };
    let let_go = audio.queue(pts_ns, buffer);
    let counter = match track {
      Track::System => &self.stats.audio_not_ready,
      Track::Microphone => &self.stats.microphone_not_ready,
    };
    counter.fetch_add(let_go, Ordering::Relaxed);
    self.drain_audio(track, false);
  }

  /// Writes what `track` holds back, waiting a moment for a busy encoder
  /// when `waiting`.
  fn drain_audio(&mut self, track: Track, waiting: bool) {
    if self.failed.is_some() {
      return;
    }
    let captured_at = self.origin_wall.unwrap_or(self.base);
    let Some(audio) = self.track_mut(track) else {
      return;
    };
    let drained = if waiting {
      audio.drain_waiting(captured_at)
    } else {
      audio.drain(captured_at)
    };
    match drained {
      Ok(silence_ns) => {
        let counter = match track {
          Track::System => &self.stats.audio_silence_ns,
          Track::Microphone => &self.stats.microphone_silence_ns,
        };
        counter.fetch_add(silence_ns, Ordering::Relaxed);
      }
      Err(error) => self.audio_refused(track, error),
    }
  }

  pub(super) fn drain_audio_backlogs(&mut self) {
    self.drain_audio(Track::System, false);
    self.drain_audio(Track::Microphone, false);
  }

  /// Writes what the tracks still hold back before the movie is closed.
  pub(super) fn flush_audio_backlogs(&mut self) {
    self.drain_audio(Track::System, true);
    self.drain_audio(Track::Microphone, true);
  }

  pub(super) fn audio_refused(&mut self, track: Track, error: TrackError) {
    let counter = match track {
      Track::System => &self.stats.audio_rejected,
      Track::Microphone => &self.stats.microphone_rejected,
    };
    counter.fetch_add(1, Ordering::Relaxed);
    let name = track.name();
    let reason = match error {
      TrackError::Rejected(error) => format!("{name} sample was rejected: {error}"),
      TrackError::Stopped => format!(
        "{name} track failed: {}",
        asset_writer_error(&self.writer, "the audio track could not continue")
      ),
    };
    self.refused(reason);
  }
}
