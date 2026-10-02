// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

mod scaler;
#[cfg(test)]
mod tests;

use std::{sync::mpsc, thread::JoinHandle};

use cidre::{arc, cv};

use crate::recording::monitor::RecordingMonitor;
use scaler::Scaler;

const MAX_WIDTH: usize = 96;
const MAX_HEIGHT: usize = 54;

pub(super) struct CameraFrame(pub(super) arc::R<cv::PixelBuf>);

// SAFETY: the capture callback retains the pixel buffer, moves that ownership
// into this bounded channel and never accesses that retained reference again.
// Only this worker reads it afterwards.
unsafe impl Send for CameraFrame {}

pub(super) struct ConfidenceWorker {
  sender: Option<mpsc::SyncSender<CameraFrame>>,
  thread: Option<JoinHandle<()>>,
}

impl ConfidenceWorker {
  pub(super) fn spawn(monitor: std::sync::Arc<RecordingMonitor>) -> Result<Self, String> {
    // There is deliberately no backlog: while this worker scales one frame,
    // capture drops confidence frames and hands over the next current one as
    // soon as the worker is waiting again.
    let (sender, receiver) = mpsc::sync_channel::<CameraFrame>(0);
    let thread = std::thread::Builder::new()
      .name("screenwide-camera-confidence".to_owned())
      .spawn(move || {
        // A failed scaler still drains the channel: capture uses a rendezvous
        // send, so a receiver that stops listening would stall it.
        let mut scaler = match Scaler::create() {
          Ok(scaler) => Some(scaler),
          Err(error) => {
            eprintln!("The camera confidence thumbnail scaler is unavailable: {error}");
            None
          }
        };
        while let Ok(frame) = receiver.recv() {
          let Some(scaler) = scaler.as_mut() else {
            continue;
          };
          let Some((width, height)) = thumbnail_size(&frame.0) else {
            continue;
          };
          let mut pixels = vec![0_u8; usize::from(width) * usize::from(height) * 4];
          if scaler.thumbnail(&frame.0, width, height, &mut pixels) {
            monitor.send_camera(width, height, pixels);
          }
        }
      })
      .map_err(|error| error.to_string())?;
    Ok(Self {
      sender: Some(sender),
      thread: Some(thread),
    })
  }

  pub(super) fn sender(&self) -> mpsc::SyncSender<CameraFrame> {
    self.sender.as_ref().expect("worker is active").clone()
  }

  pub(super) fn stop(mut self) {
    self.sender.take();
    if let Some(thread) = self.thread.take() {
      let _ = thread.join();
    }
  }
}

impl Drop for ConfidenceWorker {
  fn drop(&mut self) {
    self.sender.take();
    if let Some(thread) = self.thread.take() {
      let _ = thread.join();
    }
  }
}

/// The thumbnail keeps the source aspect ratio inside the recording bar's
/// budget and never upscales a camera that is already smaller.
fn thumbnail_size(buffer: &cv::PixelBuf) -> Option<(u16, u16)> {
  let source_width = buffer.width();
  let source_height = buffer.height();
  if source_width == 0 || source_height == 0 {
    return None;
  }
  let scale = (MAX_WIDTH as f64 / source_width as f64)
    .min(MAX_HEIGHT as f64 / source_height as f64)
    .min(1.0);
  let width = ((source_width as f64 * scale).round() as u16).max(1);
  let height = ((source_height as f64 * scale).round() as u16).max(1);
  Some((width, height))
}
