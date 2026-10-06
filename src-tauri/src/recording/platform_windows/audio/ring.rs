// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Audio held in memory for the replay buffer, on the replay's timeline, and
//! written out as the raw sidecars a recording leaves only when a clip is
//! saved. From there a clip's audio is muxed exactly as a recording's is.

use std::sync::Mutex;

use super::raw::on_timeline;
use super::*;
use crate::recording::replay::ring::{AudioChunk, AudioRing};

/// A packet that starts this close after where the ring's audio ends is
/// taken to follow on from it: capture timestamps jitter by a few
/// milliseconds, and laying each packet at its own timestamp would cut
/// clicks into the sound. A longer gap is a real one, kept as silence.
const CONTINUOUS_WITHIN_NS: i64 = 100_000_000;

/// Where captured audio goes.
#[derive(Clone, Copy)]
pub(in crate::recording::platform_windows) enum AudioDestination<'a> {
  /// Raw sidecars beside the recording at this path, muxed when it stops.
  Sidecars(&'a Path),
  /// Memory, for as far back as a replay clip can reach.
  Rings { horizon_ns: i64 },
}

impl AudioDestination<'_> {
  pub(super) fn keep(self, kind: &str, index: usize, channels: u16, sample_rate: u32) -> Keep {
    match self {
      Self::Sidecars(video) => Keep::File(sidecar_path(video, kind, index)),
      Self::Rings { horizon_ns } => Keep::Ring(Arc::new(Mutex::new(AudioRing::new(
        channels,
        sample_rate,
        horizon_ns,
      )))),
    }
  }
}

pub(super) enum Keep {
  File(PathBuf),
  Ring(Arc<Mutex<AudioRing>>),
}

/// Fills `ring` with `packets` for as long as the capture runs.
pub(super) fn fill_ring(
  ring: &Mutex<AudioRing>,
  sample_rate: u32,
  channels: u16,
  origin: &OnceLock<Instant>,
  packets: mpsc::Receiver<Packet>,
) {
  for mut packet in packets {
    let Some(origin) = on_timeline(&mut packet, origin, sample_rate, channels) else {
      continue;
    };
    let pts_ns =
      i64::try_from(packet.captured_at.duration_since(origin).as_nanos()).unwrap_or(i64::MAX);
    let mut ring = ring.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    let pts_ns = match ring.end_ns() {
      Some(end) if pts_ns < end.saturating_add(CONTINUOUS_WITHIN_NS) => end,
      _ => pts_ns,
    };
    ring.push(AudioChunk {
      pts_ns,
      samples: packet.samples,
    });
  }
}

impl RawSink {
  /// The stretch `start_ns..end_ns` of this sink's ring, written as a raw
  /// sidecar at `path` that starts exactly at `start_ns`.
  fn write_clip(&self, path: PathBuf, start_ns: i64, end_ns: i64) -> Result<RawSource, String> {
    let Some(ring) = &self.ring else {
      return Err("This audio capture keeps no replay".to_owned());
    };
    let chunks = ring
      .lock()
      .unwrap_or_else(|poisoned| poisoned.into_inner())
      .clip(start_ns, end_ns);
    let source = RawSource {
      channels: self.channels,
      first_offset_ms: 0,
      path,
      sample_rate: self.sample_rate,
    };
    write_chunks(&source, &chunks, start_ns).inspect_err(|_| {
      let _ = fs::remove_file(&source.path);
    })?;
    Ok(source)
  }
}

/// Writes `chunks` from `start_ns` on, with silence wherever the capture had
/// none, so every sample lands at its own time.
fn write_chunks(source: &RawSource, chunks: &[AudioChunk], start_ns: i64) -> Result<(), String> {
  let file = File::create(&source.path).map_err(|error| error.to_string())?;
  let mut file = BufWriter::new(file);
  let channels = usize::from(source.channels.max(1));
  let frame_at = |pts_ns: i64| {
    let offset = u128::try_from(pts_ns.saturating_sub(start_ns)).unwrap_or(0);
    (offset * u128::from(source.sample_rate) / 1_000_000_000) as usize
  };
  let mut written = 0_usize;
  for chunk in chunks {
    let at = frame_at(chunk.pts_ns);
    // One frame of slack absorbs the rounding between chunks that follow on.
    if at > written + 1 {
      let silence = vec![0.0_f32; (at - written) * channels];
      file
        .write_all(bytemuck::cast_slice(&silence))
        .map_err(|error| error.to_string())?;
      written = at;
    }
    // Raw sidecars are little-endian, as Windows is.
    file
      .write_all(bytemuck::cast_slice(&chunk.samples))
      .map_err(|error| error.to_string())?;
    written += chunk.samples.len() / channels;
  }
  file.flush().map_err(|error| error.to_string())
}

impl AudioCaptures {
  /// The audio of `start_ns..end_ns` as raw sidecars beside `video`, ready
  /// to be muxed the way a recording's audio is.
  pub(in crate::recording::platform_windows) fn clip(
    &self,
    video: &Path,
    start_ns: i64,
    end_ns: i64,
  ) -> Result<AudioFiles, String> {
    let mut written = Vec::new();
    let result = (|| {
      let microphone = match &self.microphone {
        Some(capture) => {
          let path = sidecar_path(video, "microphone", 0);
          let source = capture.sink.write_clip(path, start_ns, end_ns)?;
          written.push(source.path.clone());
          Some(source)
        }
        None => None,
      };
      let mut system = Vec::with_capacity(self.system.len());
      for (index, capture) in self.system.iter().enumerate() {
        let source =
          capture
            .sink
            .write_clip(sidecar_path(video, "system", index), start_ns, end_ns)?;
        written.push(source.path.clone());
        system.push(source);
      }
      Ok(AudioFiles {
        has_microphone: microphone.is_some(),
        has_system_audio: !system.is_empty(),
        microphone,
        system,
      })
    })();
    if result.is_err() {
      for path in written {
        let _ = fs::remove_file(path);
      }
    }
    result
  }
}
