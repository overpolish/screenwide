// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;

impl RawSink {
  pub(super) fn start(
    path: PathBuf,
    sample_rate: u32,
    channels: u16,
    origin: Arc<OnceLock<Instant>>,
  ) -> Result<Self, String> {
    let (sender, packets) = mpsc::sync_channel(128);
    let worker_path = path.clone();
    let worker = thread::Builder::new()
      .name("screenwide-windows-audio-writer".to_owned())
      .spawn(move || write_raw(worker_path, sample_rate, channels, origin, packets))
      .map_err(|error| error.to_string())?;
    Ok(Self {
      cleanup_on_drop: true,
      path,
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
    result
  }
}

impl Drop for RawSink {
  fn drop(&mut self) {
    self.sender.take();
    if let Some(worker) = self.worker.take() {
      let _ = worker.join();
    }
    if self.cleanup_on_drop {
      let _ = fs::remove_file(&self.path);
    }
  }
}

fn write_raw(
  path: PathBuf,
  sample_rate: u32,
  channels: u16,
  origin: Arc<OnceLock<Instant>>,
  packets: mpsc::Receiver<Packet>,
) -> Result<RawSource, String> {
  let file = File::create(&path).map_err(|error| error.to_string())?;
  let mut file = BufWriter::new(file);
  let channels_usize = usize::from(channels.max(1));
  let mut first_at = None;
  for mut packet in packets {
    let Some(origin) = origin.get().copied() else {
      continue;
    };
    let frames = packet.samples.len() / channels_usize;
    if frames == 0 {
      continue;
    }
    let packet_duration = Duration::from_secs_f64(frames as f64 / f64::from(sample_rate));
    let packet_end = packet
      .captured_at
      .checked_add(packet_duration)
      .unwrap_or(packet.captured_at);
    if packet_end <= origin {
      continue;
    }
    if packet.captured_at < origin {
      let skip_frames = (origin.duration_since(packet.captured_at).as_secs_f64()
        * f64::from(sample_rate))
      .ceil() as usize;
      let skip = skip_frames
        .saturating_mul(channels_usize)
        .min(packet.samples.len());
      packet.samples.drain(..skip);
      packet.captured_at = origin;
    }
    if packet.samples.is_empty() {
      continue;
    }
    first_at.get_or_insert(packet.captured_at);
    for sample in packet.samples {
      file
        .write_all(&sample.to_le_bytes())
        .map_err(|error| error.to_string())?;
    }
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
