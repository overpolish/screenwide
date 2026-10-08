// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! One pane's playback reader on a thread of its own, so the screen and the
//! camera open, and decode each output tick, at the same time. A reader never
//! leaves the thread that opened it: its Media Foundation runtime guard has to
//! shut down on the thread that started it.

use std::{
  path::PathBuf,
  sync::{
    mpsc::{self, Receiver, SyncSender},
    Arc,
  },
  thread::JoinHandle,
};

use super::gpu_decoder::{GpuFrame, GpuVideoReader};
use crate::editor::preview_platform::RecordingPreviewSurface;

pub(super) struct PaneDecoder {
  opened: Receiver<Result<(), String>>,
  requests: Option<SyncSender<u64>>,
  frames: Receiver<Result<Option<GpuFrame>, String>>,
  thread: Option<JoinHandle<()>>,
}

impl PaneDecoder {
  /// Starts opening the pane's reader at `start_ms`; [`Self::opened`] waits
  /// for it.
  pub(super) fn spawn(
    path: PathBuf,
    start_ms: u64,
    surface: Arc<RecordingPreviewSurface>,
  ) -> Result<Self, String> {
    let (opened_tx, opened) = mpsc::sync_channel(1);
    let (request_tx, requests) = mpsc::sync_channel::<u64>(1);
    let (frame_tx, frames) = mpsc::sync_channel(1);
    let thread = std::thread::Builder::new()
      .name("recording-preview-pane-decoder".to_owned())
      .spawn(move || {
        let mut reader = match GpuVideoReader::open_for_playback(&path, start_ms, surface) {
          Ok(reader) => reader,
          Err(error) => {
            let _ = opened_tx.send(Err(error));
            return;
          }
        };
        if opened_tx.send(Ok(())).is_err() {
          return;
        }
        while let Ok(target_ms) = requests.recv() {
          if frame_tx.send(reader.frame_at(target_ms)).is_err() {
            return;
          }
        }
      })
      .map_err(|error| error.to_string())?;
    Ok(Self {
      opened,
      requests: Some(request_tx),
      frames,
      thread: Some(thread),
    })
  }

  pub(super) fn opened(&self) -> Result<(), String> {
    self
      .opened
      .recv()
      .map_err(|_| "The preview decoder stopped while opening".to_owned())?
  }

  /// Asks for the frame at `target_ms`. The reader keeps the sample behind
  /// its previous frame until it decodes this one, so ask only once that
  /// frame has been presented.
  pub(super) fn request(&self, target_ms: u64) -> bool {
    self
      .requests
      .as_ref()
      .is_some_and(|requests| requests.send(target_ms).is_ok())
  }

  /// The frame the last [`Self::request`] asked for.
  pub(super) fn frame(&self) -> Result<Option<GpuFrame>, String> {
    self
      .frames
      .recv()
      .map_err(|_| "The preview decoder stopped".to_owned())?
  }
}

impl Drop for PaneDecoder {
  fn drop(&mut self) {
    // Closing the request channel ends the thread, which releases its reader
    // where it was opened.
    self.requests.take();
    if let Some(thread) = self.thread.take() {
      let _ = thread.join();
    }
  }
}
