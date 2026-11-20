// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use std::sync::atomic::Ordering;

use super::*;

impl AudioWriter {
  fn track(&self, track: Track) -> Option<&AudioTrack> {
    match track {
      Track::System => self.system_audio.as_ref(),
      Track::Microphone => self.microphone.as_ref(),
    }
  }

  fn track_mut(&mut self, track: Track) -> Option<&mut AudioTrack> {
    match track {
      Track::System => self.system_audio.as_mut(),
      Track::Microphone => self.microphone.as_mut(),
    }
  }

  pub(super) fn append_pcm(&mut self, buffer: MicrophoneBuffer, track: Track) {
    if self.failed.is_some() || self.timeline.is_paused() {
      return;
    }
    let Some(origin) = self.origin else { return };
    let Some(format) = self.track(track).map(AudioTrack::format) else {
      return;
    };
    let Some(buffer) = microphone_buffer_from_origin(buffer, origin, format) else {
      return;
    };
    let pts = self
      .timeline
      .wall_pts_ns(self.elapsed_ns(buffer.captured_at));
    let Some(audio) = self.track_mut(track) else {
      return;
    };
    let let_go = audio.queue(pts, buffer);
    let counter = match track {
      Track::System => &self.stats.audio_not_ready,
      Track::Microphone => &self.stats.microphone_not_ready,
    };
    counter.fetch_add(let_go, Ordering::Relaxed);
    self.drain(track, false);
  }

  /// Writes what `track` holds back, waiting a moment for a busy encoder
  /// when `waiting`.
  fn drain(&mut self, track: Track, waiting: bool) {
    if self.failed.is_some() {
      return;
    }
    let captured_at = self.origin.unwrap_or(self.base);
    let Some(audio) = self.track_mut(track) else {
      return;
    };
    let drained = if waiting {
      audio.drain_waiting(captured_at)
    } else {
      audio.drain(captured_at)
    };
    if let Err(error) = drained {
      self.track_failed(error);
    }
  }

  pub(super) fn drain_backlogs(&mut self, waiting: bool) {
    self.drain(Track::System, waiting);
    self.drain(Track::Microphone, waiting);
  }

  pub(super) fn append_system_audio(&mut self, sample: AudioSample) {
    self.append_pcm(
      MicrophoneBuffer {
        captured_at: sample.wall,
        samples: sample.samples,
      },
      Track::System,
    );
  }

  pub(super) fn pad_tracks_to(&mut self, end_ns: i64) {
    let captured_at = self.origin.unwrap_or(self.base);
    for track in [Track::System, Track::Microphone] {
      let Some(audio) = self.track_mut(track) else {
        continue;
      };
      if audio.end_ns().unwrap_or(-1) >= end_ns {
        continue;
      }
      if let Err(error) = audio.pad_to(end_ns, captured_at) {
        self.track_failed(error);
      }
    }
  }

  fn track_failed(&mut self, error: TrackError) {
    let reason = match error {
      TrackError::Rejected(error) => error,
      TrackError::Stopped => {
        asset_writer_error(&self.writer, "The audio recording could not continue")
      }
    };
    self.fail(reason);
  }
}
