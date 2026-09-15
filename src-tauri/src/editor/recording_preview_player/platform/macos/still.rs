// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later
//! Paused-frame and scrub decoding for the native preview player.
//!
//! Stills are decoded by the same `AVAssetReader` pipeline as playback and
//! composed by the same GPU compositor, so a paused frame is pixel-identical
//! to the playing frame at that position. Scrubbing decodes at the presented
//! size and the settled frame is refined at full resolution.
#[path = "still/worker.rs"]
mod worker;
use worker::run;

use super::composition::gpu_still_overlay;
use super::cursor::gpu_cursor_preview;
use super::image::frame_position;
use super::still_decode::{scaled_output, DecodedFrame, PaneDecoder};
use crate::editor::annotations::timing::RecordingAnnotationClip;
use crate::editor::preview_platform::{NativeWorkspacePlacement, RecordingWorkspaceLayer};
use crate::editor::recording_preview_player::{PlayerSources, RecordingPreviewPlayerEvent};
use std::sync::{Arc, RwLock};
use std::{sync::atomic::Ordering, sync::mpsc, thread::JoinHandle};
use tauri::ipc::Channel;
enum DecoderCommand {
  Seek {
    annotation_clips: Vec<RecordingAnnotationClip>,
    position_ms: u64,
    request_id: u64,
    target_sizes: Vec<(u32, u32)>,
    /// A mid-gesture skim: the scrubber may land on the cheapest nearby frame
    /// instead of decoding the exact position.
    rough: bool,
  },
  Stop,
}
pub(crate) struct NativeStillDecoder {
  annotation_clips: Arc<RwLock<Vec<RecordingAnnotationClip>>>,
  sender: mpsc::Sender<DecoderCommand>,
  thread: Option<JoinHandle<()>>,
}
struct CachedImages {
  camera: Option<DecodedFrame>,
  camera_ms: Option<u64>,
  screen: DecodedFrame,
  screen_ms: u64,
  sizes: (u32, u32, Option<(u32, u32)>),
}

impl NativeStillDecoder {
  pub(crate) fn spawn(
    sources: PlayerSources,
    event_channel: Channel<RecordingPreviewPlayerEvent>,
  ) -> Result<Self, String> {
    let (sender, receiver) = mpsc::channel();
    let annotation_clips = Arc::clone(&sources.annotation_clips);
    let thread = std::thread::Builder::new()
      .name("recording-preview-still".to_owned())
      .spawn(move || run(sources, receiver, event_channel))
      .map_err(|error| error.to_string())?;
    Ok(Self {
      annotation_clips,
      sender,
      thread: Some(thread),
    })
  }

  pub(crate) fn seek(
    &self,
    position_ms: u64,
    request_id: u64,
    rough: bool,
    target_sizes: Vec<(u32, u32)>,
  ) -> Result<(), String> {
    self
      .sender
      .send(DecoderCommand::Seek {
        annotation_clips: self
          .annotation_clips
          .read()
          .map_err(|_| "The annotations are unavailable")?
          .clone(),
        position_ms,
        request_id,
        rough,
        target_sizes,
      })
      .map_err(|_| "The native preview decoder stopped".to_owned())
  }

  pub(crate) fn is_finished(&self) -> bool {
    self
      .thread
      .as_ref()
      .is_some_and(std::thread::JoinHandle::is_finished)
  }

  pub(crate) fn stop(mut self) {
    let _ = self.sender.send(DecoderCommand::Stop);
    if let Some(thread) = self.thread.take() {
      let _ = thread.join();
    }
  }
}

#[cfg(test)]
#[path = "still_tests.rs"]
mod tests;
