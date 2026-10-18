// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The playback decode thread, the same on every platform: it steps through
//! output ticks from the start position at the playback rate, asks the
//! platform's decoders for each pane's frame at that moment, and hands the
//! pair to the player worker, which presents it on time or drops it late.

use std::sync::{
  atomic::{AtomicBool, Ordering},
  mpsc::{self, SyncSender, TrySendError},
  Arc,
};
use std::time::Duration;

use super::present::{Moment, PaneFrame};
use super::VideoFramePayload;
use crate::editor::recording_preview_player::video::{
  presentation_elapsed_ms, source_position_ms, VideoFrame, OUTPUT_FPS,
};

/// A platform's decoders for one playback range, one per pane.
pub(in crate::editor::recording_preview_player::platform) trait PaneDecoders {
  type Frame: PaneFrame + Send + 'static;
  /// Whether a frame holds on to its decoder's samples, so the next moment
  /// must wait until this one has been presented and released them.
  const HOLDS_SAMPLES: bool;

  /// Each pane's frame at `position_ms`, the screen's first; `None` for a
  /// pane past its end or with nothing new. `Err` ends playback.
  fn frames_at(&mut self, position_ms: u64) -> Result<[Option<Self::Frame>; 2], ()>;
}

/// How long the platform has to open its decoders.
const OPEN_TIMEOUT: Duration = Duration::from_secs(5);

/// Opens the decoders `open` makes on the playback thread, which is where
/// they live, and plays from `start_ms` at `playback_rate` until cancelled,
/// out of frames, or the worker goes away.
pub(in crate::editor::recording_preview_player::platform) fn spawn<D, Open>(
  open: Open,
  start_ms: u64,
  playback_rate: f64,
  factors: (f64, f64),
  cancelled: Arc<AtomicBool>,
  sender: SyncSender<VideoFrame>,
) -> Result<std::thread::JoinHandle<()>, String>
where
  D: PaneDecoders<Frame = super::backend::Frame>,
  Open: FnOnce() -> Result<D, String> + Send + 'static,
{
  let (opened_tx, opened_rx) = mpsc::sync_channel(1);
  let playing = Arc::clone(&cancelled);
  let thread = std::thread::Builder::new()
    .name("recording-preview-video".to_owned())
    .spawn(move || {
      let decoders = match open() {
        Ok(decoders) => {
          let _ = opened_tx.send(Ok(()));
          decoders
        }
        Err(error) => {
          let _ = opened_tx.send(Err(error));
          return;
        }
      };
      play(
        decoders,
        start_ms,
        playback_rate,
        factors,
        &playing,
        &sender,
      );
    })
    .map_err(|error| error.to_string())?;
  match opened_rx.recv_timeout(OPEN_TIMEOUT) {
    Ok(Ok(())) => Ok(thread),
    Ok(Err(error)) => {
      let _ = thread.join();
      Err(error)
    }
    Err(_) => {
      cancelled.store(true, Ordering::Release);
      Err("The preview decoder did not open in time".to_owned())
    }
  }
}

fn play<D: PaneDecoders<Frame = super::backend::Frame>>(
  mut decoders: D,
  start_ms: u64,
  playback_rate: f64,
  factors: (f64, f64),
  cancelled: &AtomicBool,
  sender: &SyncSender<VideoFrame>,
) {
  // One output tick covers this much source time, which is how far an
  // annotation's reveal travels between frames for its blur to fade over.
  let frame_ms = (1_000.0 * playback_rate / OUTPUT_FPS as f64) as f32;
  let mut tick = 0_u64;
  while !cancelled.load(Ordering::Acquire) {
    let position_ms = source_position_ms(start_ms, tick, playback_rate);
    let Ok([screen, camera]) = decoders.frames_at(position_ms) else {
      return;
    };
    if screen.is_none() && camera.is_none() {
      return;
    }
    let (presented, released) = if D::HOLDS_SAMPLES {
      let (presented, released) = mpsc::sync_channel(0);
      (Some(presented), Some(released))
    } else {
      (None, None)
    };
    let frame = VideoFrame {
      presentation_elapsed_ms: presentation_elapsed_ms(tick),
      payload: VideoFramePayload {
        screen,
        camera,
        moment: Moment {
          timestamp_ms: position_ms,
          frame_ms,
          factors,
        },
        presented,
      },
    };
    if !deliver(frame, cancelled, sender) {
      return;
    }
    // A frame the worker drops for being late drops its sender too.
    if let Some(released) = released {
      while !cancelled.load(Ordering::Acquire) {
        match released.recv_timeout(Duration::from_millis(50)) {
          Ok(()) | Err(mpsc::RecvTimeoutError::Disconnected) => break,
          Err(mpsc::RecvTimeoutError::Timeout) => {}
        }
      }
    }
    tick = tick.saturating_add(1);
  }
}

/// Hands `frame` to the worker, waiting while its queue is full. Answers
/// whether playback goes on.
fn deliver(mut frame: VideoFrame, cancelled: &AtomicBool, sender: &SyncSender<VideoFrame>) -> bool {
  loop {
    match sender.try_send(frame) {
      Ok(()) => return true,
      Err(TrySendError::Full(returned)) => {
        if cancelled.load(Ordering::Acquire) {
          return false;
        }
        frame = returned;
        std::thread::sleep(Duration::from_millis(1));
      }
      Err(TrySendError::Disconnected(_)) => return false,
    }
  }
}
