// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The island's camera picture during a Windows recording.
//!
//! The capture callback samples each frame down to thumbnail size itself: a
//! few thousand reads, where handing the worker a full frame would mean
//! copying megabytes. Sending the thumbnail over IPC happens here, off the
//! capture thread, and only for the latest one.

use std::{
  sync::{mpsc, Arc},
  thread::JoinHandle,
};

use crate::monitor::RecordingMonitor;

const MAX_WIDTH: u32 = 128;
const MAX_HEIGHT: u32 = 72;

/// A camera frame's pixels as the capture callback holds them.
pub(super) enum Pixels<'a> {
  Rgba(&'a [u8]),
  /// Contiguous NV12: the luma plane, then interleaved chroma at half height.
  Nv12(&'a [u8]),
}

pub(super) struct Thumbnail {
  height: u16,
  rgba: Vec<u8>,
  width: u16,
}

pub(super) struct ConfidenceWorker {
  sender: Option<mpsc::SyncSender<Thumbnail>>,
  thread: Option<JoinHandle<()>>,
}

impl ConfidenceWorker {
  pub(super) fn spawn(monitor: Arc<RecordingMonitor>) -> Result<Self, String> {
    // A rendezvous channel has no stale-frame backlog: capture offers every
    // current thumbnail, and drops it immediately while this worker is busy.
    let (sender, receiver) = mpsc::sync_channel::<Thumbnail>(0);
    let thread = std::thread::Builder::new()
      .name("screenwide-camera-confidence-windows".to_owned())
      .spawn(move || {
        while let Ok(thumbnail) = receiver.recv() {
          monitor.send_camera(thumbnail.width, thumbnail.height, thumbnail.rgba);
        }
      })
      .map_err(|error| error.to_string())?;
    Ok(Self {
      sender: Some(sender),
      thread: Some(thread),
    })
  }

  pub(super) fn sender(&self) -> mpsc::SyncSender<Thumbnail> {
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

pub(super) fn thumbnail(pixels: Pixels<'_>, width: u32, height: u32, flipped: bool) -> Thumbnail {
  let scale = (f64::from(MAX_WIDTH) / f64::from(width))
    .min(f64::from(MAX_HEIGHT) / f64::from(height))
    .min(1.0);
  let thumbnail_width = (f64::from(width) * scale).round().max(1.0) as u32;
  let thumbnail_height = (f64::from(height) * scale).round().max(1.0) as u32;
  let mut rgba = Vec::with_capacity(thumbnail_width as usize * thumbnail_height as usize * 4);
  for y in 0..thumbnail_height {
    let source_y = (u64::from(y) * u64::from(height) / u64::from(thumbnail_height)) as usize;
    for x in 0..thumbnail_width {
      let sampled_x = (u64::from(x) * u64::from(width) / u64::from(thumbnail_width)) as u32;
      let source_x = if flipped {
        width - 1 - sampled_x
      } else {
        sampled_x
      } as usize;
      let pixel = match pixels {
        Pixels::Rgba(frame) => {
          let at = (source_y * width as usize + source_x) * 4;
          [frame[at], frame[at + 1], frame[at + 2], frame[at + 3]]
        }
        Pixels::Nv12(frame) => {
          let luma = width as usize * height as usize;
          let chroma = luma + (source_y / 2) * width as usize + (source_x & !1);
          studio_bt601_to_rgba(
            frame[source_y * width as usize + source_x],
            frame[chroma],
            frame[chroma + 1],
          )
        }
      };
      rgba.extend_from_slice(&pixel);
    }
  }
  Thumbnail {
    height: thumbnail_height as u16,
    rgba,
    width: thumbnail_width as u16,
  }
}

fn studio_bt601_to_rgba(y: u8, u: u8, v: u8) -> [u8; 4] {
  let c = 298 * (i32::from(y) - 16);
  let d = i32::from(u) - 128;
  let e = i32::from(v) - 128;
  let channel = |value: i32| ((value + 128) >> 8).clamp(0, 255) as u8;
  [
    channel(c + 409 * e),
    channel(c - 100 * d - 208 * e),
    channel(c + 516 * d),
    255,
  ]
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn downsamples_and_mirrors_the_latest_camera_frame() {
    let frame = [1, 2, 3, 4, 10, 20, 30, 40];
    let thumbnail = thumbnail(Pixels::Rgba(&frame), 2, 1, true);
    assert_eq!((thumbnail.width, thumbnail.height), (2, 1));
    assert_eq!(thumbnail.rgba, vec![10, 20, 30, 40, 1, 2, 3, 4]);
  }

  #[test]
  fn samples_nv12_luma_with_its_own_chroma_pair() {
    // Two pixel columns per chroma pair: the left pair is neutral, the right
    // one strongly red, so a misplaced chroma read shows up as colour.
    let frame = [
      235, 235, 16, 16, // luma row 0
      235, 235, 16, 16, // luma row 1
      128, 128, 128, 240, // chroma: neutral, then high V
    ];
    let thumbnail = thumbnail(Pixels::Nv12(&frame), 4, 2, false);
    assert_eq!(&thumbnail.rgba[0..4], &[255, 255, 255, 255]);
    let red_corner = &thumbnail.rgba[8..12];
    assert!(red_corner[0] > red_corner[1] && red_corner[0] > red_corner[2]);
  }
}
