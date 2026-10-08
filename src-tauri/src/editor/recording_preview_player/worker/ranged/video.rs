// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Each kept range plays from its own decoder. Opening one seeks and decodes
//! its first frame, and joining one waits out its last decode, both of which
//! can outlast many frames. The playback loop paces frames against the audio
//! clock, which keeps running meanwhile, so the loop never waits on either:
//! the next range's decoder opens on a helper thread while the current range
//! plays, and a finished decoder is cancelled at once but joined only after
//! playback ends.

use std::{
  process::Child,
  sync::{
    atomic::{AtomicBool, Ordering},
    mpsc::{self, Receiver},
    Arc, Mutex,
  },
  thread::JoinHandle,
};

use super::{effective_rate, platform, RunContext};
use crate::editor::recording_preview_player::video::VideoFrame;
use crate::editor::recording_preview_player::{PlayerSources, RecordingPreviewPlaybackRange};

/// What opening a range's decoder needs, owned so a helper thread can do it.
pub(super) struct VideoSource {
  sources: PlayerSources,
  playback_factors: Vec<f64>,
  playback_rate: f64,
  video_child: Arc<Mutex<Option<Child>>>,
}

impl VideoSource {
  pub(super) fn new(context: &RunContext) -> Arc<Self> {
    Arc::new(Self {
      sources: context.sources.clone(),
      playback_factors: context.playback_factors.clone(),
      playback_rate: context.playback_rate,
      video_child: Arc::clone(&context.video_child),
    })
  }

  /// Opens the range's decoder on the calling thread, blocking until the
  /// platform has it ready.
  fn open(&self, range: RecordingPreviewPlaybackRange) -> Result<VideoPlayback, String> {
    let cancelled = Arc::new(AtomicBool::new(false));
    let (sender, frames) = mpsc::sync_channel(3);
    let thread = platform::spawn_video(
      &self.sources,
      &self.playback_factors,
      range.source_start_ms,
      effective_rate(range, self.playback_rate),
      false,
      Arc::clone(&cancelled),
      Arc::clone(&self.video_child),
      sender,
    )?;
    Ok(VideoPlayback {
      cancelled,
      frames,
      thread,
    })
  }

  /// Starts opening the range's decoder on a helper thread.
  pub(super) fn prepare(self: &Arc<Self>, range: RecordingPreviewPlaybackRange) -> PreparedVideo {
    let source = Arc::clone(self);
    PreparedVideo(
      std::thread::Builder::new()
        .name("recording-preview-video-open".to_owned())
        .spawn(move || source.open(range))
        .map_err(|error| error.to_string()),
    )
  }
}

pub(super) struct VideoPlayback {
  cancelled: Arc<AtomicBool>,
  pub(super) frames: Receiver<VideoFrame>,
  thread: JoinHandle<()>,
}

impl VideoPlayback {
  /// Stops the decoder without waiting for its thread to exit.
  pub(super) fn cancel(self) -> JoinHandle<()> {
    self.cancelled.store(true, Ordering::Release);
    // Dropping the receiver drops any queued frame with it, which releases a
    // decoder waiting for that frame to be presented.
    drop(self.frames);
    self.thread
  }
}

/// A decoder opening on its own thread.
pub(super) struct PreparedVideo(Result<JoinHandle<Result<VideoPlayback, String>>, String>);

impl PreparedVideo {
  /// Waits for the decoder, which normally finished opening while the range
  /// before it played.
  pub(super) fn ready(self) -> Result<VideoPlayback, String> {
    self
      .0?
      .join()
      .unwrap_or_else(|_| Err("The preview decoder panicked while opening".to_owned()))
  }
}
