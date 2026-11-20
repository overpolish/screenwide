// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::ring::{fill_ring, Keep};
use super::*;
use crate::listening::Heard;
use crate::lost_audio::{lost_before, silence};

/// How many packets may wait for the sidecar writer: some ten seconds of
/// audio, so a writer held up by the disk delays the sound rather than
/// losing it.
const PACKET_QUEUE_DEPTH: usize = 1024;

impl RawSink {
  pub(super) fn start(
    keep: Keep,
    sample_rate: u32,
    channels: u16,
    origin: Arc<OnceLock<Instant>>,
  ) -> Result<Self, String> {
    let (sender, packets) = mpsc::sync_channel(PACKET_QUEUE_DEPTH);
    let (path, ring) = match &keep {
      Keep::File(path) => (Some(path.clone()), None),
      Keep::Ring(ring) => (None, Some(Arc::clone(ring))),
      Keep::Listened(..) => (None, None),
    };
    let worker = thread::Builder::new()
      .name("screenwide-windows-audio-writer".to_owned())
      .spawn(move || match keep {
        Keep::File(path) => write_raw(path, sample_rate, channels, &origin, packets).map(Some),
        Keep::Ring(ring) => {
          fill_ring(&ring, sample_rate, channels, &origin, packets);
          Ok(None)
        }
        Keep::Listened(listener, source) => {
          for mut packet in packets {
            if let Some(origin) = on_timeline(&mut packet, &origin, sample_rate, channels) {
              listener(Heard {
                source,
                samples: packet.samples,
                channels,
                sample_rate,
                at: packet.captured_at.saturating_duration_since(origin),
              });
            }
          }
          Ok(None)
        }
      })
      .map_err(|error| error.to_string())?;
    Ok(Self {
      channels,
      cleanup_on_drop: true,
      path,
      ring,
      sample_rate,
      sender: Some(sender),
      worker: Some(worker),
    })
  }

  pub(super) fn sender(&self) -> Result<mpsc::SyncSender<Packet>, String> {
    self
      .sender
      .as_ref()
      .cloned()
      .ok_or_else(|| "The audio capture has already stopped".to_owned())
  }

  pub(super) fn finish(mut self) -> Result<RawSource, String> {
    self.sender.take();
    let result = self
      .worker
      .take()
      .ok_or_else(|| "The audio writer is unavailable".to_owned())?
      .join()
      .map_err(|_| "The audio writer stopped unexpectedly".to_owned())?;
    self.cleanup_on_drop = false;
    result?.ok_or_else(|| "Replay audio is not written to a file".to_owned())
  }
}

impl Drop for RawSink {
  fn drop(&mut self) {
    self.sender.take();
    if let Some(worker) = self.worker.take() {
      let _ = worker.join();
    }
    if let Some(path) = self.path.as_ref().filter(|_| self.cleanup_on_drop) {
      let _ = fs::remove_file(path);
    }
  }
}

/// Drops whatever of `packet` was captured before the timeline's origin,
/// returning the origin if anything is left.
pub(super) fn on_timeline(
  packet: &mut Packet,
  origin: &OnceLock<Instant>,
  sample_rate: u32,
  channels: u16,
) -> Option<Instant> {
  let origin = origin.get().copied()?;
  let channels = usize::from(channels.max(1));
  let frames = packet.samples.len() / channels;
  if frames == 0 {
    return None;
  }
  let packet_duration = Duration::from_secs_f64(frames as f64 / f64::from(sample_rate));
  let packet_end = packet
    .captured_at
    .checked_add(packet_duration)
    .unwrap_or(packet.captured_at);
  if packet_end <= origin {
    return None;
  }
  if packet.captured_at < origin {
    let skip_frames = (origin.duration_since(packet.captured_at).as_secs_f64()
      * f64::from(sample_rate))
    .ceil() as usize;
    let skip = skip_frames
      .saturating_mul(channels)
      .min(packet.samples.len());
    packet.samples.drain(..skip);
    packet.captured_at = origin;
  }
  (!packet.samples.is_empty()).then_some(origin)
}

fn write_raw(
  path: PathBuf,
  sample_rate: u32,
  channels: u16,
  origin: &OnceLock<Instant>,
  packets: mpsc::Receiver<Packet>,
) -> Result<RawSource, String> {
  let file = File::create(&path).map_err(|error| error.to_string())?;
  let mut file = BufWriter::new(file);
  let format = microphone::Format {
    channels,
    sample_rate,
  };
  let mut first_at = None;
  let mut written_frames = 0_u64;
  // The run between pauses being written: which it is, the capture time it
  // started at, and where in the sidecar that was.
  let mut run: Option<(u64, Instant, i64)> = None;
  for mut packet in packets {
    if on_timeline(&mut packet, origin, sample_rate, channels).is_none() {
      continue;
    }
    first_at.get_or_insert(packet.captured_at);
    let end_ns = frames_ns(written_frames, sample_rate);
    let (run_at, run_ns) = match run {
      Some((id, at, ns)) if id == packet.run => (at, ns),
      _ => {
        run = Some((packet.run, packet.captured_at, end_ns));
        (packet.captured_at, end_ns)
      }
    };
    let since_ns = packet
      .captured_at
      .saturating_duration_since(run_at)
      .as_nanos();
    let pts_ns = run_ns.saturating_add(i64::try_from(since_ns).unwrap_or(i64::MAX));
    if let Some((from_ns, to_ns)) = lost_before(Some(end_ns), pts_ns) {
      for (_, silent) in silence(format, packet.captured_at, from_ns, to_ns) {
        written_frames += write_samples(&mut file, &silent.samples, channels)?;
      }
    }
    written_frames += write_samples(&mut file, &packet.samples, channels)?;
  }
  file.flush().map_err(|error| error.to_string())?;
  let first_offset_ms = first_at
    .map(|first| {
      first
        .saturating_duration_since(*origin.get().unwrap_or(&first))
        .as_millis()
    })
    .and_then(|value| u64::try_from(value).ok())
    .unwrap_or(0);
  Ok(RawSource {
    channels,
    first_offset_ms,
    path,
    sample_rate,
  })
}

fn frames_ns(frames: u64, sample_rate: u32) -> i64 {
  let ns = u128::from(frames) * 1_000_000_000 / u128::from(sample_rate.max(1));
  i64::try_from(ns).unwrap_or(i64::MAX)
}

/// Writes `samples` to the sidecar, answering how many frames they are.
fn write_samples(
  file: &mut BufWriter<File>,
  samples: &[f32],
  channels: u16,
) -> Result<u64, String> {
  // Raw sidecars are little-endian, as Windows is.
  file
    .write_all(bytemuck::cast_slice(samples))
    .map_err(|error| error.to_string())?;
  Ok((samples.len() / usize::from(channels.max(1))) as u64)
}
