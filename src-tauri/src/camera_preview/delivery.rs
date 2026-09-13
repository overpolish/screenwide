// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;

pub(super) fn preview_dimensions(width: u32, height: u32) -> (u32, u32) {
  let scale = (f64::from(PREVIEW_MAX_WIDTH) / f64::from(width.max(1)))
    .min(f64::from(PREVIEW_MAX_HEIGHT) / f64::from(height.max(1)))
    .min(1.0);
  (
    (f64::from(width) * scale).round().max(1.0) as u32,
    (f64::from(height) * scale).round().max(1.0) as u32,
  )
}

pub(super) fn frame_payload(frame: Buffer) -> Result<Vec<u8>, String> {
  let resolution = frame.resolution();
  let source_size = (resolution.width(), resolution.height());
  let target_size = preview_dimensions(source_size.0, source_size.1);
  let preserve_mjpeg =
    frame.source_frame_format() == FrameFormat::MJPEG && source_size == target_size;
  let (width, height, frame_data, rgba) = if preserve_mjpeg {
    (source_size.0, source_size.1, frame.buffer().to_vec(), false)
  } else {
    let decoded = match frame.source_frame_format() {
      FrameFormat::YUYV => image::RgbaImage::from_raw(
        source_size.0,
        source_size.1,
        crate::camera_frames::yuyv_to_rgba(frame.buffer(), source_size.0, source_size.1)?,
      )
      .ok_or_else(|| "The camera preview produced an incomplete image".to_owned())?,
      _ => frame
        .decode_image::<RgbAFormat>()
        .map_err(|error| error.to_string())?,
    };
    let decoded = if source_size == target_size {
      decoded
    } else {
      image::imageops::resize(
        &decoded,
        target_size.0,
        target_size.1,
        image::imageops::FilterType::Triangle,
      )
    };
    (target_size.0, target_size.1, decoded.into_raw(), true)
  };
  let mut payload = Vec::with_capacity(9 + frame_data.len());
  payload.extend_from_slice(&width.to_le_bytes());
  payload.extend_from_slice(&height.to_le_bytes());
  payload.push(u8::from(rgba));
  payload.extend(frame_data);
  Ok(payload)
}

pub(super) struct PreviewDelivery {
  cancelled: Arc<AtomicBool>,
  sender: Option<mpsc::SyncSender<Buffer>>,
  thread: Option<std::thread::JoinHandle<()>>,
}

impl PreviewDelivery {
  pub(super) fn spawn(channel: Channel) -> Result<Self, String> {
    Self::spawn_with_sink(move |body| channel.send(body))
  }

  /// The delivery thread never waits on channel disconnect alone: nokhwa can
  /// permanently leak the thread that owns the frame callback, and that
  /// callback holds a sender clone. Polling the cancel flag keeps teardown
  /// bounded to `DELIVERY_POLL_INTERVAL` no matter how many senders survive.
  pub(super) fn spawn_with_sink<S, E>(mut sink: S) -> Result<Self, String>
  where
    S: FnMut(InvokeResponseBody) -> Result<(), E> + Send + 'static,
  {
    let (sender, receiver) = mpsc::sync_channel::<Buffer>(0);
    let cancelled = Arc::new(AtomicBool::new(false));
    let thread_cancelled = Arc::clone(&cancelled);
    let thread = std::thread::Builder::new()
      .name("camera-preview-delivery".to_owned())
      .spawn(move || {
        let mut last_sent = None;
        loop {
          let frame = match receiver.recv_timeout(DELIVERY_POLL_INTERVAL) {
            Ok(frame) => {
              if thread_cancelled.load(Ordering::Acquire) {
                break;
              }
              frame
            }
            Err(mpsc::RecvTimeoutError::Timeout) => {
              if thread_cancelled.load(Ordering::Acquire) {
                break;
              }
              continue;
            }
            Err(mpsc::RecvTimeoutError::Disconnected) => break,
          };
          let now = Instant::now();
          if last_sent.is_some_and(|last| now.duration_since(last) < PREVIEW_INTERVAL) {
            continue;
          }
          let Ok(payload) = frame_payload(frame) else {
            continue;
          };
          if sink(InvokeResponseBody::Raw(payload)).is_err() {
            break;
          }
          last_sent = Some(now);
        }
      })
      .map_err(|error| error.to_string())?;
    Ok(Self {
      cancelled,
      sender: Some(sender),
      thread: Some(thread),
    })
  }

  pub(super) fn sender(&self) -> mpsc::SyncSender<Buffer> {
    self.sender.as_ref().expect("delivery is active").clone()
  }

  pub(super) fn shutdown(&mut self) {
    self.cancelled.store(true, Ordering::Release);
    self.sender.take();
    if let Some(thread) = self.thread.take() {
      let _ = thread.join();
    }
  }

  pub(super) fn stop(mut self) {
    self.shutdown();
  }
}

impl Drop for PreviewDelivery {
  fn drop(&mut self) {
    self.shutdown();
  }
}
