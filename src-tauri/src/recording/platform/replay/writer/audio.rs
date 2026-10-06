// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Audio onto the replay timeline, mapped as the recording writers map it.

use super::*;

impl ReplayWriter {
  pub(super) fn flush_preroll(&mut self) {
    for sample in std::mem::take(&mut self.pending_system_audio) {
      self.system_audio(sample);
    }
    for buffer in std::mem::take(&mut self.pending_microphone) {
      self.microphone(buffer);
    }
  }

  pub(super) fn system_audio(&mut self, sample: AudioSample) {
    let mapped = match self.origin_source_ns {
      // Beside video: on the screen's own clock, as in a recording.
      Some(origin_source_ns) => audio_sample_from_origin(sample, origin_source_ns).map(|sample| {
        let pts = self
          .timeline
          .media_pts_ns(sample.source_ns, self.elapsed_ns(sample.wall));
        (pts, sample.samples)
      }),
      // Audio only: on the wall clock from the buffer's start.
      None => {
        let format = MicrophoneFormat {
          channels: SYSTEM_AUDIO_CHANNELS as u16,
          sample_rate: SYSTEM_AUDIO_SAMPLE_RATE as u32,
        };
        self.wall_mapped(
          MicrophoneBuffer {
            captured_at: sample.wall,
            samples: sample.samples,
          },
          format,
        )
      }
    };
    if let (Some((pts_ns, samples)), Some(ring)) = (mapped, self.system_audio.as_mut()) {
      keep(ring, pts_ns, samples);
    }
  }

  pub(super) fn microphone(&mut self, buffer: MicrophoneBuffer) {
    let Some(format) = self.microphone.as_ref().map(|(format, _)| *format) else {
      return;
    };
    let mapped = self.wall_mapped(buffer, format);
    if let (Some((pts_ns, samples)), Some((_, ring))) = (mapped, self.microphone.as_mut()) {
      keep(ring, pts_ns, samples);
    }
  }

  fn wall_mapped(
    &self,
    buffer: MicrophoneBuffer,
    format: MicrophoneFormat,
  ) -> Option<(i64, Vec<f32>)> {
    let buffer = microphone_buffer_from_origin(buffer, self.origin_wall?, format)?;
    let pts = self
      .timeline
      .wall_pts_ns(self.elapsed_ns(buffer.captured_at));
    Some((pts, buffer.samples))
  }
}

/// Keeps audio continuous: a chunk that would overlap the one before it is
/// placed where that one ends, as the recording writers place it.
fn keep(ring: &mut AudioRing, pts_ns: i64, samples: Vec<f32>) {
  let pts_ns = ring.end_ns().map_or(pts_ns, |end| pts_ns.max(end));
  ring.push(AudioChunk { pts_ns, samples });
}
